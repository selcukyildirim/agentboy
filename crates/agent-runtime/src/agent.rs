use crate::context::AgentContext;
use crate::manifest::AgentManifest;
use agent_common::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    fn manifest(&self) -> AgentManifest;

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        Err(agent_common::error::AppError::Validation(
            "execute(input) not implemented - use execute_with_context".to_string(),
        ))
    }

    async fn execute_with_context(
        &self,
        input: serde_json::Value,
        ctx: &dyn AgentContext,
    ) -> AppResult<serde_json::Value> {
        self.execute(input).await
    }

    fn supports_context(&self) -> bool {
        false
    }
}

pub struct AgentExecutor;

impl AgentExecutor {
    pub async fn run(
        agent: &dyn Agent,
        input: serde_json::Value,
        ctx: Option<&dyn AgentContext>,
    ) -> AppResult<serde_json::Value> {
        match ctx {
            Some(context) if agent.supports_context() => {
                agent.execute_with_context(input, context).await
            }
            _ => agent.execute(input).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{AgentPermissions, AgentTier, ExecutionLimits};

    struct SimpleAgent;

    #[async_trait]
    impl Agent for SimpleAgent {
        fn manifest(&self) -> AgentManifest {
            AgentManifest {
                id: "test.simple".to_string(),
                version: "1.0.0".to_string(),
                name: "Simple".to_string(),
                department: "Test".to_string(),
                description: "A simple test agent".to_string(),
                tier: AgentTier::Free,
                skills: vec![],
                permissions: AgentPermissions {
                    filesystem_read: false,
                    filesystem_write: false,
                    network_llm: false,
                },
                execution: ExecutionLimits {
                    max_steps: 10,
                    timeout_seconds: 60,
                },
            }
        }

        async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
            Ok(serde_json::json!({
                "status": "ok",
                "input_received": input
            }))
        }
    }

    #[tokio::test]
    async fn test_agent_execute() {
        let agent = SimpleAgent;
        let result = agent.execute(serde_json::json!({"test": true})).await.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn test_agent_executor_run() {
        let agent = SimpleAgent;
        let result = AgentExecutor::run(&agent, serde_json::json!({"data": 1}), None).await.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_agent_supports_context() {
        let agent = SimpleAgent;
        assert!(!agent.supports_context());
    }
}