use hardening::health::{HealthChecker, HealthStatus};
use hardening::metrics::Metrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: String,
    pub components: HashMap<String, ComponentHealth>,
    pub uptime_seconds: u64,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsReport {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
}

fn metrics() -> &'static Mutex<Metrics> {
    static METRICS: OnceLock<Mutex<Metrics>> = OnceLock::new();
    METRICS.get_or_init(|| Mutex::new(Metrics::new()))
}

pub fn record_agent_execution() {
    if let Ok(mut m) = metrics().lock() {
        m.increment_counter("agent_executions", 1);
    }
}

pub fn record_document_indexed() {
    if let Ok(mut m) = metrics().lock() {
        m.increment_counter("documents_indexed", 1);
    }
}

pub fn record_cache_hit() {
    if let Ok(mut m) = metrics().lock() {
        m.increment_counter("cache_hits", 1);
    }
}

pub fn record_cache_miss() {
    if let Ok(mut m) = metrics().lock() {
        m.increment_counter("cache_misses", 1);
    }
}

#[tauri::command]
pub fn get_health() -> HealthReport {
    // Delegate to the hardening health checker (single source of truth).
    let mut checker = HealthChecker::new();
    checker.add_provider_check("providers");
    let report = checker.check_health();

    let components = report
        .components
        .into_iter()
        .map(|(name, c)| {
            (
                name.clone(),
                ComponentHealth {
                    name,
                    status: match c.status {
                        HealthStatus::Healthy => "healthy",
                        HealthStatus::Degraded => "degraded",
                        HealthStatus::Unhealthy => "unhealthy",
                    }
                    .to_string(),
                    message: c.message,
                },
            )
        })
        .collect();

    let status = match report.status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
    };

    HealthReport {
        status: status.to_string(),
        components,
        uptime_seconds: report.uptime_seconds,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn get_metrics() -> MetricsReport {
    let m = metrics().lock().unwrap();

    let hits = m.get_counter("cache_hits");
    let misses = m.get_counter("cache_misses");
    let total = hits + misses;
    let hit_rate = if total > 0 {
        hits as f64 / total as f64
    } else {
        0.0
    };

    let mut counters = HashMap::new();
    counters.insert(
        "agent_executions".to_string(),
        m.get_counter("agent_executions"),
    );
    counters.insert(
        "documents_indexed".to_string(),
        m.get_counter("documents_indexed"),
    );
    counters.insert("cache_hits".to_string(), hits);
    counters.insert("cache_misses".to_string(), misses);

    let mut gauges = HashMap::new();
    gauges.insert("cache_hit_rate".to_string(), hit_rate);
    gauges.insert(
        "active_executions".to_string(),
        m.get_gauge("active_executions"),
    );

    MetricsReport { counters, gauges }
}
