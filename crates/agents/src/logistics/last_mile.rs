use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct LastMileAgent;
impl LastMileAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for LastMileAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "logistics.last-mile".to_string(),
            version: "2.0.0".to_string(),
            name: "Last Mile".to_string(),
            department: "Logistics".to_string(),
            description:
                "Optimize last-mile delivery, customer satisfaction, and cost per delivery"
                    .to_string(),
            tier: AgentTier::Free,
            skills: vec![
                "spreadsheet.parse".to_string(),
                "spreadsheet.analyze".to_string(),
                "llm.analysis".to_string(),
            ],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 30,
                timeout_seconds: 120,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![InputField::new(
                "deliveries",
                "Deliveries",
                InputKind::File,
                true,
            )],
        }
    }
    fn supports_context(&self) -> bool {
        true
    }
    async fn execute_with_context(
        &self,
        input: serde_json::Value,
        ctx: &dyn AgentContext,
    ) -> AppResult<serde_json::Value> {
        let csv = input["deliveries"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'deliveries' CSV".to_string()))?;
        let deliveries = csv_util::parse_csv_to_maps(csv)?;
        if deliveries.is_empty() {
            return Err(AppError::Validation("No deliveries".to_string()));
        }

        let total = deliveries.len();
        let total_cost: f64 = deliveries
            .iter()
            .map(|d| csv_util::record_get_f64(d, "cost"))
            .sum();
        let cost_per = if total > 0 {
            total_cost / total as f64
        } else {
            0.0
        };
        let successful = deliveries
            .iter()
            .filter(|d| csv_util::record_get_str(d, "status") == "delivered")
            .count();
        let failed = deliveries
            .iter()
            .filter(|d| csv_util::record_get_str(d, "status") == "failed")
            .count();
        let first_attempt = deliveries
            .iter()
            .filter(|d| {
                csv_util::record_get_f64(d, "attempt_number") <= 1.0
                    && csv_util::record_get_str(d, "status") == "delivered"
            })
            .count();
        let success_rate = if total > 0 {
            successful as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        let first_attempt_rate = if successful > 0 {
            first_attempt as f64 / successful as f64 * 100.0
        } else {
            0.0
        };

        let llm = ctx.call_llm("Optimize last-mile delivery.", &format!("Last Mile ({} deliveries, {}% success, {}% first-attempt, cost/delivery: {:.2}):\n\nOptimize?", total, success_rate, first_attempt_rate, cost_per)).await?;
        Ok(
            serde_json::json!({ "summary": { "total_deliveries": total, "success_rate_pct": success_rate, "first_attempt_rate_pct": first_attempt_rate, "cost_per_delivery": cost_per, "failed": failed }, "llm_analysis": llm }),
        )
    }
}
impl Default for LastMileAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "First-attempt rate needs improvement.",
        ))
    }

    #[tokio::test]
    async fn test_last_mile() {
        let input = serde_json::json!({ "deliveries": "delivery_id,status,cost,attempt_number\nD1,delivered,10,1\nD2,delivered,15,2\nD3,failed,5,1" });
        let r = LastMileAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["total_deliveries"], 3);
        assert!(r["summary"]["success_rate_pct"].as_f64().unwrap() > 60.0);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(LastMileAgent::new()
            .execute_with_context(serde_json::json!({"deliveries":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(LastMileAgent::new().manifest().id, "logistics.last-mile");
    }
}
