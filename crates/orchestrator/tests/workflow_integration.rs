use agent_runtime::{MockAgentContext, MockLlmProvider};
use async_trait::async_trait;
use orchestrator::{build_registry, Orchestrator};
use std::sync::Arc;
use workflow_engine::runner::{ExecutionStatus, StepExecutor, WorkflowRunner};
use workflow_engine::workflow::{ParameterType, Workflow, WorkflowParameter, WorkflowStep};

struct AgentStepExecutor {
    orchestrator: Arc<Orchestrator>,
    skills: Arc<skill_sdk::SkillRegistry>,
    ctx: MockAgentContext,
}

#[async_trait]
impl StepExecutor for AgentStepExecutor {
    async fn execute_step(
        &self,
        skill_id: &str,
        input: serde_json::Value,
    ) -> agent_common::error::AppResult<serde_json::Value> {
        let agent = agents::BankReconciliationAgent::new();
        // skill_id is the agent id in this test.
        assert_eq!(skill_id, "finance.bank-reconciliation");
        let (_, output) = self
            .orchestrator
            .execute(&self.skills, &agent, input, Some(&self.ctx), None)
            .await?;
        Ok(output)
    }
}

#[tokio::test]
async fn workflow_runs_real_agent_step() {
    let orchestrator = Arc::new(Orchestrator::new());
    let skills = Arc::new(build_registry());
    let ctx = MockAgentContext::new(MockLlmProvider::with_response("ok"));

    let executor = AgentStepExecutor {
        orchestrator,
        skills,
        ctx,
    };

    let mut workflow = Workflow::new("wf-test", "Reconciliation flow");
    workflow.add_parameter(WorkflowParameter {
        name: "bank_statement".into(),
        param_type: ParameterType::String,
        default: None,
        required: true,
        description: "Bank statement CSV".into(),
    });
    workflow.add_parameter(WorkflowParameter {
        name: "ledger".into(),
        param_type: ParameterType::String,
        default: None,
        required: true,
        description: "Ledger CSV".into(),
    });
    workflow.add_step(WorkflowStep {
        id: "s1".into(),
        skill_id: "finance.bank-reconciliation".into(),
        name: "Reconcile".into(),
        input_mapping: Some(serde_json::json!({
            "bank_statement": "$param.bank_statement",
            "ledger": "$param.ledger"
        })),
        output_mapping: None,
        depends_on: vec![],
        timeout_seconds: None,
        retry_count: None,
        rollback: None,
    });

    let input = serde_json::json!({
        "bank_statement": "date,amount,reference\n2024-01-01,100,REF1",
        "ledger": "date,amount,reference\n2024-01-01,100,REF1",
    });

    let runner = WorkflowRunner::new();
    let execution = runner.run_with(&executor, &workflow, input).await.unwrap();

    assert_eq!(execution.status, ExecutionStatus::Completed);
    assert_eq!(execution.step_results.len(), 1);
    assert!(execution.step_results[0].output.is_some());
}
