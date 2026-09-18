use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct VendorRiskAgent;

impl VendorRiskAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for VendorRiskAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.vendor-risk".to_string(),
            version: "2.0.0".to_string(),
            name: "Vendor Risk".to_string(),
            department: "Procurement".to_string(),
            description: "Assess vendor risk based on delivery performance, quality, financial stability, and concentration".to_string(),
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
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("vendors", "Vendors", InputKind::File, true),
            ],
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
        let vendors_csv = input["vendors"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'vendors' CSV".to_string()))?;

        let vendors = csv_util::parse_csv_to_maps(vendors_csv)?;
        if vendors.is_empty() {
            return Err(AppError::Validation("No vendor records found".to_string()));
        }

        let total_spend: f64 = vendors
            .iter()
            .map(|r| csv_util::record_get_f64(r, "annual_spend"))
            .sum();

        let mut vendor_reports = Vec::new();
        let mut high_risk_count = 0;

        for vendor in &vendors {
            let name = csv_util::record_get_str(vendor, "name");
            let delivery_score = csv_util::record_get_f64(vendor, "delivery_score");
            let quality_score = csv_util::record_get_f64(vendor, "quality_score");
            let financial_score = csv_util::record_get_f64(vendor, "financial_score");
            let annual_spend = csv_util::record_get_f64(vendor, "annual_spend");
            let lead_time_days = csv_util::record_get_f64(vendor, "lead_time_days");

            let risk_score = calculate_risk_score(
                delivery_score,
                quality_score,
                financial_score,
                lead_time_days,
            );
            let risk_level = if risk_score > 70 {
                "high"
            } else if risk_score > 40 {
                "medium"
            } else {
                "low"
            };

            if risk_level == "high" {
                high_risk_count += 1;
            }

            let concentration = if total_spend > 0.0 {
                annual_spend / total_spend
            } else {
                0.0
            };

            vendor_reports.push(serde_json::json!({
                "name": name,
                "delivery_score": delivery_score,
                "quality_score": quality_score,
                "financial_score": financial_score,
                "annual_spend": annual_spend,
                "lead_time_days": lead_time_days,
                "risk_score": risk_score,
                "risk_level": risk_level,
                "concentration": format!("{:.1}%", concentration * 100.0),
            }));
        }

        vendor_reports.sort_by(|a, b| {
            b["risk_score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["risk_score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let system_prompt = "You are a vendor risk manager. Analyze vendor risk profiles and recommend mitigation strategies. Be concise.";
        let user_prompt = format!(
            "Vendor Risk Assessment ({} vendors):\n- High risk: {}\n- Total spend: {:.2}\n\nVendor Details:\n{}\n\nProvide risk mitigation recommendations.",
            vendors.len(), high_risk_count, total_spend,
            serde_json::to_string_pretty(&vendor_reports).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_vendors": vendors.len(),
                "high_risk": high_risk_count,
                "total_spend": total_spend,
            },
            "vendors": vendor_reports,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for VendorRiskAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_risk_score(delivery: f64, quality: f64, financial: f64, lead_time: f64) -> u32 {
    let mut score: u32 = 0;

    if delivery < 0.5 {
        score += 30;
    } else if delivery < 0.7 {
        score += 15;
    }
    if quality < 0.5 {
        score += 30;
    } else if quality < 0.7 {
        score += 15;
    }
    if financial < 0.5 {
        score += 25;
    } else if financial < 0.7 {
        score += 10;
    }
    if lead_time > 30.0 {
        score += 15;
    } else if lead_time > 14.0 {
        score += 5;
    }

    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "Vendor B is high risk. Consider backup suppliers.",
        ))
    }

    #[tokio::test]
    async fn test_vendor_risk() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendors": "name,delivery_score,quality_score,financial_score,annual_spend,lead_time_days\nAcme,0.9,0.8,0.9,50000,7\nBeta,0.3,0.4,0.3,30000,45"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total_vendors"], 2);
        assert_eq!(result["summary"]["high_risk"], 1);
    }

    #[tokio::test]
    async fn test_low_risk() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendors": "name,delivery_score,quality_score,financial_score,annual_spend,lead_time_days\nAcme,0.9,0.9,0.9,10000,5"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["high_risk"], 0);
    }

    #[tokio::test]
    async fn test_concentration() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendors": "name,delivery_score,quality_score,financial_score,annual_spend,lead_time_days\nA,0.9,0.9,0.9,900,5\nB,0.9,0.9,0.9,100,5"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let vendors = result["vendors"].as_array().unwrap();
        assert_eq!(vendors[0]["concentration"], "90.0%");
    }

    #[tokio::test]
    async fn test_missing_vendors() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_vendors() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "vendors": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_risk_sorted() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendors": "name,delivery_score,quality_score,financial_score,annual_spend,lead_time_days\nGood,0.9,0.9,0.9,100,5\nBad,0.2,0.2,0.2,100,60"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let vendors = result["vendors"].as_array().unwrap();
        assert!(
            vendors[0]["risk_score"].as_f64().unwrap()
                >= vendors[1]["risk_score"].as_f64().unwrap()
        );
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = VendorRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "vendors": "name,delivery_score,quality_score,financial_score,annual_spend,lead_time_days\nA,0.5,0.5,0.5,100,10"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_risk_score_calculation() {
        let score = calculate_risk_score(0.9, 0.9, 0.9, 5.0);
        assert!(score < 20);
        let score = calculate_risk_score(0.2, 0.2, 0.2, 60.0);
        assert!(score > 70);
    }

    #[test]
    fn test_manifest() {
        let agent = VendorRiskAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.vendor-risk");
        assert!(m.permissions.network_llm);
    }
}
