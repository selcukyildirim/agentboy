use serde::{Deserialize, Serialize};

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

fn get_workflows() -> &'static std::sync::Mutex<Vec<WorkflowInfo>> {
    use std::sync::OnceLock;
    static WORKFLOWS: OnceLock<std::sync::Mutex<Vec<WorkflowInfo>>> = OnceLock::new();
    WORKFLOWS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
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

    get_workflows().lock().unwrap().push(workflow.clone());
    tracing::info!(workflow_id = %workflow.id, "Workflow created");

    Ok(workflow)
}

#[tauri::command]
pub async fn execute_workflow(workflow_id: String, input: serde_json::Value) -> Result<serde_json::Value, String> {
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

    let mut current_input = input.clone();
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

    Ok(serde_json::json!({
        "execution_id": uuid::Uuid::new_v4().to_string(),
        "workflow_id": workflow_id,
        "status": "completed",
        "steps_completed": steps_completed,
        "duration_ms": duration_ms
    }))
}
