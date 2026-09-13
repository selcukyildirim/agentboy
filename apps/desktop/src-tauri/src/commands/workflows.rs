use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub step_count: usize,
    pub created_at: String,
}

#[tauri::command]
pub fn list_workflows() -> Vec<WorkflowInfo> {
    vec![
        WorkflowInfo {
            id: "wf-sample".to_string(),
            name: "Sample Workflow".to_string(),
            description: "A sample workflow template".to_string(),
            version: 1,
            step_count: 3,
            created_at: chrono::Utc::now().to_rfc3339(),
        },
    ]
}

#[tauri::command]
pub fn create_workflow(name: String, description: String) -> Result<WorkflowInfo, String> {
    Ok(WorkflowInfo {
        id: format!("wf-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
        name,
        description,
        version: 1,
        step_count: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn execute_workflow(workflow_id: String, input: serde_json::Value) -> Result<serde_json::Value, String> {
    tracing::info!(workflow_id = %workflow_id, "Executing workflow from UI");

    Ok(serde_json::json!({
        "execution_id": uuid::Uuid::new_v4().to_string(),
        "workflow_id": workflow_id,
        "status": "completed",
        "steps_completed": 3,
        "duration_ms": 1500
    }))
}