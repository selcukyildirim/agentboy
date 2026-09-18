use agent_common::error::{AppError, AppResult};
use cache_core::llm_cache::CachedLlmResult;
use cache_core::memory::MemoryCache;
use cache_core::metrics::CacheUsageMetrics;
use cache_core::persistent::PersistentCache;
use hardening::rate_limit::RateLimiter;
use hardening::resilience::{CircuitBreaker, RetryPolicy};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

fn breakers() -> &'static Mutex<HashMap<String, CircuitBreaker>> {
    static B: OnceLock<Mutex<HashMap<String, CircuitBreaker>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(HashMap::new()))
}

fn limiters() -> &'static Mutex<HashMap<String, RateLimiter>> {
    static L: OnceLock<Mutex<HashMap<String, RateLimiter>>> = OnceLock::new();
    L.get_or_init(|| Mutex::new(HashMap::new()))
}

fn persistent_cache() -> &'static OnceLock<PersistentCache> {
    static C: OnceLock<PersistentCache> = OnceLock::new();
    &C
}

fn l1_cache() -> &'static MemoryCache<CachedLlmResult> {
    static L: OnceLock<MemoryCache<CachedLlmResult>> = OnceLock::new();
    L.get_or_init(|| MemoryCache::new(500, Duration::from_secs(600)))
}

fn cache_metrics() -> &'static Mutex<CacheUsageMetrics> {
    static M: OnceLock<Mutex<CacheUsageMetrics>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(CacheUsageMetrics::new()))
}

/// Initialise the persistent cache (cache-core) on the shared pool.
pub fn init(pool: SqlitePool) {
    let _ = persistent_cache().set(PersistentCache::new(pool));
}

pub async fn cache_get(key: &str) -> Option<CachedLlmResult> {
    // L1 in-memory.
    if let Some(entry) = l1_cache().get(key).await {
        let mut m = agent_common::sync::lock(cache_metrics());
        m.record_llm_hit(0);
        m.record_l1_hit();
        return Some(entry);
    }

    // L2 persistent.
    let cache = persistent_cache().get()?;
    match cache.get(key).await.ok().flatten() {
        Some(value) => {
            if let Ok(entry) = serde_json::from_str::<CachedLlmResult>(&value) {
                l1_cache().insert(key.to_string(), entry.clone()).await;
                let mut m = agent_common::sync::lock(cache_metrics());
                m.record_llm_hit(0);
                m.record_l2_hit();
                Some(entry)
            } else {
                None
            }
        }
        None => {
            let mut m = agent_common::sync::lock(cache_metrics());
            m.record_llm_miss();
            m.record_l1_miss();
            m.record_l2_miss();
            None
        }
    }
}

pub async fn cache_put(key: String, value: CachedLlmResult) {
    l1_cache().insert(key.clone(), value.clone()).await;
    if let Some(cache) = persistent_cache().get() {
        if let Ok(serialized) = serde_json::to_string(&value) {
            let _ = cache.insert(&key, "llm", &serialized, None).await;
        }
    }
}

pub async fn cache_clear() {
    l1_cache().invalidate_all();
    let _ = clear_persistent_cache("llm").await;
}

pub async fn clear_persistent_cache(entry_type: &str) -> u64 {
    match persistent_cache().get() {
        Some(cache) => cache.invalidate_by_type(entry_type).await.unwrap_or(0),
        None => 0,
    }
}

pub async fn persistent_entry_count() -> u64 {
    match persistent_cache().get() {
        Some(cache) => cache.stats().await.map(|s| s.total_entries).unwrap_or(0),
        None => 0,
    }
}

pub fn cache_metrics_snapshot() -> CacheUsageMetrics {
    agent_common::sync::lock(cache_metrics()).clone()
}

/// Record provider-native prompt-cache token usage (cache-core prompt cache).
pub fn record_prompt_cache(provider: &str, cache_hit: bool, read_tokens: u32, write_tokens: u32) {
    if read_tokens == 0 && write_tokens == 0 {
        return;
    }
    let abstraction = cache_core::prompt_cache::PromptCacheAbstraction::new(
        cache_core::prompt_cache::PromptCacheMode::Auto,
    );
    let normalized = abstraction.normalize_metrics(cache_hit, read_tokens, write_tokens, provider);
    let mut m = agent_common::sync::lock(cache_metrics());
    m.record_prompt_cache(normalized.cache_read_tokens, normalized.cache_write_tokens);
    m.add_cost_saving(normalized.estimated_saving_usd);
}

fn is_infrastructure_error(err: &AppError) -> bool {
    match err {
        AppError::Provider { message, .. } => {
            let m = message.to_lowercase();
            m.contains("timeout")
                || m.contains("rate")
                || m.contains("429")
                || m.contains("500")
                || m.contains("502")
                || m.contains("503")
                || m.contains("network")
                || m.contains("connect")
        }
        _ => false,
    }
}

/// Run an LLM operation through rate limiting, a circuit breaker and retries
/// (hardening). Shared state is keyed by provider id.
pub async fn call_with_resilience<T, F, Fut>(provider: &str, mut op: F) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    {
        let mut limiters = agent_common::sync::lock(limiters());
        let limiter = limiters.entry(provider.to_string()).or_insert_with(|| {
            let mut r = RateLimiter::new();
            r.add_limit(provider, 60, 1.0);
            r
        });
        if !limiter.try_acquire(provider, 1) {
            return Err(AppError::Provider {
                provider: provider.to_string(),
                message: "rate limit exceeded".to_string(),
            });
        }
    }

    {
        let mut breakers = agent_common::sync::lock(breakers());
        let cb = breakers
            .entry(provider.to_string())
            .or_insert_with(|| CircuitBreaker::new(5, Duration::from_secs(30)));
        if !cb.can_execute() {
            return Err(AppError::Provider {
                provider: provider.to_string(),
                message: "circuit breaker open".to_string(),
            });
        }
    }

    let policy = RetryPolicy::new(3, Duration::from_millis(200));
    let mut attempt = 0u32;

    loop {
        attempt += 1;
        match op().await {
            Ok(value) => {
                if let Some(cb) = agent_common::sync::lock(breakers()).get_mut(provider) {
                    cb.record_success();
                }
                return Ok(value);
            }
            Err(err) => {
                if is_infrastructure_error(&err) && policy.should_retry(attempt) {
                    let delay = policy.delay_for_attempt(attempt);
                    tracing::warn!(
                        provider,
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        "retrying after infrastructure error"
                    );
                    tokio::time::sleep(delay).await;
                    continue;
                }
                if let Some(cb) = agent_common::sync::lock(breakers()).get_mut(provider) {
                    cb.record_failure();
                }
                return Err(err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_resilience_success() {
        let result: AppResult<u32> = call_with_resilience("test-ok", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_resilience_retries_then_succeeds() {
        static CALLS: AtomicU32 = AtomicU32::new(0);
        CALLS.store(0, Ordering::SeqCst);
        let result: AppResult<u32> = call_with_resilience("test-retry", || async {
            let n = CALLS.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                Err(AppError::Provider {
                    provider: "test-retry".into(),
                    message: "timeout".into(),
                })
            } else {
                Ok(7)
            }
        })
        .await;
        assert_eq!(result.unwrap(), 7);
        assert!(CALLS.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn test_resilience_non_infra_error_no_retry() {
        static CALLS: AtomicU32 = AtomicU32::new(0);
        CALLS.store(0, Ordering::SeqCst);
        let result: AppResult<u32> = call_with_resilience("test-fatal", || async {
            CALLS.fetch_add(1, Ordering::SeqCst);
            Err(AppError::Validation("bad input".into()))
        })
        .await;
        assert!(result.is_err());
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_cache_metrics_snapshot_defaults() {
        let m = cache_metrics_snapshot();
        assert_eq!(m.llm_cache_hits, 0);
    }
}
