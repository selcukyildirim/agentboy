use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct InvoiceControlAgent;

impl InvoiceControlAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for InvoiceControlAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.invoice-control".to_string(),
            version: "2.0.0".to_string(),
            name: "Invoice Control".to_string(),
            department: "Accounting".to_string(),
            description: "Three-way match: validate invoices against PO and receipt, detect fraud and discrepancies".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.compare".to_string(), "document.validate".to_string(), "llm.analysis".to_string()],
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
                InputField::new("invoice", "Invoice (JSON)", InputKind::Json, true),
                InputField::new("purchase_order", "Purchase Order (JSON)", InputKind::Json, false),
                InputField::new("receipt", "Receipt (JSON)", InputKind::Json, false),
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
        let invoice = input["invoice"]
            .as_object()
            .ok_or_else(|| AppError::Validation("Missing 'invoice' object".to_string()))?;
        let po = input["purchase_order"].as_object();
        let receipt = input["receipt"].as_object();

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        let inv_total = invoice.get("total").and_then(|v| v.as_str()).unwrap_or("0");
        let inv_vendor = invoice.get("vendor").and_then(|v| v.as_str()).unwrap_or("");
        let inv_date = invoice.get("date").and_then(|v| v.as_str()).unwrap_or("");

        if let Some(po_data) = po {
            let po_total = po_data.get("total").and_then(|v| v.as_str()).unwrap_or("0");
            let po_vendor = po_data.get("vendor").and_then(|v| v.as_str()).unwrap_or("");

            if inv_total != po_total {
                issues.push(serde_json::json!({
                    "type": "amount_mismatch",
                    "severity": "high",
                    "invoice_total": inv_total,
                    "po_total": po_total,
                    "description": "Invoice total does not match PO total"
                }));
            }
            if !inv_vendor.is_empty() && !po_vendor.is_empty() && inv_vendor != po_vendor {
                warnings.push(serde_json::json!({
                    "type": "vendor_mismatch",
                    "severity": "medium",
                    "invoice_vendor": inv_vendor,
                    "po_vendor": po_vendor,
                }));
            }

            if let Some(po_items) = po_data.get("items").and_then(|v| v.as_array()) {
                if let Some(inv_items) = invoice.get("items").and_then(|v| v.as_array()) {
                    if inv_items.len() != po_items.len() {
                        warnings.push(serde_json::json!({
                            "type": "item_count_mismatch",
                            "severity": "low",
                            "invoice_items": inv_items.len(),
                            "po_items": po_items.len(),
                        }));
                    }
                }
            }
        }

        if let Some(receipt_data) = receipt {
            let receipt_amount = receipt_data
                .get("amount")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            if inv_total != receipt_amount {
                warnings.push(serde_json::json!({
                    "type": "receipt_amount_mismatch",
                    "severity": "medium",
                    "invoice_amount": inv_total,
                    "receipt_amount": receipt_amount,
                }));
            }
            let receipt_date = receipt_data
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !inv_date.is_empty() && !receipt_date.is_empty() && inv_date != receipt_date {
                warnings.push(serde_json::json!({
                    "type": "date_mismatch",
                    "severity": "low",
                    "invoice_date": inv_date,
                    "receipt_date": receipt_date,
                }));
            }
        }

        let status = if issues.iter().any(|i| i["severity"] == "high") {
            "rejected"
        } else if !issues.is_empty() {
            "requires_review"
        } else if !warnings.is_empty() {
            "approved_with_warnings"
        } else {
            "approved"
        };

        let system_prompt = "You are an accounts payable specialist. Analyze the invoice validation results (three-way match) and provide guidance on resolution. Be concise.";
        let user_prompt = format!(
            "Invoice Validation Result:\nStatus: {}\nVendor: {}\nTotal: {}\n\nIssues ({}):\n{}\n\nWarnings ({}):\n{}\n\nProvide resolution guidance.",
            status, inv_vendor, inv_total,
            issues.len(), serde_json::to_string_pretty(&issues).unwrap_or_default(),
            warnings.len(), serde_json::to_string_pretty(&warnings).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "status": status,
            "issues": issues,
            "warnings": warnings,
            "issue_count": issues.len(),
            "warning_count": warnings.len(),
            "has_po_match": po.is_some(),
            "has_receipt_match": receipt.is_some(),
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for InvoiceControlAgent {
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
            "Three-way match looks good.",
        ))
    }

    #[tokio::test]
    async fn test_approved() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "100.00", "vendor": "ACME", "date": "2024-01-15"},
            "purchase_order": {"total": "100.00", "vendor": "ACME"},
            "receipt": {"amount": "100.00", "date": "2024-01-15"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "approved");
        assert_eq!(result["issue_count"], 0);
        assert_eq!(result["warning_count"], 0);
    }

    #[tokio::test]
    async fn test_amount_mismatch() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "150.00", "vendor": "ACME"},
            "purchase_order": {"total": "100.00", "vendor": "ACME"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "rejected");
        assert_eq!(result["issue_count"], 1);
    }

    #[tokio::test]
    async fn test_vendor_mismatch_warning() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "100.00", "vendor": "ACME"},
            "purchase_order": {"total": "100.00", "vendor": "BETA"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "approved_with_warnings");
        assert_eq!(result["warning_count"], 1);
    }

    #[tokio::test]
    async fn test_no_po_no_receipt() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "100.00", "vendor": "ACME"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "approved");
        assert!(!result["has_po_match"].as_bool().unwrap());
        assert!(!result["has_receipt_match"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_receipt_mismatch() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "100.00", "vendor": "ACME"},
            "receipt": {"amount": "95.00"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["warning_count"], 1);
    }

    #[tokio::test]
    async fn test_missing_invoice() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = InvoiceControlAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "invoice": {"total": "100.00", "vendor": "ACME"}
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = InvoiceControlAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.invoice-control");
        assert!(m.permissions.network_llm);
    }
}
