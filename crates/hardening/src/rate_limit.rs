use std::collections::HashMap;
use std::time::Instant;

pub struct RateLimiter {
    limits: HashMap<String, TokenBucket>,
}

struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: f64::from(capacity),
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        if self.tokens >= f64::from(tokens) {
            self.tokens -= f64::from(tokens);
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = elapsed
            .mul_add(self.refill_rate, self.tokens)
            .min(f64::from(self.capacity));
        self.last_refill = now;
    }
}

impl RateLimiter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
        }
    }

    pub fn add_limit(&mut self, key: &str, capacity: u32, refill_rate: f64) {
        self.limits
            .insert(key.to_string(), TokenBucket::new(capacity, refill_rate));
    }

    pub fn try_acquire(&mut self, key: &str, tokens: u32) -> bool {
        if let Some(bucket) = self.limits.get_mut(key) {
            bucket.try_consume(tokens)
        } else {
            true
        }
    }

    #[must_use]
    pub fn remaining(&self, key: &str) -> Option<f64> {
        self.limits.get(key).map(|b| b.tokens)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        let mut limiter = Self::new();
        limiter.add_limit("llm_api", 100, 10.0);
        limiter.add_limit("file_ops", 50, 5.0);
        limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new();
        limiter.add_limit("test", 5, 1.0);

        assert!(limiter.try_acquire("test", 3));
        assert!(limiter.try_acquire("test", 2));
        assert!(!limiter.try_acquire("test", 1));
    }

    #[test]
    fn test_rate_limiter_unknown_key_allows() {
        let mut limiter = RateLimiter::new();
        assert!(limiter.try_acquire("nonexistent", 100));
    }

    #[test]
    fn test_rate_limiter_refill() {
        let mut limiter = RateLimiter::new();
        limiter.add_limit("test", 5, 100.0);

        assert!(limiter.try_acquire("test", 5));
        assert!(!limiter.try_acquire("test", 1));

        std::thread::sleep(Duration::from_millis(20));
        assert!(limiter.try_acquire("test", 1));
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let mut limiter = RateLimiter::new();
        limiter.add_limit("test", 10, 0.0);

        let before = limiter.remaining("test").unwrap();
        assert!(before >= 10.0);

        limiter.try_acquire("test", 3);
        let after = limiter.remaining("test").unwrap();
        assert!(after < before);
    }

    #[test]
    fn test_rate_limiter_nonexistent_key_none() {
        let limiter = RateLimiter::new();
        assert!(limiter.remaining("nonexistent").is_none());
    }

    #[test]
    fn test_rate_limiter_default_has_two_keys() {
        let limiter = RateLimiter::default();
        assert!(limiter.remaining("llm_api").is_some());
        assert!(limiter.remaining("file_ops").is_some());
    }
}
