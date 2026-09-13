use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct SpendAnalyticsAgent;

impl SpendAnalyticsAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for SpendAnalyticsAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.spend-analytics".to_string(),
            version: "2.0.0".to_string(),
            name: "Spend Analytics".to_string(),
            department: "Procurement".to_string(),
            description: "Analyze procurement spend patterns, identify consolidation opportunities and savings".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 40,
                timeout_seconds: 180,
            },
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
        let spend_csv = input["spend_data"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'spend_data' CSV".to_string()))?;

        let records = csv_util::parse_csv_to_maps(spend_csv)?;
        if records.is_empty() {
            return Err(AppError::Validation("No spend records found".to_string()));
        }

        let total_spend: f64 = records.iter().map(|r| csv_util::record_get_f64(r, "amount")).sum();
        let by_category = csv_util::sum_by(&records, "category", "amount");
        let by_vendor = csv_util::sum_by(&records, "vendor", "amount");

        let top_vendor = by_vendor.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
        let vendor_concentration = if let Some((_, amount)) = top_vendor {
            if total_spend > 0.0 { amount / total_spend } else { 0.0 }
        } else {
            0.0
        };

        let avg_transaction = if !records.is_empty() {
            total_spend / records.len() as f64
        } else {
            0.0
        };

        let unique_vendors = by_vendor.len();
        let consolidation_potential = if unique_vendors > 3 { "high" } else if unique_vendors > 1 { "medium" } else { "low" };

        let mut category_list: Vec<serde_json::Value> = by_category.iter().map(|(cat, amt)| {
            serde_json::json!({
                "category": cat,
                "total": amt,
                "percentage": if total_spend > 0.0 { amt / total_spend * 100.0 } else { 0.0 },
            })
        }).collect();
        category_list.sort_by(|a, b| b["total"].as_f64().unwrap_or(0.0).partial_cmp(&a["total"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let system_prompt = "You are a procurement spend analyst. Analyze spend patterns and identify cost optimization opportunities. Be concise.";
        let user_prompt = format!(
            "Spend Analytics:\n- Total spend: {:.2}\n- Transactions: {}\n- Unique vendors: {}\n- Avg transaction: {:.2}\n- Top vendor concentration: {:.1}%\n- Consolidation potential: {}\n\nBy Category:\n{}\n\nBy Vendor:\n{}\n\nProvide spend optimization recommendations.",
            total_spend, records.len(), unique_vendors, avg_transaction,
            vendor_concentration * 100.0, consolidation_potential,
            serde_json::to_string_pretty(&category_list).unwrap_or_default(),
            serde_json::to_string_pretty(&by_vendor).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_spend": total_spend,
                "transaction_count": records.len(),
                "unique_vendors": unique_vendors,
                "avg_transaction": avg_transaction,
                "top_vendor_concentration": format!("{:.1}%", vendor_concentration * 100.0),
                "consolidation_potential": consolidation_potential,
            },
            "by_category": category_list,
            "by_vendor": by_vendor,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for SpendAnalyticsAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Consider consolidating vendors for 15% savings."))
    }

    #[tokio::test]
    async fn test_spend_analytics() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "spend_data": "vendor,category,amount\nAcme,Office,500\nAcme,Travel,300\nBeta,Office,200\nGamma,Supplies,1000"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["transaction_count"], 4);
        assert!(result["summary"]["total_spend"].as_f64().unwrap() > 0.0);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_vendor_concentration() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "spend_data": "vendor,category,amount\nAcme,Office,900\nAcme,Travel,900\nBeta,Office,100\nBeta,Travel,100"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["unique_vendors"], 2);
    }

    #[tokio::test]
    async fn test_category_breakdown() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "spend_data": "vendor,category,amount\nA,Office,100\nB,Travel,200\nC,Office,300"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let cats = result["by_category"].as_array().unwrap();
        assert!(!cats.is_empty());
    }

    #[tokio::test]
    async fn test_missing_data() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_data() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "spend_data": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_consolidation_potential() {
        let agent = SpendAnalyticsAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "spend_data": "vendor,category,amount\nA,C1,100\nB,C1,200\nC,C1,300\nD,C1,400"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["consolidation_potential"], "high");
    }

    #[test]
    fn test_manifest() {
        let agent = SpendAnalyticsAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.spend-analytics");
        assert!(m.permissions.network_llm);
    }
}