use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct RiskRegisterAgent;
impl RiskRegisterAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for RiskRegisterAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "pmo.risk-register".to_string(),
            version: "2.0.0".to_string(),
            name: "Risk Register".to_string(),
            department: "PMO".to_string(),
            description: "Maintain and analyze project risk register, track mitigation status"
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
            input_schema: vec![InputField::new("risks", "Risks", InputKind::File, true)
                .with_example("risk_id,description,probability,impact\nR1,Vendor delay,0.6,0.8")],
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
        let csv = input["risks"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'risks' CSV".to_string()))?;
        let risks = csv_util::parse_csv_to_maps(csv)?;
        if risks.is_empty() {
            return Err(AppError::Validation("No risks".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = risks.iter().map(|r| {
            let prob = csv_util::record_get_f64(r, "probability");
            let impact = csv_util::record_get_f64(r, "impact");
            let score = prob * impact;
            let level = if score >= 0.6 { "high" } else if score >= 0.3 { "medium" } else { "low" };
            let mitigation = csv_util::record_get_str(r, "mitigation_status");
            let status = if mitigation.is_empty() || mitigation == "none" { "unmitigated" } else { mitigation };
            serde_json::json!({ "risk_id": csv_util::record_get_str(r, "risk_id"), "description": csv_util::record_get_str(r, "description"), "project": csv_util::record_get_str(r, "project"), "probability": prob, "impact": impact, "score": score, "level": level, "mitigation_status": status })
        }).collect();
        analysis.sort_by(|a, b| {
            b["score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let high = analysis.iter().filter(|a| a["level"] == "high").count();
        let unmitigated = analysis
            .iter()
            .filter(|a| a["mitigation_status"] == "unmitigated")
            .count();

        let llm = ctx
            .call_llm(
                "Analyze risk register.",
                &format!(
                    "Risks ({} total, {} high, {} unmitigated):\n{}\n\nPriorities?",
                    risks.len(),
                    high,
                    unmitigated,
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_risks": risks.len(), "high": high, "unmitigated": unmitigated }, "risks": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for RiskRegisterAgent {
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
            "Risk R1 is unmitigated and high.",
        ))
    }

    #[tokio::test]
    async fn test_risks() {
        let input = serde_json::json!({ "risks": "risk_id,description,project,probability,impact,mitigation_status\nR1,Vendor delay,Alpha,0.8,0.9,\nR2,Scope creep,Alpha,0.4,0.5,in_progress" });
        let r = RiskRegisterAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["high"], 1);
        assert_eq!(r["summary"]["unmitigated"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(RiskRegisterAgent::new()
            .execute_with_context(serde_json::json!({"risks":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(RiskRegisterAgent::new().manifest().id, "pmo.risk-register");
    }
}
