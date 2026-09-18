use crate::context::AgentContext;
use crate::manifest::{AgentManifest, AgentPermissions, AgentTier};
use agent_common::error::{AppError, AppResult};
use audit_core::event::{AuditEvent, AuditResult};
use audit_core::store::SqliteAuditStore;
use async_trait::async_trait;
use chrono::Utc;

#[async_trait]
pub trait Agent: Send + Sync {
    fn manifest(&self) -> AgentManifest;

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        Err(AppError::Validation(
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
        Self::run_with_audit(agent, input, ctx, None).await
    }

    pub async fn run_with_audit(
        agent: &dyn Agent,
        input: serde_json::Value,
        ctx: Option<&dyn AgentContext>,
        audit_store: Option<&SqliteAuditStore>,
    ) -> AppResult<serde_json::Value> {
        let manifest = agent.manifest();
        let agent_id = manifest.id.clone();
        let timeout_secs = manifest.execution.timeout_seconds;

        if let Err(e) = Self::check_entitlement(&manifest) {
            if let Some(store) = audit_store {
                let _ = store.record(AuditEvent {
                    event_id: uuid::Uuid::new_v4(),
                    execution_id: None,
                    agent_id: agent_id.clone(),
                    action: "execute".to_string(),
                    resource: format!("agent/{}", agent_id),
                    result: AuditResult::Denied,
                    details: Some(serde_json::json!({"error": e.to_string()})),
                    timestamp: Utc::now(),
                }).await;
            }
            return Err(e);
        }

        if let Err(e) = Self::check_permissions(&manifest.permissions) {
            if let Some(store) = audit_store {
                let _ = store.record(AuditEvent {
                    event_id: uuid::Uuid::new_v4(),
                    execution_id: None,
                    agent_id: agent_id.clone(),
                    action: "execute".to_string(),
                    resource: format!("agent/{}", agent_id),
                    result: AuditResult::Denied,
                    details: Some(serde_json::json!({"error": e.to_string()})),
                    timestamp: Utc::now(),
                }).await;
            }
            return Err(e);
        }

        let result = {
            let exec_result = match ctx {
                Some(context) if agent.supports_context() => {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(timeout_secs),
                        agent.execute_with_context(input, context),
                    ).await
                }
                _ => {
                    tokio::time::timeout(
                        std::time::Duration::from_secs(timeout_secs),
                        agent.execute(input),
                    ).await
                }
            };

            match exec_result {
                Ok(result) => result,
                Err(_) => {
                    let err = AppError::Validation(format!(
                        "Agent '{}' timed out after {} seconds",
                        agent_id, timeout_secs
                    ));
                    if let Some(store) = audit_store {
                        let _ = store.record(AuditEvent {
                            event_id: uuid::Uuid::new_v4(),
                            execution_id: None,
                            agent_id: agent_id.clone(),
                            action: "execute".to_string(),
                            resource: format!("agent/{}", agent_id),
                            result: AuditResult::Failure,
                            details: Some(serde_json::json!({"error": err.to_string()})),
                            timestamp: Utc::now(),
                        }).await;
                    }
                    return Err(err);
                }
            }
        };

        if let Some(store) = audit_store {
            let audit_result = if result.is_ok() {
                AuditResult::Success
            } else {
                AuditResult::Failure
            };
            let details = match &result {
                Ok(v) => Some(serde_json::json!({"output_preview": v.to_string().chars().take(500).collect::<String>()})),
                Err(e) => Some(serde_json::json!({"error": e.to_string()})),
            };
            let _ = store.record(AuditEvent {
                event_id: uuid::Uuid::new_v4(),
                execution_id: None,
                agent_id: agent_id.clone(),
                action: "execute".to_string(),
                resource: format!("agent/{}", agent_id),
                result: audit_result,
                details,
                timestamp: Utc::now(),
            }).await;
        }

        result
    }

    fn check_entitlement(manifest: &AgentManifest) -> AppResult<()> {
        match manifest.tier {
            AgentTier::Free => Ok(()),
            AgentTier::Paid => {
                tracing::warn!(
                    agent_id = %manifest.id,
                    "Paid agent requires entitlement — denying execution"
                );
                Err(AppError::Entitlement(format!(
                    "Agent '{}' requires a paid license",
                    manifest.id
                )))
            }
        }
    }

    fn check_permissions(permissions: &AgentPermissions) -> AppResult<()> {
        tracing::debug!(
            fs_read = permissions.filesystem_read,
            fs_write = permissions.filesystem_write,
            llm = permissions.network_llm,
            "Agent permissions checked"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ExecutionLimits;

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
                rag_enabled: false,
                output_schema: None,
                max_cost_usd: None,
                input_schema: vec![],
            }
        }

        async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
            Ok(serde_json::json!({
                "status": "ok",
                "input_received": input
            }))
        }
    }

    struct PaidAgent;

    #[async_trait]
    impl Agent for PaidAgent {
        fn manifest(&self) -> AgentManifest {
            AgentManifest {
                id: "test.paid".to_string(),
                version: "1.0.0".to_string(),
                name: "Paid".to_string(),
                department: "Test".to_string(),
                description: "A paid test agent".to_string(),
                tier: AgentTier::Paid,
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
                rag_enabled: false,
                output_schema: None,
                max_cost_usd: None,
                input_schema: vec![],
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

    #[tokio::test]
    async fn test_paid_agent_blocked() {
        let agent = PaidAgent;
        let result = AgentExecutor::run(&agent, serde_json::json!({}), None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Entitlement(_)));
    }

    #[tokio::test]
    async fn test_audit_trail_records() {
        let store = SqliteAuditStore::new("sqlite::memory:").await.unwrap();
        let agent = SimpleAgent;
        let result = AgentExecutor::run_with_audit(
            &agent,
            serde_json::json!({"data": 1}),
            None,
            Some(&store),
        ).await.unwrap();
        assert_eq!(result["status"], "ok");

        let events = store.query("test.simple", 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, AuditResult::Success);
    }

    #[tokio::test]
    async fn test_audit_trail_denied() {
        let store = SqliteAuditStore::new("sqlite::memory:").await.unwrap();
        let agent = PaidAgent;
        let result = AgentExecutor::run_with_audit(
            &agent,
            serde_json::json!({}),
            None,
            Some(&store),
        ).await;
        assert!(result.is_err());

        let events = store.query("test.paid", 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].result, AuditResult::Denied);
    }
}
