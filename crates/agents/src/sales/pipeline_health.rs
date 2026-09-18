use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct PipelineHealthAgent;

impl PipelineHealthAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for PipelineHealthAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.pipeline-health".to_string(),
            version: "2.0.0".to_string(),
            name: "Pipeline Health".to_string(),
            department: "Sales".to_string(),
            description: "Analyze pipeline velocity, stage conversion, and identify bottlenecks"
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
            input_schema: vec![InputField::new("deals", "Deals", InputKind::File, true)
                .with_example(
                    "deal_id,account,stage,amount,probability\nD1,Acme,Proposal,50000,60",
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
        let deals_csv = input["deals"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'deals' CSV".to_string()))?;

        let deals = csv_util::parse_csv_to_maps(deals_csv)?;
        if deals.is_empty() {
            return Err(AppError::Validation("No deals found".to_string()));
        }

        let stages = vec![
            "discovery",
            "qualification",
            "proposal",
            "negotiation",
            "won",
            "lost",
        ];
        let mut stage_counts = std::collections::HashMap::new();
        let mut stage_values = std::collections::HashMap::new();

        for deal in &deals {
            let stage = csv_util::record_get_str(deal, "stage");
            let value = csv_util::record_get_f64(deal, "value");
            *stage_counts.entry(stage.to_string()).or_insert(0u64) += 1;
            *stage_values.entry(stage.to_string()).or_insert(0.0) += value;
        }

        let stage_summary: Vec<serde_json::Value> = stages
            .iter()
            .filter_map(|s| {
                let count = stage_counts.get(*s).copied().unwrap_or(0);
                let value = stage_values.get(*s).copied().unwrap_or(0.0);
                if count == 0 {
                    return None;
                }
                Some(serde_json::json!({
                    "stage": s,
                    "count": count,
                    "value": value,
                }))
            })
            .collect();

        let discovery = stage_counts.get("discovery").copied().unwrap_or(0);
        let qualification = stage_counts.get("qualification").copied().unwrap_or(0);
        let proposal = stage_counts.get("proposal").copied().unwrap_or(0);
        let won = stage_counts.get("won").copied().unwrap_or(0);
        let lost = stage_counts.get("lost").copied().unwrap_or(0);

        let discovery_to_qual = if discovery > 0 {
            qualification as f64 / discovery as f64
        } else {
            0.0
        };
        let qual_to_proposal = if qualification > 0 {
            proposal as f64 / qualification as f64
        } else {
            0.0
        };
        let close_rate = if (won + lost) > 0 {
            won as f64 / (won + lost) as f64
        } else {
            0.0
        };

        let system_prompt = "You are a sales pipeline analyst. Analyze pipeline health and identify bottlenecks. Be concise.";
        let user_prompt = format!(
            "Pipeline Health ({} deals):\n- Stage breakdown: {:?}\n- Discovery→Qualification: {:.1}%\n- Qualification→Proposal: {:.1}%\n- Close rate: {:.1}%\n\nProvide pipeline health assessment.",
            deals.len(), stage_summary, discovery_to_qual * 100.0, qual_to_proposal * 100.0, close_rate * 100.0,
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_deals": deals.len(),
                "discovery_to_qualification": format!("{:.1}%", discovery_to_qual * 100.0),
                "qualification_to_proposal": format!("{:.1}%", qual_to_proposal * 100.0),
                "close_rate": format!("{:.1}%", close_rate * 100.0),
            },
            "stages": stage_summary,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for PipelineHealthAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Pipeline is well balanced."))
    }

    #[tokio::test]
    async fn test_pipeline_health() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,stage\nD1,10000,discovery\nD2,20000,qualification\nD3,50000,won\nD4,15000,lost"
        });
        let result = PipelineHealthAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_deals"], 4);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_empty_deals() {
        let ctx = make_ctx();
        assert!(PipelineHealthAgent::new()
            .execute_with_context(serde_json::json!({ "deals": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_close_rate() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "deals": "deal_id,value,stage\nD1,10000,won\nD2,20000,won\nD3,15000,lost"
        });
        let result = PipelineHealthAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["close_rate"], "66.7%");
    }

    #[tokio::test]
    async fn test_manifest() {
        let m = PipelineHealthAgent::new().manifest();
        assert_eq!(m.id, "sales.pipeline-health");
        assert!(m.permissions.network_llm);
    }
}
