use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct ProductionSchedulerAgent;
impl ProductionSchedulerAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for ProductionSchedulerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "operations.production-scheduler".to_string(),
            version: "2.0.0".to_string(),
            name: "Production Scheduler".to_string(),
            department: "Operations".to_string(),
            description:
                "Optimize production scheduling, capacity utilization, and identify bottlenecks"
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
                "orders",
                "Production Orders",
                InputKind::File,
                true,
            )
            .with_example("order_id,customer,amount,status\nO1,Acme,5000,on_hold")],
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
        let csv = input["orders"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'orders' CSV".to_string()))?;
        let orders = csv_util::parse_csv_to_maps(csv)?;
        if orders.is_empty() {
            return Err(AppError::Validation("No orders".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = orders.iter().map(|o| {
            let required = csv_util::record_get_f64(o, "required_date_days");
            let lead_time = csv_util::record_get_f64(o, "lead_time_days");
            let urgency = if required <= lead_time { "critical" } else if required <= lead_time * 1.5 { "tight" } else { "normal" };
            serde_json::json!({ "order_id": csv_util::record_get_str(o, "order_id"), "product": csv_util::record_get_str(o, "product"), "quantity": csv_util::record_get_f64(o, "quantity"), "required_date_days": required, "lead_time_days": lead_time, "urgency": urgency })
        }).collect();
        analysis.sort_by(|a, b| {
            a["required_date_days"]
                .as_f64()
                .unwrap_or(f64::INFINITY)
                .partial_cmp(&b["required_date_days"].as_f64().unwrap_or(f64::INFINITY))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let critical = analysis
            .iter()
            .filter(|a| a["urgency"] == "critical")
            .count();
        let user_prompt = format!(
            "Production Schedule ({} orders, {} critical):\n{}\n\nOptimize schedule.",
            orders.len(),
            critical,
            serde_json::to_string_pretty(&analysis).unwrap_or_default()
        );
        let llm = ctx
            .call_llm(
                "Optimize production schedule and flag bottlenecks.",
                &user_prompt,
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_orders": orders.len(), "critical": critical }, "orders": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for ProductionSchedulerAgent {
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
            "Prioritize critical orders.",
        ))
    }

    #[tokio::test]
    async fn test_scheduling() {
        let input = serde_json::json!({ "orders": "order_id,product,quantity,required_date_days,lead_time_days\nO1,Widget,100,5,10\nO2,Gadget,50,15,8" });
        let r = ProductionSchedulerAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["critical"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(ProductionSchedulerAgent::new()
            .execute_with_context(serde_json::json!({"orders":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            ProductionSchedulerAgent::new().manifest().id,
            "operations.production-scheduler"
        );
    }
}
