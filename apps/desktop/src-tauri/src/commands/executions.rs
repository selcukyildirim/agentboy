use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub step_type: String,
    pub description: String,
    pub status: String,
    pub duration_ms: u64,
}

fn get_executions() -> &'static Mutex<Vec<Execution>> {
    static EXECUTIONS: OnceLock<Mutex<Vec<Execution>>> = OnceLock::new();
    EXECUTIONS.get_or_init(|| Mutex::new(Vec::new()))
}

fn get_steps() -> &'static Mutex<std::collections::HashMap<String, Vec<ExecutionStep>>> {
    static STEPS: OnceLock<Mutex<std::collections::HashMap<String, Vec<ExecutionStep>>>> =
        OnceLock::new();
    STEPS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

#[tauri::command]
pub fn list_executions() -> Vec<Execution> {
    get_executions().lock().unwrap().clone()
}

#[tauri::command]
pub fn get_execution(execution_id: String) -> Result<Execution, String> {
    let execs = get_executions().lock().unwrap();
    execs
        .iter()
        .find(|e| e.id == execution_id)
        .cloned()
        .ok_or_else(|| format!("Execution {} not found", execution_id))
}

#[tauri::command]
pub fn save_execution(execution: Execution) -> Result<Execution, String> {
    let saved = execution.clone();
    get_executions().lock().unwrap().push(saved.clone());
    Ok(saved)
}

pub fn save_step(execution_id: &str, step: ExecutionStep) {
    get_steps()
        .lock()
        .unwrap()
        .entry(execution_id.to_string())
        .or_default()
        .push(step);
}

#[tauri::command]
pub fn get_execution_steps(execution_id: String) -> Result<Vec<ExecutionStep>, String> {
    let steps = get_steps().lock().unwrap();
    Ok(steps.get(&execution_id).cloned().unwrap_or_default())
}
