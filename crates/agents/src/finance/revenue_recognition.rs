use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct RevenueRecognitionAgent;

impl RevenueRecognitionAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for RevenueRecognitionAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.revenue-recognition".to_string(),
            version: "2.0.0".to_string(),
            name: "Revenue Recognition".to_string(),
            department: "Finance".to_string(),
            description: "Analyze revenue streams, recognize earned vs deferred revenue, and identify patterns".to_string(),
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
                max_steps: 40,
                timeout_seconds: 180,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("invoices", "Invoices", InputKind::File, true).with_example("invoice_id,customer,amount,due_date\nINV-1,Acme,1000.00,2024-02-01"),
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
        let invoices_csv = input["invoices"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'invoices' CSV string".to_string()))?;

        let invoices = csv_util::parse_csv_to_maps(invoices_csv)?;
        if invoices.is_empty() {
            return Err(AppError::Validation("No invoice records found".to_string()));
        }

        let mut earned = 0.0;
        let mut deferred = 0.0;
        let mut total_revenue = 0.0;
        let mut by_customer = std::collections::HashMap::new();
        let mut by_product = std::collections::HashMap::new();
        let mut revenue_by_month = std::collections::HashMap::new();
        let mut aging = std::collections::HashMap::new();

        for invoice in &invoices {
            let amount = csv_util::record_get_f64(invoice, "amount");
            let status = csv_util::record_get_str(invoice, "status");
            let customer = csv_util::record_get_str(invoice, "customer");
            let product = csv_util::record_get_str(invoice, "product");
            let date = csv_util::record_get_str(invoice, "date");
            let days_outstanding = csv_util::record_get_f64(invoice, "days_outstanding");

            total_revenue += amount;

            match status {
                "paid" | "earned" => earned += amount,
                "pending" | "deferred" => deferred += amount,
                _ => earned += amount,
            }

            *by_customer.entry(customer.to_string()).or_insert(0.0) += amount;
            *by_product.entry(product.to_string()).or_insert(0.0) += amount;

            if date.len() >= 7 {
                let month = &date[..7];
                *revenue_by_month.entry(month.to_string()).or_insert(0.0) += amount;
            }

            if days_outstanding > 0.0 {
                let bucket = if days_outstanding <= 30.0 {
                    "0-30"
                } else if days_outstanding <= 60.0 {
                    "31-60"
                } else if days_outstanding <= 90.0 {
                    "61-90"
                } else {
                    "90+"
                };
                *aging.entry(bucket.to_string()).or_insert(0.0) += amount;
            }
        }

        let avg_invoice = if invoices.is_empty() {
            0.0
        } else {
            total_revenue / invoices.len() as f64
        };

        let system_prompt = "You are a revenue recognition specialist. Analyze the revenue data and provide insights on revenue quality, deferred revenue risks, customer concentration, and recognition compliance. Be concise and professional.";

        let user_prompt = format!(
            "Revenue Recognition Report:\n\
             - Total invoices: {}\n\
             - Total revenue: {:.2}\n\
             - Earned revenue: {:.2}\n\
             - Deferred revenue: {:.2}\n\
             - Average invoice: {:.2}\n\n\
             Revenue by Customer:\n{}\n\n\
             Revenue by Product:\n{}\n\n\
             Monthly Trend:\n{}\n\n\
             AR Aging:\n{}\n\n\
             Provide revenue quality analysis and recognition concerns.",
            invoices.len(),
            total_revenue,
            earned,
            deferred,
            avg_invoice,
            serde_json::to_string_pretty(&by_customer).unwrap_or_default(),
            serde_json::to_string_pretty(&by_product).unwrap_or_default(),
            serde_json::to_string_pretty(&revenue_by_month).unwrap_or_default(),
            serde_json::to_string_pretty(&aging).unwrap_or_default(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_invoices": invoices.len(),
                "total_revenue": total_revenue,
                "earned_revenue": earned,
                "deferred_revenue": deferred,
                "average_invoice": avg_invoice,
                "recognition_rate": if total_revenue > 0.0 { earned / total_revenue * 100.0 } else { 0.0 },
            },
            "by_customer": by_customer,
            "by_product": by_product,
            "monthly_trend": revenue_by_month,
            "ar_aging": aging,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for RevenueRecognitionAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "Revenue recognition looks healthy. 80% earned, 20% deferred.",
        ))
    }

    #[tokio::test]
    async fn test_revenue_recognition() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoices": "date,amount,status,customer,product,days_outstanding\n2024-01-01,1000,paid,Acme,Widget,0\n2024-01-15,2000,pending,Beta,Gadget,0\n2024-02-01,500,paid,Acme,Widget,45"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total_invoices"], 3);
        assert!(result["summary"]["total_revenue"].as_f64().unwrap() > 0.0);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_earned_vs_deferred() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoices": "date,amount,status,customer,product\n2024-01-01,1000,paid,C1,P1\n2024-01-02,2000,pending,C2,P2"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["earned_revenue"], 1000.0);
        assert_eq!(result["summary"]["deferred_revenue"], 2000.0);
    }

    #[tokio::test]
    async fn test_by_customer() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoices": "date,amount,status,customer,product\n2024-01-01,100,paid,Acme,P1\n2024-01-02,200,paid,Acme,P2\n2024-01-03,300,paid,Beta,P1"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let by_cust = result["by_customer"].as_object().unwrap();
        assert_eq!(by_cust["Acme"], 300.0);
        assert_eq!(by_cust["Beta"], 300.0);
    }

    #[tokio::test]
    async fn test_missing_invoices() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_invoices() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "invoices": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_ar_aging() {
        let agent = RevenueRecognitionAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoices": "date,amount,status,customer,product,days_outstanding\n2024-01-01,100,paid,C1,P1,15\n2024-01-02,200,paid,C2,P2,45\n2024-01-03,300,paid,C3,P3,95"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let aging = result["ar_aging"].as_object().unwrap();
        assert!(aging.contains_key("0-30"));
        assert!(aging.contains_key("31-60"));
        assert!(aging.contains_key("90+"));
    }

    #[test]
    fn test_manifest() {
        let agent = RevenueRecognitionAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.revenue-recognition");
        assert!(m.permissions.network_llm);
    }
}
