use agent_common::error::{ApiError, ErrorCode};
use agent_runtime::agent::AgentExecutor;
use agent_runtime::context::{AgentConfig, AgentContext, DefaultAgentContext, LlmProvider, LlmUsage};
use agent_runtime::manifest::InputField;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::commands::{executions, providers};
use crate::cost;
use crate::llm_factory::{self, ProviderSpec};

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub department: String,
    pub description: String,
    pub tier: String,
    pub version: String,
    pub skills: Vec<String>,
    pub input_schema: Vec<InputField>,
}

fn to_info(agent: &dyn agent_runtime::agent::Agent) -> AgentInfo {
    let m = agent.manifest();
    AgentInfo {
        id: m.id,
        name: m.name,
        department: m.department,
        description: m.description,
        tier: format!("{:?}", m.tier),
        version: m.version,
        skills: m.skills,
        input_schema: m.input_schema,
    }
}

#[tauri::command]
pub async fn list_agents(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AgentInfo>, ApiError> {
    let registry = state.registry.read().await;
    Ok(registry
        .list()
        .iter()
        .filter_map(|id| registry.get(id).map(to_info))
        .collect())
}

#[tauri::command]
pub async fn get_agent_manifest(
    state: tauri::State<'_, AppState>,
    agent_id: String,
) -> Result<AgentInfo, ApiError> {
    let registry = state.registry.read().await;
    registry
        .get(&agent_id)
        .map(to_info)
        .ok_or_else(|| ApiError::new(ErrorCode::NOT_FOUND, format!("Agent {agent_id} not found")))
}

async fn resolve_provider_spec() -> Result<ProviderSpec, ApiError> {
    let (provider, base_url, model) = providers::active_provider_meta().ok_or_else(|| {
        ApiError::new(
            ErrorCode::PROVIDER_UNAVAILABLE,
            "No AI provider configured. Add one in Settings → Providers.",
        )
    })?;
    let api_key = providers::read_secret(&provider).await;
    let model = model.unwrap_or_else(|| ProviderSpec::default_model_for(&provider));
    Ok(ProviderSpec {
        provider,
        api_key,
        base_url,
        model,
    })
}

#[tauri::command]
pub async fn execute_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    input: serde_json::Value,
    offline: Option<bool>,
) -> Result<serde_json::Value, ApiError> {
    tracing::info!(agent_id = %agent_id, "Executing agent from UI");

    let offline = offline.unwrap_or(false);
    let execution_id = uuid::Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now();
    let start = std::time::Instant::now();

    let mut steps: Vec<executions::ExecutionStep> = Vec::new();

    let result = {
        let registry = state.registry.read().await;
        let agent = registry.get(&agent_id).ok_or_else(|| {
            ApiError::new(ErrorCode::NOT_FOUND, format!("Agent {agent_id} not found"))
        })?;

        let manifest = agent.manifest();
        steps.push(executions::ExecutionStep {
            step_number: 1,
            step_type: "planning".to_string(),
            description: format!("Resolve agent '{}' v{}", manifest.id, manifest.version),
            status: "completed".to_string(),
            duration_ms: 0,
        });

        let mut config = AgentConfig::default();
        config.offline = offline;

        let mut usage = LlmUsage::default();
        let mut model_name: Option<String> = None;

        let outcome = if manifest.permissions.network_llm && !offline {
            let spec = resolve_provider_spec().await?;
            model_name = Some(spec.model.clone());
            let provider = llm_factory::build(&spec).map_err(ApiError::from)?;
            let ctx = DefaultAgentContext::new(Box::new(DynLlmProvider(provider)), config);
            let outcome = AgentExecutor::run(agent, input.clone(), Some(&ctx)).await;
            usage = ctx.total_usage();
            outcome
        } else if manifest.permissions.network_llm && offline {
            steps.push(executions::ExecutionStep {
                step_number: 2,
                step_type: "note".to_string(),
                description: "Offline mode: LLM analysis skipped".to_string(),
                status: "completed".to_string(),
                duration_ms: 0,
            });
            let ctx = DefaultAgentContext::new(Box::new(NoOpLlmProvider), config);
            AgentExecutor::run(agent, input.clone(), Some(&ctx)).await
        } else {
            AgentExecutor::run(agent, input.clone(), None).await
        };

        (outcome, usage, model_name)
    };

    let (outcome, usage, model_name) = result;
    let duration_ms = start.elapsed().as_millis() as u64;
    let cost_usd = model_name
        .as_deref()
        .and_then(|m| cost::estimate_cost_usd(m, &usage));

    steps.push(executions::ExecutionStep {
        step_number: steps.len() as u32 + 1,
        step_type: "execution".to_string(),
        description: format!("Execute agent '{agent_id}'"),
        status: if outcome.is_ok() { "completed" } else { "failed" }.to_string(),
        duration_ms,
    });

    let status = if outcome.is_ok() { "completed" } else { "failed" };
    let execution = executions::Execution {
        id: execution_id.clone(),
        agent_id: agent_id.clone(),
        status: status.to_string(),
        input,
        output: outcome.as_ref().ok().cloned(),
        started_at: started_at.to_rfc3339(),
        completed_at: Some(chrono::Utc::now().to_rfc3339()),
        duration_ms: Some(duration_ms),
        model: model_name.clone(),
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        cost_usd,
        error: outcome.as_ref().err().map(|e| e.to_string()),
        steps: steps.clone(),
    };
    executions::append_execution(execution);

    match outcome {
        Ok(output) => Ok(serde_json::json!({
            "execution_id": execution_id,
            "agent_id": agent_id,
            "status": "completed",
            "duration_ms": duration_ms,
            "model": model_name,
            "usage": {
                "input_tokens": usage.prompt_tokens,
                "output_tokens": usage.completion_tokens,
                "total_tokens": usage.total_tokens,
            },
            "cost_usd": cost_usd,
            "steps": steps,
            "output": output,
        })),
        Err(e) => Err(ApiError::from(e)),
    }
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
