use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

pub struct SupplierComparisonAgent;

impl SupplierComparisonAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for SupplierComparisonAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.supplier-comparison".to_string(),
            version: "2.0.0".to_string(),
            name: "Supplier Comparison".to_string(),
            department: "Procurement".to_string(),
            description: "Weighted multi-criteria supplier comparison with LLM strategic analysis".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.compare".to_string(), "llm.analysis".to_string()],
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
        let suppliers = input["suppliers"]
            .as_array()
            .ok_or_else(|| AppError::Validation("Missing 'suppliers' array".to_string()))?;

        let w = input["weights"].as_object().cloned().unwrap_or_else(|| {
            serde_json::Map::from_iter(vec![
                ("price".to_string(), serde_json::json!(0.4)),
                ("quality".to_string(), serde_json::json!(0.3)),
                ("delivery".to_string(), serde_json::json!(0.2)),
                ("reliability".to_string(), serde_json::json!(0.1)),
            ])
        });

        let mut scored = Vec::new();
        for s in suppliers {
            let name = s["name"].as_str().unwrap_or("Unknown");
            let ps = s["price_score"].as_f64().unwrap_or(0.5);
            let qs = s["quality_score"].as_f64().unwrap_or(0.5);
            let ds = s["delivery_score"].as_f64().unwrap_or(0.5);
            let rs = s["reliability_score"].as_f64().unwrap_or(0.5);

            let weighted = ps * w.get("price").and_then(|v| v.as_f64()).unwrap_or(0.4)
                + qs * w.get("quality").and_then(|v| v.as_f64()).unwrap_or(0.3)
                + ds * w.get("delivery").and_then(|v| v.as_f64()).unwrap_or(0.2)
                + rs * w.get("reliability").and_then(|v| v.as_f64()).unwrap_or(0.1);

            let rec = if weighted >= 0.7 { "recommended" } else if weighted >= 0.4 { "consider" } else { "not_recommended" };

            scored.push(serde_json::json!({
                "name": name,
                "price_score": ps,
                "quality_score": qs,
                "delivery_score": ds,
                "reliability_score": rs,
                "weighted_score": weighted,
                "recommendation": rec,
            }));
        }

        scored.sort_by(|a, b| {
            b["weighted_score"].as_f64().unwrap_or(0.0)
                .partial_cmp(&a["weighted_score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best = scored.first().cloned();

        let system_prompt = "You are a supplier strategy analyst. Compare suppliers and provide recommendation with risk considerations. Be concise.";
        let user_prompt = format!(
            "Supplier Comparison ({} suppliers):\n{}\nBest: {}\n\nProvide strategic supplier recommendation.",
            scored.len(),
            serde_json::to_string_pretty(&scored).unwrap_or_default(),
            serde_json::to_string_pretty(&best).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "suppliers": scored,
            "best_supplier": best,
            "total_suppliers": scored.len(),
            "weights": w,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for SupplierComparisonAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Supplier A is best overall."))
    }

    #[tokio::test]
    async fn test_comparison() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "suppliers": [
                {"name": "A", "price_score": 0.8, "quality_score": 0.9, "delivery_score": 0.7, "reliability_score": 0.8},
                {"name": "B", "price_score": 0.9, "quality_score": 0.7, "delivery_score": 0.8, "reliability_score": 0.7}
            ]
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["total_suppliers"], 2);
        assert!(result["best_supplier"].is_object());
    }

    #[tokio::test]
    async fn test_custom_weights() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "suppliers": [
                {"name": "A", "price_score": 0.5, "quality_score": 0.9, "delivery_score": 0.5, "reliability_score": 0.5},
                {"name": "B", "price_score": 0.9, "quality_score": 0.5, "delivery_score": 0.5, "reliability_score": 0.5}
            ],
            "weights": {"price": 0.1, "quality": 0.8, "delivery": 0.05, "reliability": 0.05}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["best_supplier"]["name"], "A");
    }

    #[tokio::test]
    async fn test_missing_suppliers() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_single_supplier() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "suppliers": [{"name": "Sole", "price_score": 0.5, "quality_score": 0.5, "delivery_score": 0.5, "reliability_score": 0.5}]
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["total_suppliers"], 1);
    }

    #[tokio::test]
    async fn test_recommendation_labels() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "suppliers": [
                {"name": "Good", "price_score": 0.9, "quality_score": 0.9, "delivery_score": 0.9, "reliability_score": 0.9},
                {"name": "Bad", "price_score": 0.1, "quality_score": 0.1, "delivery_score": 0.1, "reliability_score": 0.1}
            ]
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let suppliers = result["suppliers"].as_array().unwrap();
        assert_eq!(suppliers[0]["recommendation"], "recommended");
        assert_eq!(suppliers[1]["recommendation"], "not_recommended");
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = SupplierComparisonAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "suppliers": [{"name": "A", "price_score": 0.5, "quality_score": 0.5, "delivery_score": 0.5, "reliability_score": 0.5}]
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = SupplierComparisonAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.supplier-comparison");
        assert!(m.permissions.network_llm);
    }
}