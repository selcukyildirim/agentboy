use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct RegulatoryMonitorAgent;
impl RegulatoryMonitorAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for RegulatoryMonitorAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "legal.regulatory-monitor".to_string(), version: "2.0.0".to_string(), name: "Regulatory Monitor".to_string(), department: "Legal".to_string(), description: "Monitor regulatory changes, assess impact on business, and track compliance deadlines".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["regulations"].as_str().ok_or_else(|| AppError::Validation("Missing 'regulations' CSV".to_string()))?;
        let regs = csv_util::parse_csv_to_maps(csv)?;
        if regs.is_empty() { return Err(AppError::Validation("No regulations".to_string())); }

        let mut analysis: Vec<serde_json::Value> = regs.iter().map(|r| {
            let impact = csv_util::record_get_f64(r, "impact_score");
            let days_to_comply = csv_util::record_get_f64(r, "days_to_comply");
            let status = csv_util::record_get_str(r, "compliance_status");
            let urgency = if days_to_comply < 30.0 && impact > 0.7 { "critical" } else if days_to_comply < 90.0 { "action_needed" } else { "monitoring" };
            serde_json::json!({ "regulation": csv_util::record_get_str(r, "regulation"), "jurisdiction": csv_util::record_get_str(r, "jurisdiction"), "impact_score": impact, "days_to_comply": days_to_comply, "compliance_status": status, "urgency": urgency })
        }).collect();
        analysis.sort_by(|a, b| a["days_to_comply"].as_f64().unwrap_or(f64::INFINITY).partial_cmp(&b["days_to_comply"].as_f64().unwrap_or(f64::INFINITY)).unwrap_or(std::cmp::Ordering::Equal));

        let critical = analysis.iter().filter(|a| a["urgency"] == "critical").count();
        let non_compliant = analysis.iter().filter(|a| a["compliance_status"] != "compliant").count();

        let llm = ctx.call_llm("Monitor regulations.", &format!("Regulations ({} total, {} critical, {} non-compliant):\n{}\n\nActions?", regs.len(), critical, non_compliant, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_regulations": regs.len(), "critical": critical, "non_compliant": non_compliant }, "regulations": analysis, "llm_analysis": llm }))
    }
}
impl Default for RegulatoryMonitorAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("GDPR update is critical.")) }

    #[tokio::test]
    async fn test_regulations() {
        let input = serde_json::json!({ "regulations": "regulation,jurisdiction,impact_score,days_to_comply,compliance_status\nGDPR Update,EU,0.9,20,non_compliant\nSOX,US,0.5,120,compliant" });
        let r = RegulatoryMonitorAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["critical"], 1);
        assert_eq!(r["summary"]["non_compliant"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(RegulatoryMonitorAgent::new().execute_with_context(serde_json::json!({"regulations":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(RegulatoryMonitorAgent::new().manifest().id, "legal.regulatory-monitor"); }
}