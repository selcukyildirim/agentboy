use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};

use crate::csv_util;

pub struct PurchaseOrderValidatorAgent;

impl PurchaseOrderValidatorAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for PurchaseOrderValidatorAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "procurement.po-validator".to_string(),
            version: "2.0.0".to_string(),
            name: "PO Validator".to_string(),
            department: "Procurement".to_string(),
            description: "Validate purchase orders against budget, policy, and vendor limits".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "document.validate".to_string(), "llm.analysis".to_string()],
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
            input_schema: vec![
                InputField::new("purchase_orders", "Purchase Orders", InputKind::File, true).with_example("po_number,supplier,amount\nPO-1,Acme,1000.00"),
                InputField::new("budget_limit", "Budget Limit", InputKind::Number, true).with_example("50000"),
                InputField::new("approval_threshold", "Approval Threshold", InputKind::Number, true).with_example("10000"),
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
        let po_csv = input["purchase_orders"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'purchase_orders' CSV".to_string()))?;
        let budget_limit = input["budget_limit"].as_f64().unwrap_or(f64::MAX);
        let approval_threshold = input["approval_threshold"].as_f64().unwrap_or(5000.0);

        let pos = csv_util::parse_csv_to_maps(po_csv)?;
        if pos.is_empty() {
            return Err(AppError::Validation("No purchase orders found".to_string()));
        }

        let mut validations = Vec::new();
        let mut total_amount = 0.0;
        let mut requires_approval = 0;
        let mut budget_exceeded = 0;

        for po in &pos {
            let amount = csv_util::record_get_f64(po, "amount");
            let vendor = csv_util::record_get_str(po, "vendor");
            let po_number = csv_util::record_get_str(po, "po_number");
            total_amount += amount;

            let mut issues = Vec::new();
            if amount > approval_threshold {
                issues.push("Exceeds approval threshold".to_string());
                requires_approval += 1;
            }
            if amount > budget_limit {
                issues.push("Exceeds budget limit".to_string());
                budget_exceeded += 1;
            }
            let vendor_limit = csv_util::record_get_f64(po, "vendor_limit");
            if vendor_limit > 0.0 && amount > vendor_limit {
                issues.push("Exceeds vendor limit".to_string());
            }

            let status = if issues.is_empty() {
                "approved"
            } else if issues.iter().any(|i| i.contains("budget")) {
                "rejected"
            } else {
                "pending_approval"
            };

            validations.push(serde_json::json!({
                "po_number": po_number,
                "vendor": vendor,
                "amount": amount,
                "status": status,
                "issues": issues,
            }));
        }

        let system_prompt = "You are a purchase order compliance officer. Analyze PO validation results and recommend actions. Be concise.";
        let user_prompt = format!(
            "PO Validation:\n- Total POs: {}\n- Total amount: {:.2}\n- Requires approval: {}\n- Budget exceeded: {}\n\nPO Details:\n{}\n\nProvide validation summary and actions.",
            pos.len(), total_amount, requires_approval, budget_exceeded,
            serde_json::to_string_pretty(&validations).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_pos": pos.len(),
                "total_amount": total_amount,
                "requires_approval": requires_approval,
                "budget_exceeded": budget_exceeded,
                "approved": validations.iter().filter(|v| v["status"] == "approved").count(),
            },
            "validations": validations,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for PurchaseOrderValidatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("All POs are within policy."))
    }

    #[tokio::test]
    async fn test_all_approved() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "purchase_orders": "po_number,vendor,amount,vendor_limit\nPO001,Acme,1000,5000\nPO002,Beta,2000,5000",
            "budget_limit": 10000,
            "approval_threshold": 5000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["approved"], 2);
    }

    #[tokio::test]
    async fn test_requires_approval() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "purchase_orders": "po_number,vendor,amount,vendor_limit\nPO001,Acme,6000,10000",
            "approval_threshold": 5000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["requires_approval"], 1);
    }

    #[tokio::test]
    async fn test_budget_exceeded() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "purchase_orders": "po_number,vendor,amount,vendor_limit\nPO001,Acme,15000,20000",
            "budget_limit": 10000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["budget_exceeded"], 1);
    }

    #[tokio::test]
    async fn test_missing_pos() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_pos() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "purchase_orders": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_vendor_limit() {
        let agent = PurchaseOrderValidatorAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "purchase_orders": "po_number,vendor,amount,vendor_limit\nPO001,Acme,8000,5000",
            "budget_limit": 20000
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["approved"], 0);
    }

    #[test]
    fn test_manifest() {
        let agent = PurchaseOrderValidatorAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "procurement.po-validator");
        assert!(m.permissions.network_llm);
    }
}