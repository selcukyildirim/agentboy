use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const MAX_ENTRIES: usize = 500;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub step_type: String,
    pub description: String,
    pub status: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Execution {
    pub id: String,
    pub agent_id: String,
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub steps: Vec<ExecutionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub count: u64,
    pub success: u64,
    pub failed: u64,
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub avg_cost_usd: f64,
    pub daily: Vec<DailyStat>,
}

fn storage_path() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
    } else {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
    };
    base.join(".agentboy").join("executions.json")
}

fn load_from_disk() -> Vec<Execution> {
    match std::fs::read_to_string(storage_path()) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_to_disk(executions: &[Execution]) -> Result<(), String> {
    let path = storage_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    let slice = if executions.len() > MAX_ENTRIES {
        &executions[executions.len() - MAX_ENTRIES..]
    } else {
        executions
    };
    let json = serde_json::to_string_pretty(slice).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {e}"))?;
    Ok(())
}

fn get_executions() -> &'static Mutex<Vec<Execution>> {
    static EXECUTIONS: OnceLock<Mutex<Vec<Execution>>> = OnceLock::new();
    EXECUTIONS.get_or_init(|| Mutex::new(load_from_disk()))
}

pub fn append_execution(execution: Execution) {
    let mut execs = get_executions().lock().unwrap();
    execs.push(execution);
    if let Err(e) = save_to_disk(&execs) {
        tracing::warn!(error = %e, "Failed to persist execution");
    }
}

#[tauri::command]
pub fn list_executions() -> Vec<Execution> {
    get_executions().lock().unwrap().iter().rev().cloned().collect()
}

#[tauri::command]
pub fn get_execution(execution_id: String) -> Result<Execution, String> {
    get_executions()
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.id == execution_id)
        .cloned()
        .ok_or_else(|| format!("Execution {execution_id} not found"))
}

#[tauri::command]
pub fn get_execution_steps(execution_id: String) -> Result<Vec<ExecutionStep>, String> {
    get_executions()
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.id == execution_id)
        .map(|e| e.steps.clone())
        .ok_or_else(|| format!("Execution {execution_id} not found"))
}

#[tauri::command]
pub fn delete_execution(execution_id: String) -> Result<(), String> {
    let mut execs = get_executions().lock().unwrap();
    execs.retain(|e| e.id != execution_id);
    save_to_disk(&execs)
}

#[tauri::command]
pub fn get_execution_stats(days: Option<u32>) -> ExecutionStats {
    let days = days.unwrap_or(7) as i64;
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days);

    let execs = get_executions().lock().unwrap();
    let filtered: Vec<&Execution> = execs
        .iter()
        .filter(|e| {
            chrono::DateTime::parse_from_rfc3339(&e.started_at)
                .map(|dt| dt.with_timezone(&chrono::Utc) >= cutoff)
                .unwrap_or(true)
        })
        .collect();

    let total = filtered.len() as u64;
    let success = filtered.iter().filter(|e| e.status == "completed").count() as u64;
    let failed = filtered.iter().filter(|e| e.status == "failed").count() as u64;
    let success_rate = if total > 0 {
        success as f64 / total as f64
    } else {
        0.0
    };

    let mut durations: Vec<f64> = filtered
        .iter()
        .filter_map(|e| e.duration_ms.map(|d| d as f64))
        .collect();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let avg_duration_ms = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<f64>() / durations.len() as f64
    };
    let percentile = |p: f64| -> f64 {
        if durations.is_empty() {
            return 0.0;
        }
        let idx = ((durations.len() - 1) as f64 * p).round() as usize;
        durations[idx]
    };

    let total_input_tokens: u64 = filtered.iter().map(|e| e.input_tokens as u64).sum();
    let total_output_tokens: u64 = filtered.iter().map(|e| e.output_tokens as u64).sum();
    let total_cost_usd: f64 = filtered.iter().filter_map(|e| e.cost_usd).sum();
    let avg_cost_usd = if total > 0 {
        total_cost_usd / total as f64
    } else {
        0.0
    };

    let mut by_day: std::collections::BTreeMap<String, DailyStat> =
        std::collections::BTreeMap::new();
    for e in &filtered {
        let date = e.started_at.get(0..10).unwrap_or("").to_string();
        let entry = by_day.entry(date.clone()).or_insert(DailyStat {
            date,
            count: 0,
            success: 0,
            failed: 0,
            tokens: 0,
            cost_usd: 0.0,
        });
        entry.count += 1;
        if e.status == "completed" {
            entry.success += 1;
        } else if e.status == "failed" {
            entry.failed += 1;
        }
        entry.tokens += (e.input_tokens + e.output_tokens) as u64;
        entry.cost_usd += e.cost_usd.unwrap_or(0.0);
    }

    ExecutionStats {
        total,
        success,
        failed,
        success_rate,
        avg_duration_ms,
        p50_duration_ms: percentile(0.50),
        p95_duration_ms: percentile(0.95),
        total_input_tokens,
        total_output_tokens,
        total_cost_usd,
        avg_cost_usd,
        daily: by_day.into_values().collect(),
    }
}
