use agent_common::error::{AppError, AppResult};
use agent_common::types::{ExecutionId, ExecutionStatus};
use agent_runtime::state::{ExecutionState, ExecutionStep};
use crate::context::ExecutionContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Orchestrator {
    executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
    steps: Arc<RwLock<HashMap<String, Vec<ExecutionStep>>>>,
    states: Arc<RwLock<HashMap<String, ExecutionState>>>,
    cancelled: Arc<RwLock<Vec<String>>>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
            steps: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            cancelled: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_execution(
        &self,
        agent_id: &str,
        agent_version: &str,
        input: serde_json::Value,
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
                from: format!("{:?}", current),
                to: format!("{:?}", new_state),
            });
        }

        states.insert(id.clone(), new_state.clone());

        if new_state.is_terminal() {
            if let Some(mut ctx) = self.executions.write().await.get_mut(&id) {
                ctx.step = ctx.step;
            }
        }

        Ok(())
    }

    pub async fn cancel(&self, execution_id: ExecutionId, reason: &str) -> AppResult<()> {
        let id = execution_id.to_string();
        let mut cancelled = self.cancelled.write().await;
        cancelled.push(id.clone());

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
            .ok_or_else(|| AppError::NotFound(format!("Execution {} not found", execution_id)))
    }

    pub async fn get_steps(&self, execution_id: ExecutionId) -> AppResult<Vec<ExecutionStep>> {
        let steps = self.steps.read().await;
        Ok(steps.get(&execution_id.to_string()).cloned().unwrap_or_default())
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
        let id = orch.start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60).await.unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Pending));
    }

    #[tokio::test]
    async fn test_transition() {
        let orch = Orchestrator::new();
        let id = orch.start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60).await.unwrap();
        orch.transition(id, ExecutionState::Planning).await.unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Planning));
    }

    #[tokio::test]
    async fn test_cancel() {
        let orch = Orchestrator::new();
        let id = orch.start_execution("agent-1", "1.0.0", serde_json::json!({}), 10, 60).await.unwrap();
        orch.cancel(id, "user requested").await.unwrap();
        let state = orch.get_state(id).await.unwrap();
        assert!(matches!(state, ExecutionState::Cancelled { .. }));
    }
}