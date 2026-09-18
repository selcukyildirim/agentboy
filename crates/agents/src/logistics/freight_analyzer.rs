use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct FreightAnalyzerAgent;
impl FreightAnalyzerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for FreightAnalyzerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "logistics.freight-analyzer".to_string(),
            version: "2.0.0".to_string(),
            name: "Freight Analyzer".to_string(),
            department: "Logistics".to_string(),
            description: "Analyze freight costs, compare carriers, and optimize shipping contracts"
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
            input_schema: vec![InputField::new(
                "shipments",
                "Shipments",
                InputKind::File,
                true,
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
        let csv = input["shipments"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'shipments' CSV".to_string()))?;
        let shipments = csv_util::parse_csv_to_maps(csv)?;
        if shipments.is_empty() {
            return Err(AppError::Validation("No shipments".to_string()));
        }

        let by_carrier = csv_util::sum_by(&shipments, "carrier", "cost");
        let shipments_by_carrier = csv_util::group_by(&shipments, "carrier");
        let mut carrier_analysis: Vec<serde_json::Value> = by_carrier.iter().map(|(carrier, cost)| {
            let count = shipments_by_carrier.get(carrier).map(|v| v.len()).unwrap_or(1) as f64;
            let avg = cost / count;
            serde_json::json!({ "carrier": carrier, "total_cost": cost, "shipment_count": count as u64, "avg_cost": avg })
        }).collect();
        carrier_analysis.sort_by(|a, b| {
            a["avg_cost"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&b["avg_cost"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_cost: f64 = shipments
            .iter()
            .map(|s| csv_util::record_get_f64(s, "cost"))
            .sum();
        let llm = ctx
            .call_llm(
                "Analyze freight costs and optimize.",
                &format!(
                    "Freight ({} shipments, total cost: {:.0}):\n{}\n\nOptimize?",
                    shipments.len(),
                    total_cost,
                    serde_json::to_string_pretty(&carrier_analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_shipments": shipments.len(), "total_cost": total_cost, "carriers": carrier_analysis.len() }, "carriers": carrier_analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for FreightAnalyzerAgent {
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
            "Carrier B is most cost-effective.",
        ))
    }

    #[tokio::test]
    async fn test_freight() {
        let input = serde_json::json!({ "shipments": "shipment_id,carrier,cost\nS1,A,500\nS2,B,300\nS3,A,600" });
        let r = FreightAnalyzerAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["total_shipments"], 3);
        assert_eq!(r["summary"]["carriers"], 2);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(FreightAnalyzerAgent::new()
            .execute_with_context(serde_json::json!({"shipments":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            FreightAnalyzerAgent::new().manifest().id,
            "logistics.freight-analyzer"
        );
    }
}
