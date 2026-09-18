use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};

use crate::csv_util;

pub struct PricingOptimizationAgent;

impl PricingOptimizationAgent {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl Agent for PricingOptimizationAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.pricing-optimization".to_string(),
            version: "2.0.0".to_string(),
            name: "Pricing Optimization".to_string(),
            department: "Sales".to_string(),
            description: "Analyze pricing elasticity, discount impact, and optimize pricing strategies".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("products", "Products", InputKind::File, true).with_example("product,cost,price,list_price\nWidget,70,100,120"),
            ],
        }
    }

    fn supports_context(&self) -> bool { true }

    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let products_csv = input["products"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'products' CSV".to_string()))?;

        let products = csv_util::parse_csv_to_maps(products_csv)?;
        if products.is_empty() {
            return Err(AppError::Validation("No products found".to_string()));
        }

        let mut product_analysis: Vec<serde_json::Value> = products.iter().map(|p| {
            let base_price = csv_util::record_get_f64(p, "base_price");
            let current_price = csv_util::record_get_f64(p, "current_price");
            let units_sold = csv_util::record_get_f64(p, "units_sold");
            let competitor_price = csv_util::record_get_f64(p, "competitor_price");

            let discount_pct = if base_price > 0.0 { (1.0 - current_price / base_price) * 100.0 } else { 0.0 };
            let price_vs_competitor = if competitor_price > 0.0 { (current_price / competitor_price - 1.0) * 100.0 } else { 0.0 };
            let revenue = current_price * units_sold;

            let pricing_status = if price_vs_competitor > 10.0 { "premium" }
                else if price_vs_competitor < -10.0 { "discounted" }
                else { "competitive" };

            serde_json::json!({
                "name": csv_util::record_get_str(p, "name"),
                "base_price": base_price,
                "current_price": current_price,
                "units_sold": units_sold,
                "competitor_price": competitor_price,
                "discount_pct": format!("{:.1}%", discount_pct),
                "price_vs_competitor": format!("{:.1}%", price_vs_competitor),
                "revenue": revenue,
                "pricing_status": pricing_status,
            })
        }).collect();

        product_analysis.sort_by(|a, b| b["revenue"].as_f64().unwrap_or(0.0).partial_cmp(&a["revenue"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let total_revenue: f64 = product_analysis.iter().map(|p| p["revenue"].as_f64().unwrap_or(0.0)).sum();
        let avg_discount: f64 = products.iter().map(|p| {
            let base = csv_util::record_get_f64(p, "base_price");
            let current = csv_util::record_get_f64(p, "current_price");
            if base > 0.0 { (1.0 - current / base) * 100.0 } else { 0.0 }
        }).sum::<f64>() / products.len() as f64;

        let system_prompt = "You are a pricing strategist. Analyze pricing effectiveness and recommend optimizations. Be concise.";
        let user_prompt = format!(
            "Pricing Analysis ({} products, total revenue: {:.2}):\n- Avg discount: {:.1}%\n\nProducts:\n{}\n\nProvide pricing optimization recommendations.",
            products.len(), total_revenue, avg_discount,
            serde_json::to_string_pretty(&product_analysis).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_products": products.len(), "total_revenue": total_revenue, "avg_discount": format!("{:.1}%", avg_discount) },
            "products": product_analysis,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for PricingOptimizationAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Widget A is underpriced. Consider raising price 10%."))
    }

    #[tokio::test]
    async fn test_pricing() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "products": "name,base_price,current_price,units_sold,competitor_price\nWidget A,100,80,500,90\nWidget B,200,200,300,180"
        });
        let result = PricingOptimizationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total_products"], 2);
        assert!(result["summary"]["total_revenue"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_pricing_status() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "products": "name,base_price,current_price,units_sold,competitor_price\nPremium,100,120,100,100\nCheap,100,80,100,100\nFair,100,100,100,100"
        });
        let result = PricingOptimizationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        let products = result["products"].as_array().unwrap();
        let names: Vec<&str> = products.iter().map(|p| p["name"].as_str().unwrap()).collect();
        let statuses: Vec<&str> = products.iter().map(|p| p["pricing_status"].as_str().unwrap()).collect();
        let premium_idx = names.iter().position(|n| *n == "Premium").unwrap();
        let cheap_idx = names.iter().position(|n| *n == "Cheap").unwrap();
        let fair_idx = names.iter().position(|n| *n == "Fair").unwrap();
        assert_eq!(statuses[premium_idx], "premium");
        assert_eq!(statuses[cheap_idx], "discounted");
        assert_eq!(statuses[fair_idx], "competitive");
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(PricingOptimizationAgent::new().execute_with_context(serde_json::json!({ "products": "" }), &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "products": "name,base_price,current_price,units_sold,competitor_price\nA,100,100,100,100"
        });
        let result = PricingOptimizationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = PricingOptimizationAgent::new().manifest();
        assert_eq!(m.id, "sales.pricing-optimization");
        assert!(m.permissions.network_llm);
    }
}