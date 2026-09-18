use std::time::Duration;

pub struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryPolicy {
    #[must_use]
    pub const fn new(max_retries: u32, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    #[must_use]
    pub const fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    #[must_use]
    pub const fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms =
            self.base_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped = delay_ms.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(capped as u64)
    }

    #[must_use]
    pub const fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }

    #[must_use]
    pub const fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(500))
    }
}

pub enum CircuitState {
    Closed,
    Open {
        opened_at: chrono::DateTime<chrono::Utc>,
    },
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    timeout_secs: u64,
    failure_count: u32,
    success_count: u32,
    state: CircuitState,
}

impl CircuitBreaker {
    #[must_use]
    pub const fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            success_threshold: 1,
            timeout_secs: timeout.as_secs(),
            failure_count: 0,
            success_count: 0,
            state: CircuitState::Closed,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match &self.state {
            CircuitState::Closed => true,
            CircuitState::Open { opened_at } => {
                let elapsed = chrono::Utc::now().signed_duration_since(*opened_at);
                if elapsed.num_seconds() as u64 >= self.timeout_secs {
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub const fn record_success(&mut self) {
        match &self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        match &self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open {
                        opened_at: chrono::Utc::now(),
                    };
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open {
                    opened_at: chrono::Utc::now(),
                };
            }
            _ => {}
        }
    }

    #[must_use]
    pub const fn state_name(&self) -> &str {
        match &self.state {
            CircuitState::Closed => "closed",
            CircuitState::Open { .. } => "open",
            CircuitState::HalfOpen => "half_open",
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100));
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));

        let delay = policy.delay_for_attempt(0);
        assert_eq!(delay, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::new(5, Duration::from_millis(100));
        let d0 = policy.delay_for_attempt(0);
        let d1 = policy.delay_for_attempt(1);
        let d2 = policy.delay_for_attempt(2);

        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn test_retry_policy_max_delay_capped() {
        let policy =
            RetryPolicy::new(10, Duration::from_millis(100)).with_max_delay(Duration::from_secs(1));
        let d10 = policy.delay_for_attempt(10);

        assert!(d10 <= Duration::from_secs(1));
    }

    #[test]
    fn test_retry_policy_custom_multiplier() {
        let policy = RetryPolicy::new(5, Duration::from_millis(100)).with_backoff_multiplier(3.0);
        let d1 = policy.delay_for_attempt(1);

        assert_eq!(d1, Duration::from_millis(300));
    }

    #[test]
    fn test_retry_policy_max_retries() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100));
        assert_eq!(policy.max_retries(), 3);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(1));
        assert!(cb.can_execute());

        cb.record_failure();
        assert!(cb.can_execute());

        cb.record_failure();
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));
        cb.record_failure();
        cb.record_failure();
        assert!(cb.can_execute());

        cb.record_success();
        assert_eq!(cb.state_name(), "closed");
    }

    #[test]
    fn test_circuit_breaker_half_open_recovery() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(1));
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.can_execute());

        std::thread::sleep(Duration::from_millis(1100));
        assert!(cb.can_execute());
        assert_eq!(cb.state_name(), "half_open");

        cb.record_success();
        assert_eq!(cb.state_name(), "closed");
    }

    #[test]
    fn test_circuit_breaker_half_open_failure_reopens() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure();
        cb.record_failure();

        std::thread::sleep(Duration::from_millis(20));
        assert!(cb.can_execute());

        cb.record_failure();
        assert_eq!(cb.state_name(), "open");
    }

    #[test]
    fn test_circuit_breaker_default() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.state_name(), "closed");
    }
}
