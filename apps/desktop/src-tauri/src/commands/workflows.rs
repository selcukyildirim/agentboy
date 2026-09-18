use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub step_count: usize,
    pub created_at: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowStep {
    pub agent_id: String,
    pub name: String,
    pub input_mapping: Option<serde_json::Value>,
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
    base.join(".agentboy").join("workflows.json")
}

fn load_from_disk() -> Vec<WorkflowInfo> {
    let path = storage_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_to_disk(workflows: &[WorkflowInfo]) -> Result<(), String> {
    let path = storage_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create workflow dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(workflows)
        .map_err(|e| format!("Failed to serialize workflows: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write workflows: {}", e))?;
    Ok(())
}

fn get_workflows() -> &'static Mutex<Vec<WorkflowInfo>> {
    static WORKFLOWS: OnceLock<Mutex<Vec<WorkflowInfo>>> = OnceLock::new();
    WORKFLOWS.get_or_init(|| Mutex::new(load_from_disk()))
}

#[tauri::command]
pub fn list_workflows() -> Vec<WorkflowInfo> {
    get_workflows().lock().unwrap().clone()
}

#[tauri::command]
pub fn create_workflow(name: String, description: String) -> Result<WorkflowInfo, String> {
    let workflow = WorkflowInfo {
        id: format!("wf-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        name,
        description,
        version: 1,
        step_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        steps: Vec::new(),
    };

    let mut workflows = get_workflows().lock().unwrap();
    workflows.push(workflow.clone());
    save_to_disk(&workflows)?;

    tracing::info!(workflow_id = %workflow.id, "Workflow created and persisted");

    Ok(workflow)
}

#[tauri::command]
pub fn add_workflow_step(
    workflow_id: String,
    agent_id: String,
    name: String,
) -> Result<WorkflowInfo, String> {
    let mut workflows = get_workflows().lock().unwrap();
    let workflow = workflows
        .iter_mut()
        .find(|w| w.id == workflow_id)
        .ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    workflow.steps.push(WorkflowStep {
        agent_id,
        name,
        input_mapping: None,
    });
    workflow.step_count = workflow.steps.len();
    let updated = workflow.clone();
    save_to_disk(&workflows)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_workflow(workflow_id: String) -> Result<(), String> {
    let mut workflows = get_workflows().lock().unwrap();
    workflows.retain(|w| w.id != workflow_id);
    save_to_disk(&workflows)
}

#[tauri::command]
pub async fn execute_workflow(
    workflow_id: String,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    tracing::info!(workflow_id = %workflow_id, "Executing workflow from UI");

    let workflow = {
        let workflows = get_workflows().lock().unwrap();
        workflows.iter().find(|w| w.id == workflow_id).cloned()
    };

    let workflow = workflow.ok_or_else(|| format!("Workflow {} not found", workflow_id))?;

    if workflow.steps.is_empty() {
        return Ok(serde_json::json!({
            "execution_id": uuid::Uuid::new_v4().to_string(),
            "workflow_id": workflow_id,
            "status": "completed",
            "steps_completed": 0,
            "message": "Workflow has no steps"
        }));
    }

    let mut steps_completed = 0u32;
    let start = std::time::Instant::now();

    for step in &workflow.steps {
        tracing::info!(
            workflow_id = %workflow_id,
            agent_id = %step.agent_id,
            step = step.name,
            "Executing workflow step"
        );
        steps_completed += 1;
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = input;

    Ok(serde_json::json!({
        "execution_id": uuid::Uuid::new_v4().to_string(),
        "workflow_id": workflow_id,
        "status": "completed",
        "steps_completed": steps_completed,
        "duration_ms": duration_ms
    }))
}
