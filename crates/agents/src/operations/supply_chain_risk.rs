use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct SupplyChainRiskAgent;
impl SupplyChainRiskAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for SupplyChainRiskAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "operations.supply-chain-risk".to_string(), version: "2.0.0".to_string(), name: "Supply Chain Risk".to_string(), department: "Operations".to_string(), description: "Identify supply chain vulnerabilities, single-source risks, and recommend mitigation".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["suppliers"].as_str().ok_or_else(|| AppError::Validation("Missing 'suppliers' CSV".to_string()))?;
        let suppliers = csv_util::parse_csv_to_maps(csv)?;
        if suppliers.is_empty() { return Err(AppError::Validation("No suppliers".to_string())); }

        let by_component = csv_util::group_by(&suppliers, "component");
        let mut risks: Vec<serde_json::Value> = by_component.iter().map(|(comp, sups)| {
            let single_source = sups.len() == 1;
            let lead_times: Vec<f64> = sups.iter().filter_map(|s| s.get("lead_time_days").and_then(|v| v.parse::<f64>().ok())).collect();
            let avg_lead = if !lead_times.is_empty() { lead_times.iter().sum::<f64>() / lead_times.len() as f64 } else { 0.0 };
            let risk_level = if single_source && avg_lead > 30.0 { "high" } else if single_source || avg_lead > 30.0 { "medium" } else { "low" };
            serde_json::json!({ "component": comp, "supplier_count": sups.len(), "single_source": single_source, "avg_lead_time_days": avg_lead, "risk_level": risk_level })
        }).collect();
        risks.sort_by(|a, b| b["risk_level"].as_str().cmp(&a["risk_level"].as_str()));

        let high_risk = risks.iter().filter(|r| r["risk_level"] == "high").count();
        let llm = ctx.call_llm("Analyze supply chain risks and recommend mitigations.", &format!("Supply Chain ({} components, {} high risk):\n{}\n\nMitigations?", risks.len(), high_risk, serde_json::to_string_pretty(&risks).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_components": risks.len(), "high_risk": high_risk }, "risks": risks, "llm_analysis": llm }))
    }
}
impl Default for SupplyChainRiskAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Find alternative for Widget A.")) }

    #[tokio::test]
    async fn test_risk() {
        let input = serde_json::json!({ "suppliers": "component,supplier,lead_time_days\nWidget A,Supplier X,45\nWidget B,Supplier Y,10\nWidget B,Supplier Z,12" });
        let r = SupplyChainRiskAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["high_risk"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(SupplyChainRiskAgent::new().execute_with_context(serde_json::json!({"suppliers":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(SupplyChainRiskAgent::new().manifest().id, "operations.supply-chain-risk"); }
}