use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct ChangeRequestAgent;
impl ChangeRequestAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for ChangeRequestAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "pmo.change-request".to_string(), version: "2.0.0".to_string(), name: "Change Request".to_string(), department: "PMO".to_string(), description: "Analyze change requests, assess impact on schedule/budget, and recommend approvals".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 }, rag_enabled: false, output_schema: None, max_cost_usd: None }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["changes"].as_str().ok_or_else(|| AppError::Validation("Missing 'changes' CSV".to_string()))?;
        let changes = csv_util::parse_csv_to_maps(csv)?;
        if changes.is_empty() { return Err(AppError::Validation("No change requests".to_string())); }

        let mut analysis: Vec<serde_json::Value> = changes.iter().map(|c| {
            let schedule_impact = csv_util::record_get_f64(c, "schedule_impact_days");
            let budget_impact = csv_util::record_get_f64(c, "budget_impact");
            let priority = csv_util::record_get_str(c, "priority");
            let total_impact = schedule_impact.abs() + budget_impact.abs() / 1000.0;
            let recommendation = if total_impact > 10.0 { "reject" } else if total_impact > 5.0 { "review" } else { "approve" };
            serde_json::json!({ "cr_id": csv_util::record_get_str(c, "cr_id"), "project": csv_util::record_get_str(c, "project"), "description": csv_util::record_get_str(c, "description"), "priority": priority, "schedule_impact_days": schedule_impact, "budget_impact": budget_impact, "recommendation": recommendation })
        }).collect();

        let approve = analysis.iter().filter(|a| a["recommendation"] == "approve").count();
        let reject = analysis.iter().filter(|a| a["recommendation"] == "reject").count();

        let llm = ctx.call_llm("Analyze change requests.", &format!("Change Requests ({} total, {} approve, {} reject):\n{}\n\nDecisions?", changes.len(), approve, reject, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_changes": changes.len(), "approve": approve, "reject": reject }, "changes": analysis, "llm_analysis": llm }))
    }
}
impl Default for ChangeRequestAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("CR-2 should be rejected.")) }

    #[tokio::test]
    async fn test_changes() {
        let input = serde_json::json!({ "changes": "cr_id,project,description,priority,schedule_impact_days,budget_impact\nCR-1,Alpha,Minor tweak,low,1,1000\nCR-2,Alpha,Major change,high,15,50000" });
        let r = ChangeRequestAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["approve"], 1);
        assert_eq!(r["summary"]["reject"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(ChangeRequestAgent::new().execute_with_context(serde_json::json!({"changes":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(ChangeRequestAgent::new().manifest().id, "pmo.change-request"); }
}