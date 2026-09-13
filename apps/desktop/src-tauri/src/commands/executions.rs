use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub step_type: String,
    pub description: String,
    pub status: String,
    pub duration_ms: u64,
}

static mut EXECUTIONS: Option<Vec<Execution>> = None;

fn get_executions() -> &'static mut Vec<Execution> {
    unsafe {
        if EXECUTIONS.is_none() {
            EXECUTIONS = Some(Vec::new());
        }
        EXECUTIONS.as_mut().unwrap()
    }
}

#[tauri::command]
pub fn list_executions() -> Vec<Execution> {
    get_executions().clone()
}

#[tauri::command]
pub fn get_execution(execution_id: String) -> Result<Execution, String> {
    let execs = get_executions();
    execs.iter()
        .find(|e| e.id == execution_id)
        .cloned()
        .ok_or_else(|| format!("Execution {} not found", execution_id))
}

#[tauri::command]
pub fn save_execution(execution: Execution) -> Result<Execution, String> {
    let saved = execution.clone();
    get_executions().push(saved.clone());
    Ok(saved)
}

#[tauri::command]
pub fn get_execution_steps(execution_id: String) -> Result<Vec<ExecutionStep>, String> {
    let _ = execution_id;

    Ok(vec![
        ExecutionStep {
            step_number: 1,
            step_type: "Planning".to_string(),
            description: "Analyze input data".to_string(),
            status: "completed".to_string(),
            duration_ms: 150,
        },
        ExecutionStep {
            step_number: 2,
            step_type: "ToolCall".to_string(),
            description: "Execute analysis".to_string(),
            status: "completed".to_string(),
            duration_ms: 800,
        },
        ExecutionStep {
            step_number: 3,
            step_type: "Validation".to_string(),
            description: "Validate results".to_string(),
            status: "completed".to_string(),
            duration_ms: 100,
        },
    ])
}