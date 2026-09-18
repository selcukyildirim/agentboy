use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};
use crate::csv_util;

pub struct DeliveryTrackerAgent;
impl DeliveryTrackerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for DeliveryTrackerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "logistics.delivery-tracker".to_string(), version: "2.0.0".to_string(), name: "Delivery Tracker".to_string(), department: "Logistics".to_string(), description: "Track delivery performance, SLA compliance, and identify delays".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 }, rag_enabled: false, output_schema: None, max_cost_usd: None,
            input_schema: vec![
                InputField::new("deliveries", "Deliveries", InputKind::File, true),
            ],
        }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["deliveries"].as_str().ok_or_else(|| AppError::Validation("Missing 'deliveries' CSV".to_string()))?;
        let deliveries = csv_util::parse_csv_to_maps(csv)?;
        if deliveries.is_empty() { return Err(AppError::Validation("No deliveries".to_string())); }

        let total = deliveries.len();
        let on_time = deliveries.iter().filter(|d| csv_util::record_get_str(d, "status") == "delivered" && csv_util::record_get_f64(d, "delay_hours") <= 0.0).count();
        let late = deliveries.iter().filter(|d| csv_util::record_get_f64(d, "delay_hours") > 0.0).count();
        let delays: Vec<f64> = deliveries.iter().filter_map(|d| d.get("delay_hours").and_then(|v| v.parse::<f64>().ok())).filter(|d| *d > 0.0).collect();
        let avg_delay = if !delays.is_empty() { delays.iter().sum::<f64>() / delays.len() as f64 } else { 0.0 };
        let sla_compliance = if total > 0 { on_time as f64 / total as f64 * 100.0 } else { 0.0 };

        let llm = ctx.call_llm("Analyze delivery performance.", &format!("Deliveries ({} total, {} on-time, {:.1}% SLA, avg delay: {:.1}h):\n\nRecommendations?", total, on_time, sla_compliance, avg_delay)).await?;
        Ok(serde_json::json!({ "summary": { "total_deliveries": total, "on_time": on_time, "late": late, "sla_compliance_pct": sla_compliance, "avg_delay_hours": avg_delay }, "llm_analysis": llm }))
    }
}
impl Default for DeliveryTrackerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("SLA is below target.")) }

    #[tokio::test]
    async fn test_deliveries() {
        let input = serde_json::json!({ "deliveries": "delivery_id,status,delay_hours\nD1,delivered,0\nD2,delivered,-1\nD3,delivered,3" });
        let r = DeliveryTrackerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["on_time"], 2);
        assert_eq!(r["summary"]["late"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(DeliveryTrackerAgent::new().execute_with_context(serde_json::json!({"deliveries":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(DeliveryTrackerAgent::new().manifest().id, "logistics.delivery-tracker"); }
}