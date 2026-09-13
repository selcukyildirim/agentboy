use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct StrategicInitiativeAgent;
impl StrategicInitiativeAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for StrategicInitiativeAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "management.strategic-initiative".to_string(), version: "2.0.0".to_string(), name: "Strategic Initiative".to_string(), department: "Management".to_string(), description: "Track strategic initiative progress, milestones, and alignment with goals".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["initiatives"].as_str().ok_or_else(|| AppError::Validation("Missing 'initiatives' CSV".to_string()))?;
        let inits = csv_util::parse_csv_to_maps(csv)?;
        if inits.is_empty() { return Err(AppError::Validation("No initiatives".to_string())); }

        let mut analysis: Vec<serde_json::Value> = inits.iter().map(|i| {
            let progress = csv_util::record_get_f64(i, "progress_pct");
            let budget = csv_util::record_get_f64(i, "budget");
            let spent = csv_util::record_get_f64(i, "spent");
            let budget_status = if budget > 0.0 && spent / budget > 1.1 { "over_budget" } else if budget > 0.0 && spent / budget > 0.9 { "on_budget" } else { "under_budget" };
            let progress_status = if progress >= 80.0 { "on_track" } else if progress >= 50.0 { "at_risk" } else { "behind" };
            serde_json::json!({ "name": csv_util::record_get_str(i, "name"), "owner": csv_util::record_get_str(i, "owner"), "progress_pct": progress, "budget": budget, "spent": spent, "budget_status": budget_status, "progress_status": progress_status })
        }).collect();
        analysis.sort_by(|a, b| a["progress_pct"].as_f64().unwrap_or(0.0).partial_cmp(&b["progress_pct"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let behind = analysis.iter().filter(|a| a["progress_status"] == "behind").count();
        let over_budget = analysis.iter().filter(|a| a["budget_status"] == "over_budget").count();

        let llm = ctx.call_llm("Analyze strategic initiatives.", &format!("Initiatives ({} total, {} behind, {} over budget):\n{}\n\nActions?", inits.len(), behind, over_budget, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total": inits.len(), "behind": behind, "over_budget": over_budget }, "initiatives": analysis, "llm_analysis": llm }))
    }
}
impl Default for StrategicInitiativeAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Digital transformation is behind.")) }

    #[tokio::test]
    async fn test_initiatives() {
        let input = serde_json::json!({ "initiatives": "name,owner,progress_pct,budget,spent\nDT,Alice,30,100000,50000\nCloud,Bob,85,50000,60000" });
        let r = StrategicInitiativeAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["behind"], 1);
        assert_eq!(r["summary"]["over_budget"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(StrategicInitiativeAgent::new().execute_with_context(serde_json::json!({"initiatives":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(StrategicInitiativeAgent::new().manifest().id, "management.strategic-initiative"); }
}