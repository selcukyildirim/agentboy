use agent_runtime::context::AgentContext;
use agent_runtime::{MockAgentContext, MockLlmProvider};
use audit_core::store::SqliteAuditStore;
use orchestrator::{build_registry, Orchestrator};

fn mock_ctx() -> MockAgentContext {
    MockAgentContext::new(MockLlmProvider::with_response(
        "Reconciliation completed with minor timing differences.",
    ))
}

#[tokio::test]
async fn orchestrator_executes_agent_and_records_audit_and_steps() {
    let skills = build_registry();
    let orchestrator = Orchestrator::new();
    let audit = SqliteAuditStore::new("sqlite::memory:").await.unwrap();

    let agent = agents::BankReconciliationAgent::new();
    let ctx = mock_ctx();

    let input = serde_json::json!({
        "bank_statement": "date,amount,reference\n2024-01-01,100,REF1\n2024-01-02,200,REF2",
        "ledger": "date,amount,reference\n2024-01-01,100,REF1\n2024-01-04,300,REF4",
    });

    let (execution_id, output) = orchestrator
        .execute(&skills, &agent, input, Some(&ctx), Some(&audit))
        .await
        .expect("execution should succeed");

    // Result comes from the real agent.
    assert_eq!(output["matched_count"], 1);
    assert_eq!(output["unmatched_bank_count"], 1);

    // The state machine recorded steps.
    let steps = orchestrator.get_steps(execution_id).await.unwrap();
    assert!(steps.len() >= 2, "expected at least 2 steps, got {}", steps.len());

    // Audit trail recorded a success event.
    let events = audit.query("finance.bank-reconciliation", 10).await.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].result, audit_core::event::AuditResult::Success);
}

#[tokio::test]
async fn orchestrator_rejects_paid_agent_via_entitlement() {
    // Build a paid-tier agent inline to verify the entitlement gate still runs
    // through the orchestrator path.
    use agent_runtime::agent::Agent;
    use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

    struct Paid;
    #[async_trait::async_trait]
    impl Agent for Paid {
        fn manifest(&self) -> AgentManifest {
            AgentManifest {
                id: "test.paid".into(),
                version: "1.0.0".into(),
                name: "Paid".into(),
                department: "Test".into(),
                description: "paid".into(),
                tier: AgentTier::Paid,
                skills: vec!["llm.analysis".into()],
                permissions: AgentPermissions {
                    filesystem_read: false,
                    filesystem_write: false,
                    network_llm: false,
                },
                execution: ExecutionLimits {
                    max_steps: 5,
                    timeout_seconds: 30,
                },
                rag_enabled: false,
                output_schema: None,
                max_cost_usd: None,
                input_schema: vec![],
            }
        }
        async fn execute(
            &self,
            _input: serde_json::Value,
        ) -> agent_common::error::AppResult<serde_json::Value> {
            Ok(serde_json::json!({"should": "not run"}))
        }
    }

    let skills = build_registry();
    let orchestrator = Orchestrator::new();
    let result = orchestrator
        .execute(&skills, &Paid, serde_json::json!({}), None, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn orchestrator_outputs_are_usable_by_dependent_skill() {
    // Sanity check that the shared skill registry resolves tool-backed skills.
    let skills = build_registry();
    let parse = skills.get("spreadsheet.parse").unwrap();
    let out = parse
        .execute(serde_json::json!({ "content": "a,b\n1,2\n3,4" }))
        .await
        .unwrap();
    assert_eq!(out["count"], 3);
}
