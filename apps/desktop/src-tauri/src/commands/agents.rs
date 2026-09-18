use agent_runtime::agent::AgentExecutor;
use agent_runtime::context::{AgentConfig, DefaultAgentContext, LlmProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub department: String,
    pub description: String,
    pub tier: String,
}

#[tauri::command]
pub async fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    let registry = state.registry.read().await;
    Ok(registry
        .list()
        .iter()
        .filter_map(|id| {
            registry.get(id).map(|agent| {
                let m = agent.manifest();
                AgentInfo {
                    id: m.id,
                    name: m.name,
                    department: m.department,
                    description: m.description,
                    tier: format!("{:?}", m.tier),
                }
            })
        })
        .collect())
}

#[tauri::command]
pub async fn get_agent_manifest(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<AgentInfo, String> {
    let registry = state.registry.read().await;
    registry
        .get(&agent_id)
        .map(|agent| {
            let m = agent.manifest();
            AgentInfo {
                id: m.id,
                name: m.name,
                department: m.department,
                description: m.description,
                tier: format!("{:?}", m.tier),
            }
        })
        .ok_or_else(|| format!("Agent {} not found", agent_id))
}

#[tauri::command]
pub async fn execute_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    input: serde_json::Value,
) -> Result<serde_json::Value, String> {
    tracing::info!(agent_id = %agent_id, "Executing agent from UI");

    let execution_id = uuid::Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now();
    let start = std::time::Instant::now();

    let result = {
        let registry = state.registry.read().await;
        let agent = registry
            .get(&agent_id)
            .ok_or_else(|| format!("Agent {} not found", agent_id))?;

        let manifest = agent.manifest();

        crate::commands::executions::save_step(
            &execution_id,
            crate::commands::executions::ExecutionStep {
                step_number: 1,
                step_type: "Planning".to_string(),
                description: format!("Resolve agent '{}' v{}", manifest.id, manifest.version),
                status: "completed".to_string(),
                duration_ms: 0,
            },
        );

        if manifest.permissions.network_llm {
            let llm: Arc<dyn LlmProvider> = create_default_llm_provider();
            let config = AgentConfig::default();
            let ctx = DefaultAgentContext::new(Box::new(DynLlmProvider(llm)), config);
            AgentExecutor::run(agent, input.clone(), Some(&ctx)).await
        } else {
            AgentExecutor::run(agent, input.clone(), None).await
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let completed_at = chrono::Utc::now();

    crate::commands::executions::save_step(
        &execution_id,
        crate::commands::executions::ExecutionStep {
            step_number: 2,
            step_type: "Execution".to_string(),
            description: format!("Execute agent '{}'", agent_id),
            status: if result.is_ok() { "completed" } else { "failed" }.to_string(),
            duration_ms,
        },
    );

    let execution = crate::commands::executions::Execution {
        id: execution_id.clone(),
        agent_id: agent_id.clone(),
        status: if result.is_ok() { "completed" } else { "failed" }.to_string(),
        input,
        output: result.as_ref().ok().cloned(),
        started_at: started_at.to_rfc3339(),
        completed_at: Some(completed_at.to_rfc3339()),
        duration_ms: Some(duration_ms),
    };
    let _ = crate::commands::executions::save_execution(execution);

    match result {
        Ok(output) => Ok(serde_json::json!({
            "execution_id": execution_id,
            "agent_id": agent_id,
            "status": "completed",
            "duration_ms": duration_ms,
            "output": output,
        })),
        Err(e) => Err(e.to_string()),
    }
}

fn create_default_llm_provider() -> Arc<dyn LlmProvider> {
    Arc::new(NoOpLlmProvider)
}

struct NoOpLlmProvider;

#[async_trait::async_trait]
impl LlmProvider for NoOpLlmProvider {
    fn model_name(&self) -> &str {
        "none"
    }

    fn provider_id(&self) -> &str {
        "none"
    }

    async fn complete(
        &self,
        _request: agent_runtime::context::LlmCompletionRequest,
    ) -> agent_common::error::AppResult<agent_runtime::context::LlmCompletionResponse> {
        Err(agent_common::error::AppError::Provider {
            provider: "none".to_string(),
            message: "No LLM provider configured. Add a provider in Settings.".to_string(),
        })
    }
}

struct DynLlmProvider(Arc<dyn LlmProvider>);

#[async_trait::async_trait]
impl LlmProvider for DynLlmProvider {
    fn model_name(&self) -> &str {
        self.0.model_name()
    }

    fn provider_id(&self) -> &str {
        self.0.provider_id()
    }

    async fn complete(
        &self,
        request: agent_runtime::context::LlmCompletionRequest,
    ) -> agent_common::error::AppResult<agent_runtime::context::LlmCompletionResponse> {
        self.0.complete(request).await
    }
}
