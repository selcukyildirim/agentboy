use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct LeadScoringAgent;

impl LeadScoringAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for LeadScoringAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.lead-scoring".to_string(),
            version: "2.0.0".to_string(),
            name: "Lead Scoring".to_string(),
            department: "Sales".to_string(),
            description: "Score and rank leads based on engagement, fit, and conversion signals"
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
            input_schema: vec![InputField::new("leads", "Leads", InputKind::File, true)
                .with_example("lead_id,company,score\nL1,Acme,85")],
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
        let leads_csv = input["leads"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'leads' CSV".to_string()))?;

        let leads = csv_util::parse_csv_to_maps(leads_csv)?;
        if leads.is_empty() {
            return Err(AppError::Validation("No leads found".to_string()));
        }

        let mut scored: Vec<serde_json::Value> = leads
            .iter()
            .map(|lead| {
                let engagement = csv_util::record_get_f64(lead, "engagement_score");
                let fit = csv_util::record_get_f64(lead, "fit_score");
                let recency = csv_util::record_get_f64(lead, "recency_score");
                let composite = recency.mul_add(0.25, engagement.mul_add(0.4, fit * 0.35));
                let tier = if composite >= 0.8 {
                    "hot"
                } else if composite >= 0.5 {
                    "warm"
                } else {
                    "cold"
                };

                serde_json::json!({
                    "name": csv_util::record_get_str(lead, "name"),
                    "company": csv_util::record_get_str(lead, "company"),
                    "engagement_score": engagement,
                    "fit_score": fit,
                    "recency_score": recency,
                    "composite_score": composite,
                    "tier": tier,
                })
            })
            .collect();

        scored.sort_by(|a, b| {
            b["composite_score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["composite_score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let hot = scored.iter().filter(|s| s["tier"] == "hot").count();
        let warm = scored.iter().filter(|s| s["tier"] == "warm").count();
        let cold = scored.iter().filter(|s| s["tier"] == "cold").count();

        let system_prompt = "You are a sales lead analyst. Analyze lead scores and prioritize outreach. Be concise.";
        let user_prompt = format!(
            "Lead Scoring ({} leads):\n- Hot: {}, Warm: {}, Cold: {}\n\nTop 5 leads:\n{}\n\nProvide prioritization and outreach strategy.",
            leads.len(), hot, warm, cold,
            serde_json::to_string_pretty(&scored[..scored.len().min(5)]).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total": leads.len(), "hot": hot, "warm": warm, "cold": cold },
            "leads": scored,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for LeadScoringAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Focus on hot leads first."))
    }

    #[tokio::test]
    async fn test_scoring() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "leads": "name,company,engagement_score,fit_score,recency_score\nAlice,Acme,0.9,0.8,0.9\nBob,Beta,0.3,0.4,0.2"
        });
        let result = LeadScoringAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total"], 2);
        assert!(result["summary"]["hot"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_empty_leads() {
        let ctx = make_ctx();
        assert!(LeadScoringAgent::new()
            .execute_with_context(serde_json::json!({ "leads": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_tiers() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "leads": "name,company,engagement_score,fit_score,recency_score\nA,X,1.0,1.0,1.0\nB,Y,0.1,0.1,0.1\nC,Z,0.5,0.5,0.5"
        });
        let result = LeadScoringAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        let leads = result["leads"].as_array().unwrap();
        assert_eq!(leads[0]["tier"], "hot");
        assert_eq!(leads[2]["tier"], "cold");
    }

    #[tokio::test]
    async fn test_sorted() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "leads": "name,company,engagement_score,fit_score,recency_score\nA,X,0.2,0.2,0.2\nB,Y,0.9,0.9,0.9"
        });
        let result = LeadScoringAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        let leads = result["leads"].as_array().unwrap();
        assert!(
            leads[0]["composite_score"].as_f64().unwrap()
                >= leads[1]["composite_score"].as_f64().unwrap()
        );
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "leads": "name,company,engagement_score,fit_score,recency_score\nA,X,0.5,0.5,0.5"
        });
        let result = LeadScoringAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = LeadScoringAgent::new().manifest();
        assert_eq!(m.id, "sales.lead-scoring");
        assert!(m.permissions.network_llm);
    }
}
