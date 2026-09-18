use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct InventoryOptimizerAgent;

impl InventoryOptimizerAgent {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl Agent for InventoryOptimizerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "operations.inventory-optimizer".to_string(), version: "2.0.0".to_string(),
            name: "Inventory Optimizer".to_string(), department: "Operations".to_string(),
            description: "Optimize inventory levels, identify dead stock, and recommend reorder points".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
        }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["inventory"].as_str().ok_or_else(|| AppError::Validation("Missing 'inventory' CSV".to_string()))?;
        let items = csv_util::parse_csv_to_maps(csv)?;
        if items.is_empty() { return Err(AppError::Validation("No inventory items".to_string())); }

        let mut analysis: Vec<serde_json::Value> = items.iter().map(|i| {
            let qty = csv_util::record_get_f64(i, "quantity");
            let reorder = csv_util::record_get_f64(i, "reorder_point");
            let unit_cost = csv_util::record_get_f64(i, "unit_cost");
            let daily_demand = csv_util::record_get_f64(i, "daily_demand");
            let days_of_stock = if daily_demand > 0.0 { qty / daily_demand } else { f64::INFINITY };

            let status = if qty <= reorder { "reorder" } else if days_of_stock < 7.0 { "low_stock" } else if days_of_stock > 90.0 { "overstock" } else { "optimal" };

            serde_json::json!({
                "sku": csv_util::record_get_str(i, "sku"),
                "name": csv_util::record_get_str(i, "name"),
                "quantity": qty, "reorder_point": reorder, "unit_cost": unit_cost,
                "days_of_stock": days_of_stock, "status": status,
                "inventory_value": qty * unit_cost,
            })
        }).collect();

        analysis.sort_by(|a, b| a["days_of_stock"].as_f64().unwrap_or(f64::INFINITY).partial_cmp(&b["days_of_stock"].as_f64().unwrap_or(f64::INFINITY)).unwrap_or(std::cmp::Ordering::Equal));

        let total_value: f64 = analysis.iter().map(|a| a["inventory_value"].as_f64().unwrap_or(0.0)).sum();
        let reorder_count = analysis.iter().filter(|a| a["status"] == "reorder").count();
        let overstock_count = analysis.iter().filter(|a| a["status"] == "overstock").count();

        let user_prompt = format!("Inventory ({} items, total value: {:.0}):\n- Reorder: {}, Overstock: {}\n\nItems:\n{}\n\nRecommendations?", items.len(), total_value, reorder_count, overstock_count, serde_json::to_string_pretty(&analysis).unwrap_or_default());
        let llm = ctx.call_llm("Analyze inventory and recommend optimizations.", &user_prompt).await?;
        Ok(serde_json::json!({ "summary": { "total_items": items.len(), "total_value": total_value, "reorder": reorder_count, "overstock": overstock_count }, "items": analysis, "llm_analysis": llm }))
    }
}
impl Default for InventoryOptimizerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Reorder SKU-A immediately.")) }

    #[tokio::test]
    async fn test_inventory() {
        let input = serde_json::json!({ "inventory": "sku,name,quantity,reorder_point,unit_cost,daily_demand\nSKU-A,Widget,5,10,5,2\nSKU-B,Gadget,200,10,5,1" });
        let r = InventoryOptimizerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["reorder"], 1);
        assert_eq!(r["summary"]["overstock"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(InventoryOptimizerAgent::new().execute_with_context(serde_json::json!({"inventory":""}), &ctx()).await.is_err()); }
    #[tokio::test]
    async fn test_llm() {
        let input = serde_json::json!({ "inventory": "sku,name,quantity,reorder_point,unit_cost,daily_demand\nA,B,10,5,1,1" });
        let r = InventoryOptimizerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert!(r["llm_analysis"].as_str().is_some());
    }
    #[test] fn test_manifest() { assert_eq!(InventoryOptimizerAgent::new().manifest().id, "operations.inventory-optimizer"); }
}