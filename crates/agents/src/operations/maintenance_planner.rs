use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct MaintenancePlannerAgent;
impl MaintenancePlannerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for MaintenancePlannerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "operations.maintenance-planner".to_string(), version: "2.0.0".to_string(), name: "Maintenance Planner".to_string(), department: "Operations".to_string(), description: "Schedule preventive maintenance, predict failures, and minimize downtime".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["equipment"].as_str().ok_or_else(|| AppError::Validation("Missing 'equipment' CSV".to_string()))?;
        let equipment = csv_util::parse_csv_to_maps(csv)?;
        if equipment.is_empty() { return Err(AppError::Validation("No equipment".to_string())); }

        let mut analysis: Vec<serde_json::Value> = equipment.iter().map(|e| {
            let hours_since = csv_util::record_get_f64(e, "hours_since_maintenance");
            let interval = csv_util::record_get_f64(e, "maintenance_interval_hours");
            let health = csv_util::record_get_f64(e, "health_score");
            let urgency = if hours_since >= interval { "overdue" } else if hours_since >= interval * 0.8 { "due_soon" } else { "ok" };
            serde_json::json!({ "equipment_id": csv_util::record_get_str(e, "equipment_id"), "name": csv_util::record_get_str(e, "name"), "hours_since_maintenance": hours_since, "maintenance_interval_hours": interval, "health_score": health, "urgency": urgency })
        }).collect();
        analysis.sort_by(|a, b| a["urgency"].as_str().cmp(&b["urgency"].as_str()).then_with(|| b["hours_since_maintenance"].as_f64().unwrap_or(0.0).partial_cmp(&a["hours_since_maintenance"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal)));

        let overdue = analysis.iter().filter(|a| a["urgency"] == "overdue").count();
        let due_soon = analysis.iter().filter(|a| a["urgency"] == "due_soon").count();
        let llm = ctx.call_llm("Prioritize maintenance schedule.", &format!("Maintenance ({} equipment, {} overdue, {} due soon):\n{}\n\nSchedule?", equipment.len(), overdue, due_soon, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_equipment": equipment.len(), "overdue": overdue, "due_soon": due_soon }, "equipment": analysis, "llm_analysis": llm }))
    }
}
impl Default for MaintenancePlannerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Overhaul Machine M1 immediately.")) }

    #[tokio::test]
    async fn test_maintenance() {
        let input = serde_json::json!({ "equipment": "equipment_id,name,hours_since_maintenance,maintenance_interval_hours,health_score\nM1,Press,500,400,0.6\nM2,Lathe,100,500,0.9" });
        let r = MaintenancePlannerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["overdue"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(MaintenancePlannerAgent::new().execute_with_context(serde_json::json!({"equipment":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(MaintenancePlannerAgent::new().manifest().id, "operations.maintenance-planner"); }
}