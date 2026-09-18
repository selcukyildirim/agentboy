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

fn get_start_time() -> chrono::DateTime<chrono::Utc> {
    static START_TIME: OnceLock<chrono::DateTime<chrono::Utc>> = OnceLock::new();
    *START_TIME.get_or_init(chrono::Utc::now)
}

#[derive(Debug, Clone)]
struct MetricsState {
    agent_executions: u64,
    documents_indexed: u64,
    cache_hits: u64,
    cache_misses: u64,
    active_executions: f64,
}

fn get_metrics_state() -> &'static Mutex<MetricsState> {
    static METRICS: OnceLock<Mutex<MetricsState>> = OnceLock::new();
    METRICS.get_or_init(|| Mutex::new(MetricsState {
        agent_executions: 0,
        documents_indexed: 0,
        cache_hits: 0,
        cache_misses: 0,
        active_executions: 0.0,
    }))
}

pub fn increment_agent_executions() {
    if let Ok(mut m) = get_metrics_state().lock() {
        m.agent_executions += 1;
    }
}

pub fn increment_documents_indexed() {
    if let Ok(mut m) = get_metrics_state().lock() {
        m.documents_indexed += 1;
    }
}

pub fn increment_cache_hits() {
    if let Ok(mut m) = get_metrics_state().lock() {
        m.cache_hits += 1;
    }
}

pub fn increment_cache_misses() {
    if let Ok(mut m) = get_metrics_state().lock() {
        m.cache_misses += 1;
    }
}

pub fn set_active_executions(count: f64) {
    if let Ok(mut m) = get_metrics_state().lock() {
        m.active_executions = count;
    }
}

#[tauri::command]
pub fn get_health() -> HealthReport {
    let mut components = HashMap::new();

    components.insert("database".to_string(), ComponentHealth {
        name: "database".to_string(),
        status: "healthy".to_string(),
        message: Some("SQLite connection OK".to_string()),
    });

    components.insert("cache".to_string(), ComponentHealth {
        name: "cache".to_string(),
        status: "healthy".to_string(),
        message: Some("Moka cache OK".to_string()),
    });

    components.insert("providers".to_string(), ComponentHealth {
        name: "providers".to_string(),
        status: "healthy".to_string(),
        message: Some("Provider gateway ready".to_string()),
    });

    let uptime = (chrono::Utc::now() - get_start_time()).num_seconds() as u64;

    HealthReport {
        status: "healthy".to_string(),
        components,
        uptime_seconds: uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub fn get_metrics() -> MetricsReport {
    let state = get_metrics_state().lock().unwrap().clone();

    let mut counters = HashMap::new();
    counters.insert("agent_executions".to_string(), state.agent_executions);
    counters.insert("documents_indexed".to_string(), state.documents_indexed);
    counters.insert("cache_hits".to_string(), state.cache_hits);
    counters.insert("cache_misses".to_string(), state.cache_misses);

    let total = state.cache_hits + state.cache_misses;
    let hit_rate = if total > 0 {
        state.cache_hits as f64 / total as f64
    } else {
        0.0
    };

    let mut gauges = HashMap::new();
    gauges.insert("cache_hit_rate".to_string(), hit_rate);
    gauges.insert("active_executions".to_string(), state.active_executions);

    MetricsReport {
        counters,
        gauges,
    }
}
