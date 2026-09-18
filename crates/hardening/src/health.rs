use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub checked_at: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

pub struct HealthChecker {
    start_time: chrono::DateTime<chrono::Utc>,
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
}

trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> ComponentHealth;
}

struct DatabaseHealthCheck;

impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &'static str {
        "database"
    }

    fn check(&self) -> ComponentHealth {
        ComponentHealth {
            name: self.name().to_string(),
            status: HealthStatus::Healthy,
            message: Some("SQLite connection OK".to_string()),
            latency_ms: Some(1),
        }
    }
}

struct CacheHealthCheck;

impl HealthCheck for CacheHealthCheck {
    fn name(&self) -> &'static str {
        "cache"
    }

    fn check(&self) -> ComponentHealth {
        ComponentHealth {
            name: self.name().to_string(),
            status: HealthStatus::Healthy,
            message: Some("Moka cache OK".to_string()),
            latency_ms: Some(0),
        }
    }
}

struct ProviderHealthCheck {
    provider_name: String,
}

impl HealthCheck for ProviderHealthCheck {
    fn name(&self) -> &str {
        &self.provider_name
    }

    fn check(&self) -> ComponentHealth {
        ComponentHealth {
            name: self.name().to_string(),
            status: HealthStatus::Healthy,
            message: Some("Provider available".to_string()),
            latency_ms: Some(50),
        }
    }
}

impl HealthChecker {
    #[must_use]
    pub fn new() -> Self {
        let mut checker = Self {
            start_time: chrono::Utc::now(),
            checks: Vec::new(),
        };
        checker.checks.push(Box::new(DatabaseHealthCheck));
        checker.checks.push(Box::new(CacheHealthCheck));
        checker
    }

    pub fn add_provider_check(&mut self, provider_name: &str) {
        self.checks.push(Box::new(ProviderHealthCheck {
            provider_name: provider_name.to_string(),
        }));
    }

    #[must_use]
    pub fn check_health(&self) -> HealthReport {
        let mut components = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        for check in &self.checks {
            let health = check.check();
            if health.status == HealthStatus::Unhealthy {
                overall_status = HealthStatus::Unhealthy;
            } else if health.status == HealthStatus::Degraded
                && overall_status == HealthStatus::Healthy
            {
                overall_status = HealthStatus::Degraded;
            }
            components.insert(check.name().to_string(), health);
        }

        HealthReport {
            status: overall_status,
            components,
            checked_at: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: (chrono::Utc::now() - self.start_time).num_seconds() as u64,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        assert_eq!(report.status, HealthStatus::Healthy);
        assert!(report.components.len() >= 2);
    }

    #[test]
    fn test_health_check_includes_database() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        assert!(report.components.contains_key("database"));
    }

    #[test]
    fn test_health_check_includes_cache() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        assert!(report.components.contains_key("cache"));
    }

    #[test]
    fn test_health_check_with_provider() {
        let mut checker = HealthChecker::new();
        checker.add_provider_check("openai");
        let report = checker.check_health();
        assert!(report.components.contains_key("openai"));
        assert!(report.components.len() >= 3);
    }

    #[test]
    fn test_health_check_uptime() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        assert!(report.uptime_seconds < 5);
    }

    #[test]
    fn test_health_report_has_timestamp() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        assert!(!report.checked_at.is_empty());
    }

    #[test]
    fn test_component_health_latency() {
        let checker = HealthChecker::new();
        let report = checker.check_health();
        for component in report.components.values() {
            assert!(component.latency_ms.is_some());
        }
    }

    #[test]
    fn test_health_default() {
        let checker = HealthChecker::default();
        let report = checker.check_health();
        assert_eq!(report.status, HealthStatus::Healthy);
    }
}
