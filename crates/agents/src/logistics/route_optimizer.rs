use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct RouteOptimizerAgent;
impl RouteOptimizerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for RouteOptimizerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "logistics.route-optimizer".to_string(),
            version: "2.0.0".to_string(),
            name: "Route Optimizer".to_string(),
            department: "Logistics".to_string(),
            description:
                "Optimize delivery routes, reduce fuel costs, and improve on-time delivery"
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
            input_schema: vec![InputField::new("routes", "Routes", InputKind::File, true)],
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
        let csv = input["routes"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'routes' CSV".to_string()))?;
        let routes = csv_util::parse_csv_to_maps(csv)?;
        if routes.is_empty() {
            return Err(AppError::Validation("No routes".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = routes.iter().map(|r| {
            let distance = csv_util::record_get_f64(r, "distance_km");
            let fuel_cost = csv_util::record_get_f64(r, "fuel_cost");
            let deliveries = csv_util::record_get_f64(r, "deliveries") as u64;
            let on_time = csv_util::record_get_f64(r, "on_time_pct");
            let cost_per_delivery = if deliveries > 0 { fuel_cost / deliveries as f64 } else { 0.0 };
            let efficiency = if distance > 0.0 { deliveries as f64 / distance } else { 0.0 };
            serde_json::json!({ "route_id": csv_util::record_get_str(r, "route_id"), "driver": csv_util::record_get_str(r, "driver"), "distance_km": distance, "fuel_cost": fuel_cost, "deliveries": deliveries, "on_time_pct": on_time, "cost_per_delivery": cost_per_delivery, "efficiency": efficiency })
        }).collect();
        analysis.sort_by(|a, b| {
            b["cost_per_delivery"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["cost_per_delivery"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_cost: f64 = analysis
            .iter()
            .map(|a| a["fuel_cost"].as_f64().unwrap_or(0.0))
            .sum();
        let total_deliveries: u64 = analysis
            .iter()
            .map(|a| a["deliveries"].as_u64().unwrap_or(0))
            .sum();
        let avg_on_time: f64 = if !analysis.is_empty() {
            analysis
                .iter()
                .map(|a| a["on_time_pct"].as_f64().unwrap_or(0.0))
                .sum::<f64>()
                / analysis.len() as f64
        } else {
            0.0
        };

        let llm = ctx.call_llm("Optimize routes and reduce costs.", &format!("Routes ({} routes, {} deliveries, total cost: {:.0}, avg on-time: {:.1}%):\n{}\n\nOptimize?", routes.len(), total_deliveries, total_cost, avg_on_time, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(
            serde_json::json!({ "summary": { "total_routes": routes.len(), "total_cost": total_cost, "total_deliveries": total_deliveries, "avg_on_time_pct": avg_on_time }, "routes": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for RouteOptimizerAgent {
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
            "Route R2 is least efficient.",
        ))
    }

    #[tokio::test]
    async fn test_routes() {
        let input = serde_json::json!({ "routes": "route_id,driver,distance_km,fuel_cost,deliveries,on_time_pct\nR1,Alice,50,100,10,95\nR2,Bob,200,400,5,70" });
        let r = RouteOptimizerAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["total_routes"], 2);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(RouteOptimizerAgent::new()
            .execute_with_context(serde_json::json!({"routes":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            RouteOptimizerAgent::new().manifest().id,
            "logistics.route-optimizer"
        );
    }
}
