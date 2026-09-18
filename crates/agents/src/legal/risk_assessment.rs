use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct LegalRiskAssessmentAgent;
impl LegalRiskAssessmentAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for LegalRiskAssessmentAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "legal.risk-assessment".to_string(),
            version: "2.0.0".to_string(),
            name: "Risk Assessment".to_string(),
            department: "Legal".to_string(),
            description:
                "Assess legal exposure across business activities, contracts, and regulatory areas"
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
            let likelihood = csv_util::record_get_f64(r, "likelihood");
            let impact = csv_util::record_get_f64(r, "impact_millions");
            let exposure = likelihood * impact;
            let level = if exposure > 5.0 { "critical" } else if exposure > 1.0 { "high" } else if exposure > 0.1 { "medium" } else { "low" };
            serde_json::json!({ "area": csv_util::record_get_str(r, "area"), "description": csv_util::record_get_str(r, "description"), "likelihood": likelihood, "impact_millions": impact, "exposure": exposure, "level": level })
        }).collect();
        analysis.sort_by(|a, b| {
            b["exposure"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["exposure"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_exposure: f64 = analysis
            .iter()
            .map(|a| a["exposure"].as_f64().unwrap_or(0.0))
            .sum();
        let critical = analysis.iter().filter(|a| a["level"] == "critical").count();

        let llm = ctx
            .call_llm(
                "Assess legal risks.",
                &format!(
                    "Legal Risks ({} total, exposure: {:.1}M, {} critical):\n{}\n\nMitigations?",
                    risks.len(),
                    total_exposure,
                    critical,
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_risks": risks.len(), "total_exposure_millions": total_exposure, "critical": critical }, "risks": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for LegalRiskAssessmentAgent {
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
            "IP litigation is critical exposure.",
        ))
    }

    #[tokio::test]
    async fn test_legal_risk() {
        let input = serde_json::json!({ "risks": "area,description,likelihood,impact_millions\nIP,Potential infringement,0.5,20\nContract,Breach risk,0.3,2" });
        let r = LegalRiskAssessmentAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["critical"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(LegalRiskAssessmentAgent::new()
            .execute_with_context(serde_json::json!({"risks":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            LegalRiskAssessmentAgent::new().manifest().id,
            "legal.risk-assessment"
        );
    }
}
