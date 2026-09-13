use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct FleetManagerAgent;
impl FleetManagerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for FleetManagerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "logistics.fleet-manager".to_string(), version: "2.0.0".to_string(), name: "Fleet Manager".to_string(), department: "Logistics".to_string(), description: "Monitor fleet utilization, vehicle health, and maintenance schedules".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["vehicles"].as_str().ok_or_else(|| AppError::Validation("Missing 'vehicles' CSV".to_string()))?;
        let vehicles = csv_util::parse_csv_to_maps(csv)?;
        if vehicles.is_empty() { return Err(AppError::Validation("No vehicles".to_string())); }

        let mut analysis: Vec<serde_json::Value> = vehicles.iter().map(|v| {
            let mileage = csv_util::record_get_f64(v, "mileage_km");
            let fuel = csv_util::record_get_f64(v, "fuel_cost");
            let utilization = csv_util::record_get_f64(v, "utilization_pct");
            let health = csv_util::record_get_f64(v, "health_score");
            let status = if health < 0.4 { "needs_service" } else if utilization < 0.3 { "underutilized" } else { "ok" };
            serde_json::json!({ "vehicle_id": csv_util::record_get_str(v, "vehicle_id"), "type": csv_util::record_get_str(v, "type"), "mileage_km": mileage, "fuel_cost": fuel, "utilization_pct": utilization, "health_score": health, "status": status })
        }).collect();

        let total_fuel: f64 = analysis.iter().map(|a| a["fuel_cost"].as_f64().unwrap_or(0.0)).sum();
        let needs_service = analysis.iter().filter(|a| a["status"] == "needs_service").count();
        let underutilized = analysis.iter().filter(|a| a["status"] == "underutilized").count();

        let llm = ctx.call_llm("Analyze fleet and recommend actions.", &format!("Fleet ({} vehicles, total fuel: {:.0}, needs service: {}, underutilized: {}):\n{}\n\nRecommendations?", vehicles.len(), total_fuel, needs_service, underutilized, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_vehicles": vehicles.len(), "total_fuel_cost": total_fuel, "needs_service": needs_service, "underutilized": underutilized }, "vehicles": analysis, "llm_analysis": llm }))
    }
}
impl Default for FleetManagerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("V002 needs immediate service.")) }

    #[tokio::test]
    async fn test_fleet() {
        let input = serde_json::json!({ "vehicles": "vehicle_id,type,mileage_km,fuel_cost,utilization_pct,health_score\nV001,Truck,50000,5000,0.8,0.9\nV002,Van,30000,3000,0.2,0.3\nV003,Bus,20000,2000,0.15,0.8" });
        let r = FleetManagerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["needs_service"], 1);
        assert_eq!(r["summary"]["underutilized"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(FleetManagerAgent::new().execute_with_context(serde_json::json!({"vehicles":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(FleetManagerAgent::new().manifest().id, "logistics.fleet-manager"); }
}