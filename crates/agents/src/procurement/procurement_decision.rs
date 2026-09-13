use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

pub struct ProcurementDecisionAgent;

impl ProcurementDecisionAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for ProcurementDecisionAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.decision".to_string(),
            version: "2.0.0".to_string(),
            name: "Procurement Decision".to_string(),
            department: "Procurement".to_string(),
            description: "Make evidence-backed procurement decisions from vendor quotes, budget, and policies with LLM analysis".to_string(),
            tier: AgentTier::Free,
            skills: vec!["decision.analyze".to_string(), "spreadsheet.compare".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 40,
                timeout_seconds: 180,
            },
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
        let vendor_quotes = input["vendor_quotes"]
            .as_array()
            .ok_or_else(|| AppError::Validation("Missing 'vendor_quotes' array".to_string()))?;
        let budget_limit = input["budget_limit"]
            .as_f64()
            .ok_or_else(|| AppError::Validation("Missing 'budget_limit'".to_string()))?;

        let mut valid_quotes: Vec<&serde_json::Value> = vendor_quotes
            .iter()
            .filter(|q| {
                q.get("price")
                    .and_then(|p| p.as_f64())
                    .map(|price| price <= budget_limit)
                    .unwrap_or(false)
            })
            .collect();

        valid_quotes.sort_by(|a, b| {
            let pa = a.get("price").and_then(|p| p.as_f64()).unwrap_or(f64::MAX);
            let pb = b.get("price").and_then(|p| p.as_f64()).unwrap_or(f64::MAX);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let decision = if valid_quotes.is_empty() {
            serde_json::json!({
                "decision": "no_viable_option",
                "reason": "No vendor quotes within budget",
                "recommendations": ["Increase budget", "Negotiate with vendors", "Consider alternatives"],
            })
        } else {
            let best = valid_quotes[0];
            let best_price = best.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
            let vendor_name = best.get("vendor").and_then(|v| v.as_str()).unwrap_or("Unknown");

            let alternatives: Vec<serde_json::Value> = valid_quotes[1..].iter().map(|q| {
                serde_json::json!({
                    "vendor": q.get("vendor").and_then(|v| v.as_str()).unwrap_or("Unknown"),
                    "price": q.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0),
                })
            }).collect();

            serde_json::json!({
                "decision": "proceed_with_best",
                "recommended_vendor": vendor_name,
                "recommended_price": best_price,
                "budget_utilization": format!("{:.1}%", (best_price / budget_limit) * 100.0),
                "savings_from_budget": budget_limit - best_price,
                "alternatives": alternatives,
            })
        };

        let system_prompt = "You are a procurement advisor. Analyze the vendor selection and provide strategic recommendations. Be concise.";
        let user_prompt = format!(
            "Procurement Decision:\n- Budget: {:.2}\n- Vendor quotes: {}\n- Decision: {}\n\nVendor Details:\n{}\n\nProvide procurement strategy advice.",
            budget_limit, vendor_quotes.len(),
            serde_json::to_string_pretty(&decision).unwrap_or_default(),
            serde_json::to_string_pretty(vendor_quotes).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "recommendation": decision,
            "vendor_count": vendor_quotes.len(),
            "budget_limit": budget_limit,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for ProcurementDecisionAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Supplier A offers the best value. Recommend proceeding."))
    }

    #[tokio::test]
    async fn test_best_vendor() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendor_quotes": [
                {"vendor": "A", "price": 800},
                {"vendor": "B", "price": 950},
                {"vendor": "C", "price": 1100}
            ],
            "budget_limit": 1000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["recommendation"]["decision"], "proceed_with_best");
        assert_eq!(result["recommendation"]["recommended_vendor"], "A");
    }

    #[tokio::test]
    async fn test_over_budget() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendor_quotes": [
                {"vendor": "A", "price": 1500},
                {"vendor": "B", "price": 2000}
            ],
            "budget_limit": 1000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["recommendation"]["decision"], "no_viable_option");
    }

    #[tokio::test]
    async fn test_missing_quotes() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "budget_limit": 1000 });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_missing_budget() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "vendor_quotes": [{"vendor":"A","price":100}] });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_savings_calculation() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendor_quotes": [{"vendor": "A", "price": 700}],
            "budget_limit": 1000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["recommendation"]["savings_from_budget"], 300.0);
    }

    #[tokio::test]
    async fn test_alternatives() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendor_quotes": [
                {"vendor": "A", "price": 800},
                {"vendor": "B", "price": 900},
                {"vendor": "C", "price": 950}
            ],
            "budget_limit": 1000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["recommendation"]["alternatives"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = ProcurementDecisionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendor_quotes": [{"vendor": "A", "price": 100}],
            "budget_limit": 500
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = ProcurementDecisionAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.decision");
        assert!(m.permissions.network_llm);
    }
}