use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct ContractAnalyzerAgent;
impl ContractAnalyzerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for ContractAnalyzerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "legal.contract-analyzer".to_string(), version: "2.0.0".to_string(), name: "Contract Analyzer".to_string(), department: "Legal".to_string(), description: "Analyze contracts for key terms, obligations, renewal dates, and risk clauses".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "document.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["contracts"].as_str().ok_or_else(|| AppError::Validation("Missing 'contracts' CSV".to_string()))?;
        let contracts = csv_util::parse_csv_to_maps(csv)?;
        if contracts.is_empty() { return Err(AppError::Validation("No contracts".to_string())); }

        let mut analysis: Vec<serde_json::Value> = contracts.iter().map(|c| {
            let value = csv_util::record_get_f64(c, "annual_value");
            let risk_score = csv_util::record_get_f64(c, "risk_score");
            let auto_renew = csv_util::record_get_str(c, "auto_renew") == "true";
            let risk_level = if risk_score > 0.7 { "high" } else if risk_score > 0.4 { "medium" } else { "low" };
            serde_json::json!({ "contract_id": csv_util::record_get_str(c, "contract_id"), "counterparty": csv_util::record_get_str(c, "counterparty"), "type": csv_util::record_get_str(c, "type"), "annual_value": value, "risk_score": risk_score, "risk_level": risk_level, "auto_renew": auto_renew })
        }).collect();
        analysis.sort_by(|a, b| b["risk_score"].as_f64().unwrap_or(0.0).partial_cmp(&a["risk_score"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let high_risk = analysis.iter().filter(|a| a["risk_level"] == "high").count();
        let total_value: f64 = analysis.iter().map(|a| a["annual_value"].as_f64().unwrap_or(0.0)).sum();

        let llm = ctx.call_llm("Analyze contracts.", &format!("Contracts ({} total, value: {:.0}, {} high risk):\n{}\n\nKey risks?", contracts.len(), total_value, high_risk, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_contracts": contracts.len(), "total_value": total_value, "high_risk": high_risk }, "contracts": analysis, "llm_analysis": llm }))
    }
}
impl Default for ContractAnalyzerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Contract C is high risk.")) }

    #[tokio::test]
    async fn test_contracts() {
        let input = serde_json::json!({ "contracts": "contract_id,counterparty,type,annual_value,risk_score,auto_renew\nC1,VendorA,Service,50000,0.3,true\nC2,VendorB,License,100000,0.8,false" });
        let r = ContractAnalyzerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["high_risk"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(ContractAnalyzerAgent::new().execute_with_context(serde_json::json!({"contracts":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(ContractAnalyzerAgent::new().manifest().id, "legal.contract-analyzer"); }
}