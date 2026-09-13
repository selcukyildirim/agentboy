use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct WarehouseOptimizerAgent;
impl WarehouseOptimizerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for WarehouseOptimizerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "logistics.warehouse-optimizer".to_string(), version: "2.0.0".to_string(), name: "Warehouse Optimizer".to_string(), department: "Logistics".to_string(), description: "Optimize warehouse layout, storage utilization, and picking efficiency".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["zones"].as_str().ok_or_else(|| AppError::Validation("Missing 'zones' CSV".to_string()))?;
        let zones = csv_util::parse_csv_to_maps(csv)?;
        if zones.is_empty() { return Err(AppError::Validation("No zones".to_string())); }

        let mut analysis: Vec<serde_json::Value> = zones.iter().map(|z| {
            let capacity = csv_util::record_get_f64(z, "capacity_units");
            let used = csv_util::record_get_f64(z, "used_units");
            let picks = csv_util::record_get_f64(z, "daily_picks");
            let util = if capacity > 0.0 { used / capacity * 100.0 } else { 0.0 };
            let status = if util > 95.0 { "overflow" } else if util > 80.0 { "optimal" } else { "underutilized" };
            serde_json::json!({ "zone": csv_util::record_get_str(z, "zone"), "capacity_units": capacity, "used_units": used, "utilization_pct": util, "daily_picks": picks, "status": status })
        }).collect();

        let overflow = analysis.iter().filter(|a| a["status"] == "overflow").count();
        let total_picks: f64 = analysis.iter().map(|a| a["daily_picks"].as_f64().unwrap_or(0.0)).sum();

        let llm = ctx.call_llm("Optimize warehouse layout.", &format!("Warehouse ({} zones, {} overflow, {} daily picks):\n{}\n\nOptimize?", zones.len(), overflow, total_picks, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_zones": zones.len(), "overflow": overflow, "total_daily_picks": total_picks }, "zones": analysis, "llm_analysis": llm }))
    }
}
impl Default for WarehouseOptimizerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Zone A is overflowing.")) }

    #[tokio::test]
    async fn test_warehouse() {
        let input = serde_json::json!({ "zones": "zone,capacity_units,used_units,daily_picks\nA,1000,980,50\nB,1000,500,30" });
        let r = WarehouseOptimizerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["overflow"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(WarehouseOptimizerAgent::new().execute_with_context(serde_json::json!({"zones":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(WarehouseOptimizerAgent::new().manifest().id, "logistics.warehouse-optimizer"); }
}