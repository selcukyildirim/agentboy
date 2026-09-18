use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct TaxComplianceAgent;

impl TaxComplianceAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for TaxComplianceAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.tax-compliance".to_string(),
            version: "2.0.0".to_string(),
            name: "Tax Compliance".to_string(),
            department: "Accounting".to_string(),
            description: "Validate tax calculations, check VAT/GST compliance, flag missing tax IDs".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
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
        let transactions_csv = input["transactions"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'transactions' CSV".to_string()))?;
        let tax_rate = input["tax_rate"].as_f64().unwrap_or(0.20);

        let transactions = csv_util::parse_csv_to_maps(transactions_csv)?;
        if transactions.is_empty() {
            return Err(AppError::Validation("No transactions found".to_string()));
        }

        let mut issues = Vec::new();
        let mut total_taxable = 0.0;
        let mut total_tax = 0.0;

        for (i, txn) in transactions.iter().enumerate() {
            let amount = csv_util::record_get_f64(txn, "amount");
            let expected_tax = csv_util::record_get_f64(txn, "tax");
            let tax_id = csv_util::record_get_str(txn, "tax_id");
            let category = csv_util::record_get_str(txn, "category");

            let computed_tax = (amount * tax_rate * 100.0).round() / 100.0;

            if expected_tax > 0.0 && (expected_tax - computed_tax).abs() > 0.01 {
                issues.push(serde_json::json!({
                    "row": i + 1,
                    "type": "tax_calculation_error",
                    "severity": "high",
                    "amount": amount,
                    "expected_tax": expected_tax,
                    "computed_tax": computed_tax,
                    "difference": (expected_tax - computed_tax).abs(),
                }));
            }

            if amount > 1000.0 && tax_id.is_empty() {
                issues.push(serde_json::json!({
                    "row": i + 1,
                    "type": "missing_tax_id",
                    "severity": "medium",
                    "amount": amount,
                    "description": format!("High-value transaction ({}) missing tax_id", category),
                }));
            }

            if amount > 0.0 {
                total_taxable += amount;
                total_tax += if expected_tax > 0.0 { expected_tax } else { computed_tax };
            }
        }

        let system_prompt = "You are a tax compliance officer. Analyze the tax compliance results and flag risks. Be concise.";
        let user_prompt = format!(
            "Tax Compliance Report:\n- Total taxable: {:.2}\n- Total tax: {:.2}\n- Effective rate: {:.1}%\n- Issues: {}\n\nIssue Details:\n{}\n\nProvide compliance assessment.",
            total_taxable, total_tax,
            if total_taxable > 0.0 { total_tax / total_taxable * 100.0 } else { 0.0 },
            issues.len(),
            serde_json::to_string_pretty(&issues).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "status": if issues.iter().any(|i| i["severity"] == "high") { "non_compliant" } else if !issues.is_empty() { "review_required" } else { "compliant" },
            "summary": {
                "total_transactions": transactions.len(),
                "total_taxable": total_taxable,
                "total_tax": total_tax,
                "effective_rate": if total_taxable > 0.0 { total_tax / total_taxable * 100.0 } else { 0.0 },
                "issue_count": issues.len(),
            },
            "issues": issues,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for TaxComplianceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Tax compliance looks good."))
    }

    #[tokio::test]
    async fn test_compliant() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "amount,tax,category,tax_id\n100,20,Office,TAX001\n200,40,Travel,TAX002",
            "tax_rate": 0.20
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "compliant");
        assert_eq!(result["summary"]["issue_count"], 0);
    }

    #[tokio::test]
    async fn test_tax_calculation_error() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "amount,tax,category,tax_id\n100,15,Office,T001",
            "tax_rate": 0.20
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "non_compliant");
    }

    #[tokio::test]
    async fn test_missing_tax_id() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "amount,tax,category,tax_id\n5000,1000,Equipment,",
            "tax_rate": 0.20
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["issue_count"], 1);
    }

    #[tokio::test]
    async fn test_missing_transactions() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_transactions() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "transactions": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_default_tax_rate() {
        let agent = TaxComplianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "amount,tax,category,tax_id\n100,20,Office,ID1"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "compliant");
    }

    #[test]
    fn test_manifest() {
        let agent = TaxComplianceAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.tax-compliance");
        assert!(m.permissions.network_llm);
    }
}