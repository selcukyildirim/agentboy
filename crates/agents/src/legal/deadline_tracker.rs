use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct DeadlineTrackerAgent;
impl DeadlineTrackerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for DeadlineTrackerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "legal.deadline-tracker".to_string(), version: "2.0.0".to_string(), name: "Deadline Tracker".to_string(), department: "Legal".to_string(), description: "Track legal deadlines, filing dates, statute of limitations, and renewal dates".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["deadlines"].as_str().ok_or_else(|| AppError::Validation("Missing 'deadlines' CSV".to_string()))?;
        let deadlines = csv_util::parse_csv_to_maps(csv)?;
        if deadlines.is_empty() { return Err(AppError::Validation("No deadlines".to_string())); }

        let mut analysis: Vec<serde_json::Value> = deadlines.iter().map(|d| {
            let days_remaining = csv_util::record_get_f64(d, "days_remaining");
            let priority = csv_util::record_get_str(d, "priority");
            let status = if days_remaining < 0.0 { "overdue" } else if days_remaining <= 7.0 { "urgent" } else if days_remaining <= 30.0 { "upcoming" } else { "on_track" };
            serde_json::json!({ "item": csv_util::record_get_str(d, "item"), "type": csv_util::record_get_str(d, "type"), "days_remaining": days_remaining, "priority": priority, "status": status })
        }).collect();
        analysis.sort_by(|a, b| a["days_remaining"].as_f64().unwrap_or(f64::INFINITY).partial_cmp(&b["days_remaining"].as_f64().unwrap_or(f64::INFINITY)).unwrap_or(std::cmp::Ordering::Equal));

        let overdue = analysis.iter().filter(|a| a["status"] == "overdue").count();
        let urgent = analysis.iter().filter(|a| a["status"] == "urgent").count();

        let llm = ctx.call_llm("Review legal deadlines.", &format!("Deadlines ({} total, {} overdue, {} urgent):\n{}\n\nPriorities?", deadlines.len(), overdue, urgent, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_deadlines": deadlines.len(), "overdue": overdue, "urgent": urgent }, "deadlines": analysis, "llm_analysis": llm }))
    }
}
impl Default for DeadlineTrackerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Tax filing is overdue.")) }

    #[tokio::test]
    async fn test_deadlines() {
        let input = serde_json::json!({ "deadlines": "item,type,days_remaining,priority\nTax Filing,Filing,-2,high\nContract Renewal,Renewal,5,high\nAudit Response,Compliance,20,medium" });
        let r = DeadlineTrackerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["overdue"], 1);
        assert_eq!(r["summary"]["urgent"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(DeadlineTrackerAgent::new().execute_with_context(serde_json::json!({"deadlines":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(DeadlineTrackerAgent::new().manifest().id, "legal.deadline-tracker"); }
}