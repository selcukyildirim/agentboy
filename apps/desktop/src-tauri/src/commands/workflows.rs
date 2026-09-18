use agent_common::error::AppResult;
use agent_runtime::context::{AgentConfig, DefaultAgentContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::RwLock;
use workflow_engine::runner::{StepExecutor, WorkflowRunner};
use workflow_engine::workflow::{Trigger, Workflow, WorkflowStep};

use crate::app_state::AppState;
use crate::commands::agents::{resolve_provider_spec, DynLlmProvider};
use crate::llm_factory;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowStepInfo {
    pub agent_id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub step_count: usize,
    pub created_at: String,
    pub steps: Vec<WorkflowStepInfo>,
}

fn to_info(wf: &Workflow) -> WorkflowInfo {
    WorkflowInfo {
        id: wf.id.clone(),
        name: wf.name.clone(),
        description: wf.description.clone(),
        version: wf.version,
        step_count: wf.steps.len(),
        created_at: wf.created_at.clone(),
        steps: wf
            .steps
            .iter()
            .map(|s| WorkflowStepInfo {
                agent_id: s.skill_id.clone(),
                name: s.name.clone(),
            })
            .collect(),
    }
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

fn load_from_disk() -> Vec<Workflow> {
    std::fs::read_to_string(storage_path())
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_to_disk(workflows: &[Workflow]) -> Result<(), String> {
    let path = storage_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    let json = serde_json::to_string_pretty(workflows).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {e}"))?;
    Ok(())
}

fn store() -> &'static Mutex<Vec<Workflow>> {
    static W: OnceLock<Mutex<Vec<Workflow>>> = OnceLock::new();
    W.get_or_init(|| Mutex::new(load_from_disk()))
}

#[tauri::command]
pub fn list_workflows() -> Vec<WorkflowInfo> {
    agent_common::sync::lock(store())
        .iter()
        .map(to_info)
        .collect()
}

#[tauri::command]
pub fn create_workflow(name: String, description: String) -> Result<WorkflowInfo, String> {
    let mut wf = Workflow::new(
        &format!("wf-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        &name,
    );
    wf.description = description;
    wf.trigger = Trigger::Manual;

    let mut workflows = agent_common::sync::lock(store());
    workflows.push(wf.clone());
    save_to_disk(&workflows)?;
    tracing::info!(workflow_id = %wf.id, "Workflow created");
    Ok(to_info(&wf))
}

#[tauri::command]
pub fn add_workflow_step(
    workflow_id: String,
    agent_id: String,
    name: String,
) -> Result<WorkflowInfo, String> {
    let mut workflows = agent_common::sync::lock(store());
    let wf = workflows
        .iter_mut()
        .find(|w| w.id == workflow_id)
        .ok_or_else(|| format!("Workflow {workflow_id} not found"))?;

    let step_id = format!("step-{}", wf.steps.len() + 1);
    wf.steps.push(WorkflowStep {
        id: step_id,
        skill_id: agent_id,
        name,
        input_mapping: None,
        output_mapping: None,
        depends_on: vec![],
        timeout_seconds: None,
        retry_count: None,
        rollback: None,
    });
    wf.updated_at = chrono::Utc::now().to_rfc3339();
    let updated = wf.clone();
    save_to_disk(&workflows)?;
    Ok(to_info(&updated))
}

#[tauri::command]
pub fn delete_workflow(workflow_id: String) -> Result<(), String> {
    let mut workflows = agent_common::sync::lock(store());
    workflows.retain(|w| w.id != workflow_id);
    save_to_disk(&workflows)
}

/// Runs each workflow step's agent through the orchestrator.
struct DesktopStepExecutor {
    registry: Arc<RwLock<agent_runtime::registry::AgentRegistry>>,
    orchestrator: Arc<orchestrator::Orchestrator>,
    skills: Arc<skill_sdk::SkillRegistry>,
}

#[async_trait]
impl StepExecutor for DesktopStepExecutor {
    async fn execute_step(
        &self,
        skill_id: &str,
        input: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        let offline = std::env::var("AGENTBOY_OFFLINE").is_ok();
        let registry = self.registry.read().await;
        let agent = registry
            .get(skill_id)
            .ok_or_else(|| agent_common::error::AppError::NotFound(format!("Agent {skill_id}")))?;

        let manifest = agent.manifest();
        let config = AgentConfig {
            offline,
            ..Default::default()
        };

        let outcome = if manifest.permissions.network_llm && !offline {
            let spec = resolve_provider_spec().await.map_err(|e| {
                agent_common::error::AppError::Provider {
                    provider: "none".into(),
                    message: e.message,
                }
            })?;
            let provider = llm_factory::build(&spec)?;
            let ctx = DefaultAgentContext::new(Box::new(DynLlmProvider(provider)), config);
            self.orchestrator
                .execute(&self.skills, agent, input, Some(&ctx), None)
                .await
                .map(|(_, out)| out)
        } else {
            self.orchestrator
                .execute(&self.skills, agent, input, None, None)
                .await
                .map(|(_, out)| out)
        };
        outcome
    }
}

#[tauri::command]
pub async fn execute_workflow(
    state: tauri::State<'_, AppState>,
    workflow_id: String,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    tracing::info!(workflow_id = %workflow_id, "Executing workflow from UI");

    let workflow = {
        let workflows = agent_common::sync::lock(store());
        workflows.iter().find(|w| w.id == workflow_id).cloned()
    }
    .ok_or_else(|| format!("Workflow {workflow_id} not found"))?;

    if workflow.steps.is_empty() {
        return Ok(serde_json::json!({
            "execution_id": uuid::Uuid::new_v4().to_string(),
            "workflow_id": workflow_id,
            "status": "completed",
            "steps_completed": 0,
        }));
    }

    let executor = DesktopStepExecutor {
        registry: state.registry.clone(),
        orchestrator: state.orchestrator.clone(),
        skills: state.skills.clone(),
    };

    let runner = WorkflowRunner::new();
    let execution = runner
        .run_with(&executor, &workflow, input)
        .await
        .map_err(|e| e.to_string())?;

    // Persist run to local workflow history (workflow-engine).
    let history = workflow_engine::WorkflowHistory::new(state.pool.clone());
    if let Err(e) = history.save_execution(&execution).await {
        tracing::warn!(error = %e, "Failed to save workflow execution");
    }

    serde_json::to_value(&execution).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_workflow_runs(
    state: tauri::State<'_, AppState>,
    workflow_id: String,
    limit: Option<u32>,
) -> Result<serde_json::Value, String> {
    let history = workflow_engine::WorkflowHistory::new(state.pool.clone());
    let runs = history
        .list_by_workflow(&workflow_id, limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())?;
    serde_json::to_value(runs).map_err(|e| e.to_string())
}
