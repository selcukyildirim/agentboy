use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};

use crate::csv_util;

pub struct PriceHistoryAgent;

impl PriceHistoryAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for PriceHistoryAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.price-history".to_string(),
            version: "2.0.0".to_string(),
            name: "Price History".to_string(),
            department: "Procurement".to_string(),
            description: "Analyze historical price trends, detect volatility and significant changes with LLM insights".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
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
            input_schema: vec![
                InputField::new("prices", "Price History", InputKind::File, true),
            ],
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
        let prices_csv = input["prices"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'prices' CSV".to_string()))?;

        let records = csv_util::parse_csv_to_maps(prices_csv)?;
        if records.is_empty() {
            return Err(AppError::Validation("No price records found".to_string()));
        }

        let price_values: Vec<f64> = records.iter().map(|r| csv_util::record_get_f64(r, "price")).collect();
        let current_price = price_values.last().copied().unwrap_or(0.0);
        let avg_price = if !price_values.is_empty() {
            price_values.iter().sum::<f64>() / price_values.len() as f64
        } else {
            0.0
        };
        let min_price = price_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = price_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let first_price = price_values.first().copied().unwrap_or(0.0);
        let total_change_pct = if first_price != 0.0 {
            (current_price - first_price) / first_price * 100.0
        } else {
            0.0
        };

        let trend = if total_change_pct > 5.0 {
            "increasing"
        } else if total_change_pct < -5.0 {
            "decreasing"
        } else {
            "stable"
        };

        let mut significant_changes = Vec::new();
        for window in records.windows(2) {
            let prev = csv_util::record_get_f64(&window[0], "price");
            let curr = csv_util::record_get_f64(&window[1], "price");
            if prev != 0.0 {
                let change_pct = (curr - prev) / prev;
                if change_pct.abs() > 0.1 {
                    significant_changes.push(serde_json::json!({
                        "from_date": csv_util::record_get_str(&window[0], "date"),
                        "to_date": csv_util::record_get_str(&window[1], "date"),
                        "from_price": prev,
                        "to_price": curr,
                        "change_pct": format!("{:.1}%", change_pct * 100.0),
                        "direction": if change_pct > 0.0 { "increase" } else { "decrease" },
                    }));
                }
            }
        }

        let std_dev = csv_util::standard_deviation(&price_values);
        let cv = if avg_price > 0.0 { std_dev / avg_price } else { 0.0 };

        let system_prompt = "You are a procurement price analyst. Analyze price trends and provide purchasing recommendations. Be concise.";
        let user_prompt = format!(
            "Price History Analysis:\n- Data points: {}\n- Current: {:.2}, Avg: {:.2}, Min: {:.2}, Max: {:.2}\n- Trend: {} ({:.1}% change)\n- Volatility (CV): {:.1}%\n- Significant changes: {}\n\nProvide price analysis and procurement recommendation.",
            price_values.len(), current_price, avg_price, min_price, max_price,
            trend, total_change_pct, cv * 100.0, significant_changes.len(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "data_points": price_values.len(),
                "current_price": current_price,
                "average_price": avg_price,
                "min_price": min_price,
                "max_price": max_price,
                "total_change_pct": format!("{:.1}%", total_change_pct),
                "trend": trend,
                "volatility_cv": format!("{:.1}%", cv * 100.0),
                "significant_changes": significant_changes.len(),
            },
            "significant_changes": significant_changes,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for PriceHistoryAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Prices are trending upward. Consider locking in current rates."))
    }

    #[tokio::test]
    async fn test_price_history() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "prices": "date,price,supplier\n2024-01-01,100,A\n2024-02-01,105,A\n2024-03-01,95,A"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["data_points"], 3);
        assert!(result["summary"]["current_price"].as_f64().is_some());
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_increasing_trend() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "prices": "date,price,supplier\n2024-01-01,100,A\n2024-02-01,110,A\n2024-03-01,120,A"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["trend"], "increasing");
    }

    #[tokio::test]
    async fn test_decreasing_trend() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "prices": "date,price,supplier\n2024-01-01,100,A\n2024-02-01,90,A\n2024-03-01,80,A"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["trend"], "decreasing");
    }

    #[tokio::test]
    async fn test_significant_changes() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "prices": "date,price,supplier\n2024-01-01,100,A\n2024-02-01,150,A"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["significant_changes"], 1);
    }

    #[tokio::test]
    async fn test_missing_prices() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_prices() {
        let agent = PriceHistoryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "prices": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[test]
    fn test_manifest() {
        let agent = PriceHistoryAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.price-history");
        assert!(m.permissions.network_llm);
    }
}