use crate::context::ExecutionContext;
use agent_common::error::{AppError, AppResult};
use agent_common::types::{ExecutionId, ExecutionStatus};
use agent_runtime::agent::{Agent, AgentExecutor};
use agent_runtime::context::AgentContext;
use agent_runtime::state::{ExecutionState, ExecutionStep};
use audit_core::store::SqliteAuditStore;
use skill_sdk::registry::SkillRegistry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum number of completed executions retained in memory. Older terminal
/// executions are evicted to keep memory bounded over long sessions.
const MAX_RETAINED_EXECUTIONS: usize = 1000;

pub struct Orchestrator {
    executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    steps: Arc<RwLock<HashMap<String, Vec<ExecutionStep>>>>,
    states: Arc<RwLock<HashMap<String, ExecutionState>>>,
    cancelled: Arc<RwLock<HashSet<String>>>,
    order: Arc<RwLock<VecDeque<String>>>,
}

impl Orchestrator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            steps: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            cancelled: Arc::new(RwLock::new(HashSet::new())),
            order: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    async fn evict_old(&self) {
        let mut order = self.order.write().await;
        while order.len() > MAX_RETAINED_EXECUTIONS {
            if let Some(id) = order.pop_front() {
                let terminal = {
                    let states = self.states.read().await;
                    states.get(&id).map(|s| s.is_terminal()).unwrap_or(true)
                };
                if terminal {
                    self.executions.write().await.remove(&id);
                    self.steps.write().await.remove(&id);
                    self.states.write().await.remove(&id);
                } else {
                    // Keep non-terminal at the back; avoid an infinite loop.
                    order.push_back(id);
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub async fn start_execution(
        &self,
        agent_id: &str,
        agent_version: &str,
        _input: serde_json::Value,
        max_steps: u32,
        timeout_seconds: u64,
    ) -> AppResult<ExecutionId> {
        let execution_id = Uuid::new_v4();

        let context = ExecutionContext::new(
            execution_id,
            agent_id.to_string(),
            agent_version.to_string(),
            max_steps,
            timeout_seconds,
        );

        let mut executions = self.executions.write().await;
        executions.insert(execution_id.to_string(), context);

        let mut states = self.states.write().await;
        states.insert(execution_id.to_string(), ExecutionState::Pending);

        let mut steps = self.steps.write().await;
        steps.insert(execution_id.to_string(), Vec::new());

        {
            let mut order = self.order.write().await;
            order.push_back(execution_id.to_string());
        }
        self.evict_old().await;

        tracing::info!(
            execution_id = %execution_id,
            agent_id = %agent_id,
            "Starting agent execution"
        );

        Ok(execution_id)
    }

    pub async fn transition(
        &self,
        execution_id: ExecutionId,
        new_state: ExecutionState,
    ) -> AppResult<()> {
        let id = execution_id.to_string();

        if self.cancelled.read().await.contains(&id) {
            return Err(AppError::ExecutionCancelled {
                reason: "Execution was cancelled".to_string(),
            });
        }

        let mut states = self.states.write().await;
        let current = states.get(&id).cloned().unwrap_or(ExecutionState::Pending);

        if !current.can_transition_to(&new_state) {
            return Err(AppError::InvalidStateTransition {
                from: format!("{current:?}"),
                to: format!("{new_state:?}"),
            });
        }

        states.insert(id.clone(), new_state.clone());

        Ok(())
    }

    pub async fn cancel(&self, execution_id: ExecutionId, reason: &str) -> AppResult<()> {
        let id = execution_id.to_string();
        let mut cancelled = self.cancelled.write().await;
        cancelled.insert(id.clone());

        let mut states = self.states.write().await;
        states.insert(
            id,
            ExecutionState::Cancelled {
                reason: reason.to_string(),
            },
        );

        Ok(())
    }

    pub async fn get_state(&self, execution_id: ExecutionId) -> AppResult<ExecutionState> {
        let states = self.states.read().await;
        states
            .get(&execution_id.to_string())
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Execution {execution_id} not found")))
    }

    pub async fn get_steps(&self, execution_id: ExecutionId) -> AppResult<Vec<ExecutionStep>> {
        let steps = self.steps.read().await;
        Ok(steps
            .get(&execution_id.to_string())
            .cloned()
            .unwrap_or_default())
    }

    pub async fn status(&self, execution_id: ExecutionId) -> ExecutionStatus {
        let states = self.states.read().await;
        match states.get(&execution_id.to_string()) {
            Some(ExecutionState::Completed) => ExecutionStatus::Completed,
            Some(ExecutionState::Failed { .. }) => ExecutionStatus::Failed,
            Some(ExecutionState::Cancelled { .. }) => ExecutionStatus::Cancelled,
            Some(_) => ExecutionStatus::Running,
            None => ExecutionStatus::Pending,
        }
    }

    /// Validate skills, run the agent through the state machine, and record
    /// execution steps. This is the single execution entry point used by the
    /// desktop and workflows.
    pub async fn execute(
        &self,
        skill_registry: &SkillRegistry,
        agent: &dyn Agent,
        input: serde_json::Value,
        ctx: Option<&dyn AgentContext>,
        audit_store: Option<&SqliteAuditStore>,
    ) -> AppResult<(ExecutionId, serde_json::Value)> {
        let manifest = agent.manifest();

        if let Err(problems) = skill_registry.validate(&manifest.skills) {
            return Err(AppError::Validation(format!(
                "Agent '{}' has unresolved skills: {}",
                manifest.id,
                problems.join("; ")
            )));
        }

        // Prompt-injection defense on untrusted input before it reaches prompts.
        let mut input = input;
        let defense = rag_core::injection::InjectionDefense::new();
        let flagged = crate::injection::sanitize_untrusted_input(&defense, &mut input);
        if flagged > 0 {
            tracing::warn!(
                agent_id = %manifest.id,
                fields = flagged,
                "prompt-injection flagged in input; sanitized"
            );
        }

        let execution_id = self
            .start_execution(
                &manifest.id,
                &manifest.version,
                input.clone(),
                manifest.execution.max_steps,
                manifest.execution.timeout_seconds,
            )
            .await?;

        self.push_step(
            execution_id,
            ExecutionStep {
                step_number: 1,
                state: ExecutionState::Planning,
                input: input.clone(),
                output: None,
                started_at: chrono::Utc::now().to_rfc3339(),
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                duration_ms: Some(0),
            },
        )
        .await;

        self.transition(execution_id, ExecutionState::Planning)
            .await?;

        let outcome = AgentExecutor::run_with_audit(agent, input, ctx, audit_store).await;

        match &outcome {
            Ok(output) => {
                self.transition(execution_id, ExecutionState::Validating)
                    .await?;
                self.push_step(
                    execution_id,
                    ExecutionStep {
                        step_number: 2,
                        state: ExecutionState::Validating,
                        input: serde_json::json!({}),
                        output: Some(output.clone()),
                        started_at: chrono::Utc::now().to_rfc3339(),
                        completed_at: Some(chrono::Utc::now().to_rfc3339()),
                        duration_ms: None,
                    },
                )
                .await;
                self.transition(execution_id, ExecutionState::Completed)
                    .await?;
            }
            Err(e) => {
                self.transition(
                    execution_id,
                    ExecutionState::Failed {
                        reason: e.to_string(),
                    },
                )
                .await?;
            }
        }

        outcome.map(|output| (execution_id, output))
    }

    async fn push_step(&self, execution_id: ExecutionId, step: ExecutionStep) {
        let mut steps = self.steps.write().await;
        steps
            .entry(execution_id.to_string())
            .or_default()
            .push(step);
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_execution() {
        let orch = Orchestrator::new();
        let id = orch
            .start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60)
            .await
            .unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Pending));
    }

    #[tokio::test]
    async fn test_transition() {
        let orch = Orchestrator::new();
        let id = orch
            .start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60)
            .await
            .unwrap();
        orch.transition(id, ExecutionState::Planning).await.unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Planning));
    }

    #[tokio::test]
    async fn test_cancel() {
        let orch = Orchestrator::new();
        let id = orch
            .start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60)
            .await
            .unwrap();
        orch.cancel(id, "user requested").await.unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Cancelled { .. }));
    }
}
