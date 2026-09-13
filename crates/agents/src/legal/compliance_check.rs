use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct ComplianceCheckAgent;
impl ComplianceCheckAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for ComplianceCheckAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "legal.compliance-check".to_string(), version: "2.0.0".to_string(), name: "Compliance Check".to_string(), department: "Legal".to_string(), description: "Verify regulatory compliance across policies, procedures, and filings".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "document.validate".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["requirements"].as_str().ok_or_else(|| AppError::Validation("Missing 'requirements' CSV".to_string()))?;
        let reqs = csv_util::parse_csv_to_maps(csv)?;
        if reqs.is_empty() { return Err(AppError::Validation("No requirements".to_string())); }

        let mut analysis: Vec<serde_json::Value> = reqs.iter().map(|r| {
            let met = csv_util::record_get_str(r, "status") == "met";
            let risk = csv_util::record_get_f64(r, "risk_score");
            let status = if met { "compliant" } else if risk > 0.7 { "critical_gap" } else { "gap" };
            serde_json::json!({ "requirement": csv_util::record_get_str(r, "requirement"), "regulation": csv_util::record_get_str(r, "regulation"), "status": status, "risk_score": risk })
        }).collect();

        let compliant = analysis.iter().filter(|a| a["status"] == "compliant").count();
        let gaps = analysis.iter().filter(|a| a["status"] != "compliant").count();
        let critical = analysis.iter().filter(|a| a["status"] == "critical_gap").count();

        let llm = ctx.call_llm("Assess compliance.", &format!("Compliance ({} requirements, {} compliant, {} gaps, {} critical):\n{}\n\nActions?", reqs.len(), compliant, gaps, critical, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total": reqs.len(), "compliant": compliant, "gaps": gaps, "critical": critical }, "requirements": analysis, "llm_analysis": llm }))
    }
}
impl Default for ComplianceCheckAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("GDPR requirement is critical.")) }

    #[tokio::test]
    async fn test_compliance() {
        let input = serde_json::json!({ "requirements": "requirement,regulation,status,risk_score\nData Retention,GDPR,met,0.2\nConsent Mgmt,GDPR,,0.9\nSOX Controls,SOX,met,0.3" });
        let r = ComplianceCheckAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["compliant"], 2);
        assert_eq!(r["summary"]["critical"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(ComplianceCheckAgent::new().execute_with_context(serde_json::json!({"requirements":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(ComplianceCheckAgent::new().manifest().id, "legal.compliance-check"); }
}