use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct BoardReportAgent;
impl BoardReportAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for BoardReportAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "management.board-report".to_string(), version: "2.0.0".to_string(), name: "Board Report".to_string(), department: "Management".to_string(), description: "Generate board-ready executive summary with key metrics and strategic updates".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["metrics"].as_str().ok_or_else(|| AppError::Validation("Missing 'metrics' CSV".to_string()))?;
        let metrics = csv_util::parse_csv_to_maps(csv)?;
        if metrics.is_empty() { return Err(AppError::Validation("No metrics".to_string())); }

        let mut analysis: Vec<serde_json::Value> = metrics.iter().map(|m| {
            let current = csv_util::record_get_f64(m, "current");
            let prev = csv_util::record_get_f64(m, "previous");
            let change = if prev != 0.0 { (current - prev) / prev * 100.0 } else { 0.0 };
            let trend = if change > 5.0 { "improving" } else if change < -5.0 { "declining" } else { "stable" };
            serde_json::json!({ "metric": csv_util::record_get_str(m, "metric"), "current": current, "previous": prev, "change_pct": change, "trend": trend })
        }).collect();

        let improving = analysis.iter().filter(|a| a["trend"] == "improving").count();
        let declining = analysis.iter().filter(|a| a["trend"] == "declining").count();

        let llm = ctx.call_llm("Generate board executive summary.", &format!("Board Report ({} metrics, {} improving, {} declining):\n{}\n\nSummarize for board.", metrics.len(), improving, declining, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_metrics": metrics.len(), "improving": improving, "declining": declining }, "metrics": analysis, "llm_analysis": llm }))
    }
}
impl Default for BoardReportAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Revenue is growing. Costs need attention.")) }

    #[tokio::test]
    async fn test_board() {
        let input = serde_json::json!({ "metrics": "metric,current,previous\nRevenue,1000,900\nCosts,800,750" });
        let r = BoardReportAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["improving"], 2);
        assert_eq!(r["summary"]["declining"], 0);
    }
    #[tokio::test]
    async fn test_empty() { assert!(BoardReportAgent::new().execute_with_context(serde_json::json!({"metrics":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(BoardReportAgent::new().manifest().id, "management.board-report"); }
}