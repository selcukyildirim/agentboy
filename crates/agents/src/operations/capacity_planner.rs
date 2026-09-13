use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct CapacityPlannerAgent;
impl CapacityPlannerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for CapacityPlannerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "operations.capacity-planner".to_string(), version: "2.0.0".to_string(), name: "Capacity Planner".to_string(), department: "Operations".to_string(), description: "Analyze production capacity, utilization rates, and identify bottlenecks".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["machines"].as_str().ok_or_else(|| AppError::Validation("Missing 'machines' CSV".to_string()))?;
        let machines = csv_util::parse_csv_to_maps(csv)?;
        if machines.is_empty() { return Err(AppError::Validation("No machines".to_string())); }

        let mut analysis: Vec<serde_json::Value> = machines.iter().map(|m| {
            let capacity = csv_util::record_get_f64(m, "capacity_hours");
            let used = csv_util::record_get_f64(m, "used_hours");
            let utilization = if capacity > 0.0 { used / capacity * 100.0 } else { 0.0 };
            let status = if utilization > 90.0 { "bottleneck" } else if utilization > 70.0 { "optimal" } else { "underutilized" };
            serde_json::json!({ "machine": csv_util::record_get_str(m, "machine"), "line": csv_util::record_get_str(m, "line"), "capacity_hours": capacity, "used_hours": used, "utilization_pct": utilization, "status": status })
        }).collect();
        analysis.sort_by(|a, b| b["utilization_pct"].as_f64().unwrap_or(0.0).partial_cmp(&a["utilization_pct"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let avg_util: f64 = if !analysis.is_empty() { analysis.iter().map(|a| a["utilization_pct"].as_f64().unwrap_or(0.0)).sum::<f64>() / analysis.len() as f64 } else { 0.0 };
        let bottlenecks = analysis.iter().filter(|a| a["status"] == "bottleneck").count();

        let llm = ctx.call_llm("Analyze capacity and recommend balancing.", &format!("Capacity ({} machines, avg util: {:.1}%):\n{}\n\nRecommendations?", machines.len(), avg_util, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_machines": machines.len(), "avg_utilization": avg_util, "bottlenecks": bottlenecks }, "machines": analysis, "llm_analysis": llm }))
    }
}
impl Default for CapacityPlannerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Machine A is a bottleneck.")) }

    #[tokio::test]
    async fn test_capacity() {
        let input = serde_json::json!({ "machines": "machine,line,capacity_hours,used_hours\nM1,Line A,100,95\nM2,Line A,100,40" });
        let r = CapacityPlannerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["bottlenecks"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(CapacityPlannerAgent::new().execute_with_context(serde_json::json!({"machines":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(CapacityPlannerAgent::new().manifest().id, "operations.capacity-planner"); }
}