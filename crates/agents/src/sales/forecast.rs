use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct SalesForecastAgent;

impl SalesForecastAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for SalesForecastAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.forecast".to_string(),
            version: "2.0.0".to_string(),
            name: "Sales Forecast".to_string(),
            department: "Sales".to_string(),
            description: "Forecast sales pipeline, revenue projections, and win probability with LLM insights".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("deals", "Deals", InputKind::File, true).with_example("deal_id,account,stage,amount,probability\nD1,Acme,Proposal,50000,60"),
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
        let deals_csv = input["deals"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'deals' CSV".to_string()))?;

        let deals = csv_util::parse_csv_to_maps(deals_csv)?;
        if deals.is_empty() {
            return Err(AppError::Validation("No deals found".to_string()));
        }

        let total_value: f64 = deals
            .iter()
            .map(|r| csv_util::record_get_f64(r, "value"))
            .sum();
        let weighted_value: f64 = deals
            .iter()
            .map(|r| {
                let value = csv_util::record_get_f64(r, "value");
                let probability = csv_util::record_get_f64(r, "probability");
                value * probability
            })
            .sum();

        let won: Vec<_> = deals
            .iter()
            .filter(|r| csv_util::record_get_str(r, "stage") == "won")
            .collect();
        let lost: Vec<_> = deals
            .iter()
            .filter(|r| csv_util::record_get_str(r, "stage") == "lost")
            .collect();
        let active: Vec<_> = deals
            .iter()
            .filter(|r| {
                let s = csv_util::record_get_str(r, "stage");
                s != "won" && s != "lost"
            })
            .collect();

        let win_rate = if !won.is_empty() || !lost.is_empty() {
            won.len() as f64 / (won.len() + lost.len()) as f64
        } else {
            0.0
        };

        let avg_deal_size = if deals.is_empty() {
            0.0
        } else {
            total_value / deals.len() as f64
        };
        let avg_win_deal = if won.is_empty() {
            0.0
        } else {
            won.iter()
                .map(|r| csv_util::record_get_f64(r, "value"))
                .sum::<f64>()
                / won.len() as f64
        };

        let system_prompt = "You are a sales forecast analyst. Analyze pipeline and provide revenue forecast. Be concise.";
        let user_prompt = format!(
            "Sales Forecast:\n- Total pipeline: {:.2}\n- Weighted forecast: {:.2}\n- Win rate: {:.1}%\n- Active deals: {}\n- Won: {}, Lost: {}\n- Avg deal: {:.2}, Avg won deal: {:.2}\n\nProvide forecast and pipeline recommendations.",
            total_value, weighted_value, win_rate * 100.0, active.len(), won.len(), lost.len(), avg_deal_size, avg_win_deal,
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_pipeline_value": total_value,
                "weighted_forecast": weighted_value,
                "win_rate": format!("{:.1}%", win_rate * 100.0),
                "active_deals": active.len(),
                "won_deals": won.len(),
                "lost_deals": lost.len(),
                "avg_deal_size": avg_deal_size,
                "avg_win_deal": avg_win_deal,
            },
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for SalesForecastAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "Pipeline is healthy. Expect 80% of weighted forecast.",
        ))
    }

    #[tokio::test]
    async fn test_forecast() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,probability,stage\nD1,10000,0.7,proposal\nD2,5000,0.3,discovery\nD3,20000,1.0,won\nD4,8000,0.0,lost"
        });
        let result = SalesForecastAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["won_deals"], 1);
        assert_eq!(result["summary"]["lost_deals"], 1);
        assert!(result["summary"]["weighted_forecast"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_no_deals() {
        let ctx = make_ctx();
        let input = serde_json::json!({ "deals": "" });
        assert!(SalesForecastAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_missing_input() {
        let ctx = make_ctx();
        assert!(SalesForecastAgent::new()
            .execute_with_context(serde_json::json!({}), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_win_rate() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,probability,stage\nD1,10000,0.5,won\nD2,5000,0.5,won\nD3,8000,0.5,lost"
        });
        let result = SalesForecastAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["win_rate"], "66.7%");
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,probability,stage\nD1,1000,0.5,proposal"
        });
        let result = SalesForecastAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = SalesForecastAgent::new().manifest();
        assert_eq!(m.id, "sales.forecast");
        assert!(m.permissions.network_llm);
    }
}
