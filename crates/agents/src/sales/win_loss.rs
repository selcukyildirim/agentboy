use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct WinLossAnalysisAgent;

impl WinLossAnalysisAgent {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl Agent for WinLossAnalysisAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.win-loss".to_string(),
            version: "2.0.0".to_string(),
            name: "Win/Loss Analysis".to_string(),
            department: "Sales".to_string(),
            description: "Analyze win/loss patterns, identify common loss reasons, and recommend improvements".to_string(),
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
        let deals_csv = input["deals"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'deals' CSV".to_string()))?;

        let deals = csv_util::parse_csv_to_maps(deals_csv)?;
        if deals.is_empty() {
            return Err(AppError::Validation("No deals found".to_string()));
        }

        let won: Vec<_> = deals.iter().filter(|r| csv_util::record_get_str(r, "stage") == "won").collect();
        let lost: Vec<_> = deals.iter().filter(|r| csv_util::record_get_str(r, "stage") == "lost").collect();

        let won_value: f64 = won.iter().map(|r| csv_util::record_get_f64(r, "value")).sum();
        let lost_value: f64 = lost.iter().map(|r| csv_util::record_get_f64(r, "value")).sum();

        let mut loss_reasons = std::collections::HashMap::new();
        for deal in &lost {
            let reason = csv_util::record_get_str(deal, "loss_reason");
            if !reason.is_empty() {
                *loss_reasons.entry(reason).or_insert(0u64) += 1;
            }
        }

        let mut loss_summary: Vec<serde_json::Value> = loss_reasons.iter().map(|(reason, count)| {
            serde_json::json!({ "reason": reason, "count": count })
        }).collect();
        loss_summary.sort_by(|a, b| b["count"].as_u64().unwrap_or(0).partial_cmp(&a["count"].as_u64().unwrap_or(0)).unwrap_or(std::cmp::Ordering::Equal));

        let system_prompt = "You are a sales win/loss analyst. Analyze patterns and recommend sales improvements. Be concise.";
        let user_prompt = format!(
            "Win/Loss Analysis ({} deals):\n- Won: {} deals ({:.2})\n- Lost: {} deals ({:.2})\n- Win rate: {:.1}%\n\nLoss Reasons:\n{}\n\nProvide improvement recommendations.",
            deals.len(), won.len(), won_value, lost.len(), lost_value,
            if !deals.is_empty() { won.len() as f64 / deals.len() as f64 * 100.0 } else { 0.0 },
            serde_json::to_string_pretty(&loss_summary).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_deals": deals.len(),
                "won_count": won.len(),
                "lost_count": lost.len(),
                "won_value": won_value,
                "lost_value": lost_value,
            },
            "loss_reasons": loss_summary,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for WinLossAnalysisAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Price is the top loss reason. Improve value proposition."))
    }

    #[tokio::test]
    async fn test_win_loss() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,stage,loss_reason\nD1,10000,won,\nD2,20000,won,\nD3,15000,lost,Price\nD4,5000,lost,Price\nD6,8000,lost,Feature"
        });
        let result = WinLossAnalysisAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["won_count"], 2);
        assert_eq!(result["summary"]["lost_count"], 3);
        assert!(result["summary"]["won_value"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_loss_reasons() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,stage,loss_reason\nD1,1000,lost,Price\nD2,2000,lost,Price\nD3,3000,lost,Feature"
        });
        let result = WinLossAnalysisAgent::new().execute_with_context(input, &ctx).await.unwrap();
        let reasons = result["loss_reasons"].as_array().unwrap();
        assert_eq!(reasons[0]["reason"], "Price");
        assert_eq!(reasons[0]["count"], 2);
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(WinLossAnalysisAgent::new().execute_with_context(serde_json::json!({ "deals": "" }), &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,stage,loss_reason\nD1,1000,won,\nD2,2000,lost,X"
        });
        let result = WinLossAnalysisAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = WinLossAnalysisAgent::new().manifest();
        assert_eq!(m.id, "sales.win-loss");
        assert!(m.permissions.network_llm);
    }
}