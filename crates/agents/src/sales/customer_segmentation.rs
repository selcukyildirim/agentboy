use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct CustomerSegmentationAgent;

impl CustomerSegmentationAgent {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl Agent for CustomerSegmentationAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "sales.customer-segmentation".to_string(),
            version: "2.0.0".to_string(),
            name: "Customer Segmentation".to_string(),
            department: "Sales".to_string(),
            description: "Segment customers by revenue, frequency, and value for targeted strategies".to_string(),
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
        let customers_csv = input["customers"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'customers' CSV".to_string()))?;

        let customers = csv_util::parse_csv_to_maps(customers_csv)?;
        if customers.is_empty() {
            return Err(AppError::Validation("No customers found".to_string()));
        }

        let total_revenue: f64 = customers.iter().map(|r| csv_util::record_get_f64(r, "total_revenue")).sum();

        let mut segments: Vec<serde_json::Value> = customers.iter().map(|c| {
            let revenue = csv_util::record_get_f64(c, "total_revenue");
            let frequency = csv_util::record_get_f64(c, "purchase_frequency");
            let segment = if revenue > 50000.0 && frequency > 10.0 { "platinum" }
                else if revenue > 20000.0 && frequency > 5.0 { "gold" }
                else if revenue > 5000.0 { "silver" }
                else { "bronze" };

            serde_json::json!({
                "name": csv_util::record_get_str(c, "name"),
                "total_revenue": revenue,
                "purchase_frequency": frequency,
                "segment": segment,
                "revenue_share": if total_revenue > 0.0 { revenue / total_revenue * 100.0 } else { 0.0 },
            })
        }).collect();

        segments.sort_by(|a, b| b["total_revenue"].as_f64().unwrap_or(0.0).partial_cmp(&a["total_revenue"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let seg_counts: std::collections::HashMap<String, u64> = segments.iter().fold(std::collections::HashMap::new(), |mut acc, s| {
            *acc.entry(s["segment"].as_str().unwrap_or("unknown").to_string()).or_insert(0) += 1;
            acc
        });

        let system_prompt = "You are a customer segmentation analyst. Analyze segments and recommend targeted strategies. Be concise.";
        let user_prompt = format!(
            "Customer Segmentation ({} customers, total revenue: {:.2}):\nSegments: {:?}\n\nTop customers:\n{}\n\nProvide segment-specific strategies.",
            customers.len(), total_revenue, seg_counts,
            serde_json::to_string_pretty(&segments[..segments.len().min(5)]).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_customers": customers.len(), "total_revenue": total_revenue, "segments": seg_counts },
            "customers": segments,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for CustomerSegmentationAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Focus on platinum retention."))
    }

    #[tokio::test]
    async fn test_segmentation() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "customers": "name,total_revenue,purchase_frequency\nAlice,60000,15\nBob,25000,8\nCarol,8000,3\nDave,1000,1"
        });
        let result = CustomerSegmentationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total_customers"], 4);
        assert!(result["summary"]["total_revenue"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_segments() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "customers": "name,total_revenue,purchase_frequency\nP,60000,15\nG,25000,8\nS,8000,3\nB,1000,1"
        });
        let result = CustomerSegmentationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        let customers = result["customers"].as_array().unwrap();
        assert_eq!(customers[0]["segment"], "platinum");
        assert_eq!(customers[1]["segment"], "gold");
        assert_eq!(customers[2]["segment"], "silver");
        assert_eq!(customers[3]["segment"], "bronze");
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(CustomerSegmentationAgent::new().execute_with_context(serde_json::json!({ "customers": "" }), &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "customers": "name,total_revenue,purchase_frequency\nA,5000,5"
        });
        let result = CustomerSegmentationAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = CustomerSegmentationAgent::new().manifest();
        assert_eq!(m.id, "sales.customer-segmentation");
        assert!(m.permissions.network_llm);
    }
}