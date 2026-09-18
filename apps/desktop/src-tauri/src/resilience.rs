use agent_common::error::{AppError, AppResult};
use cache_core::llm_cache::CachedLlmResult;
use cache_core::memory::MemoryCache;
use hardening::resilience::{CircuitBreaker, RetryPolicy};
use hardening::rate_limit::RateLimiter;
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

fn llm_cache() -> &'static MemoryCache<CachedLlmResult> {
    static C: OnceLock<MemoryCache<CachedLlmResult>> = OnceLock::new();
    C.get_or_init(|| MemoryCache::new(500, Duration::from_secs(3600)))
}

/// In-memory LLM response cache (cache-core). Returns a cached response if present.
pub async fn cache_get(key: &str) -> Option<CachedLlmResult> {
    llm_cache().get(key).await
}

pub async fn cache_put(key: String, value: CachedLlmResult) {
    llm_cache().insert(key, value).await;
}

pub fn cache_clear() {
    llm_cache().invalidate_all();
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
    // Rate limit (60 requests, refill 1/s).
    {
        let mut limiters = limiters().lock().unwrap();
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

    // Circuit breaker.
    {
        let mut breakers = breakers().lock().unwrap();
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
                if let Some(cb) = breakers().lock().unwrap().get_mut(provider) {
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
                if let Some(cb) = breakers().lock().unwrap().get_mut(provider) {
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
        let result: AppResult<u32> =
            call_with_resilience("test-ok", || async { Ok(42) }).await;
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
    async fn test_llm_cache_roundtrip() {
        let entry = CachedLlmResult {
            provider: "openai".into(),
            model: "gpt-4o-mini".into(),
            prompt_hash: "abc".into(),
            response: "hello".into(),
            cached_at: "now".into(),
        };
        cache_put("k1".into(), entry.clone()).await;
        let got = cache_get("k1").await.unwrap();
        assert_eq!(got.response, "hello");
        cache_clear();
        assert!(cache_get("k1").await.is_none());
    }
}
