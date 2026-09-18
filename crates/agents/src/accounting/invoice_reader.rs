use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct InvoiceReaderAgent;

impl InvoiceReaderAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for InvoiceReaderAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.invoice-reader".to_string(),
            version: "2.0.0".to_string(),
            name: "Invoice Reader".to_string(),
            department: "Accounting".to_string(),
            description: "Extract and validate structured data from invoice text using regex + LLM"
                .to_string(),
            tier: AgentTier::Free,
            skills: vec![
                "document.parse".to_string(),
                "document.extract".to_string(),
                "llm.analysis".to_string(),
            ],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 20,
                timeout_seconds: 120,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("content", "Invoice Content", InputKind::Text, true)
                    .with_example("Invoice No: INV-123\nVendor: Acme\nTotal: 1000.00"),
                InputField::new("filename", "Filename", InputKind::Text, false)
                    .with_example("invoice.txt"),
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
        let content = input["content"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'content' string".to_string()))?;
        let filename = input["filename"].as_str().unwrap_or("unknown");

        let extracted = extract_invoice_data(content);

        let system_prompt = "You are an invoice data extraction expert. Given raw invoice text, extract all key fields (invoice number, date, vendor, line items, total, tax). Return as JSON.";
        let user_prompt = format!(
            "Raw invoice text:\n\n{}\n\nExtract all invoice fields as JSON.",
            content
        );
        let llm_result = ctx.call_llm(system_prompt, &user_prompt).await?;

        let mut merged = extracted.as_object().cloned().unwrap_or_default();
        let mut llm_enhanced = false;
        if let Ok(llm_data) = serde_json::from_str::<serde_json::Value>(&llm_result) {
            if let Some(llm_obj) = llm_data.as_object() {
                llm_enhanced = true;
                for (k, v) in llm_obj {
                    if !merged.contains_key(k) || merged[k].as_str().unwrap_or("") == "" {
                        merged.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        let fields_count = merged.len();
        Ok(serde_json::json!({
            "filename": filename,
            "extracted_data": serde_json::Value::Object(merged),
            "fields_extracted": fields_count,
            "llm_enhanced": llm_enhanced,
        }))
    }
}

impl Default for InvoiceReaderAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_invoice_data(content: &str) -> serde_json::Value {
    let mut data = serde_json::Map::new();

    if let Some(num) = extract_pattern(content, &["invoice #", "inv-", "invoice no", "invoice:"]) {
        data.insert("invoice_number".to_string(), serde_json::Value::String(num));
    }
    if let Some(date) = extract_date(content) {
        data.insert("date".to_string(), serde_json::Value::String(date));
    }
    if let Some(total) = extract_amount(
        content,
        &["total", "amount due", "balance due", "grand total"],
    ) {
        data.insert("total".to_string(), serde_json::Value::String(total));
    }
    if let Some(tax) = extract_amount(content, &["tax", "vat", "gst"]) {
        data.insert("tax".to_string(), serde_json::Value::String(tax));
    }
    if let Some(vendor) = extract_vendor(content) {
        data.insert("vendor".to_string(), serde_json::Value::String(vendor));
    }

    serde_json::Value::Object(data)
}

fn extract_pattern(content: &str, keywords: &[&str]) -> Option<String> {
    let lower = content.to_lowercase();
    for keyword in keywords {
        if let Some(pos) = lower.find(keyword) {
            let after = &content[pos + keyword.len()..];
            let value: String = after
                .chars()
                .skip_while(|c| *c == ':' || *c == ' ')
                .take_while(|c| !c.is_whitespace() && *c != '\n')
                .collect();
            if !value.is_empty() && value.len() < 50 {
                return Some(value);
            }
        }
    }
    None
}

fn extract_date(content: &str) -> Option<String> {
    let patterns = [
        r"\d{4}-\d{2}-\d{2}",
        r"\d{2}/\d{2}/\d{4}",
        r"\d{2}\.\d{2}\.\d{4}",
    ];
    for pattern in &patterns {
        if let Some(mat) = regex_lite::Regex::new(pattern).ok()?.find(content) {
            return Some(mat.as_str().to_string());
        }
    }
    None
}

fn extract_amount(content: &str, keywords: &[&str]) -> Option<String> {
    let lower = content.to_lowercase();
    for keyword in keywords {
        if let Some(pos) = lower.find(keyword) {
            let after = &content[pos..];
            for (i, c) in after.char_indices() {
                if c == '$' || c == '€' || c == '£' || c.is_ascii_digit() {
                    let amount: String = after[i..]
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
                        .collect();
                    if !amount.is_empty() {
                        return Some(amount);
                    }
                }
            }
        }
    }
    None
}

fn extract_vendor(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().take(5).collect();
    lines.first().and_then(|line| {
        let trimmed = line.trim();
        if trimmed.len() > 3 && trimmed.len() < 100 {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            r#"{"invoice_number":"INV-001","date":"2024-01-15","vendor":"ACME Corp","total":"1234.56"}"#,
        ))
    }

    #[tokio::test]
    async fn test_extract_invoice() {
        let agent = InvoiceReaderAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "content": "ACME Corp\nInvoice #INV-001\nDate: 2024-01-15\nTotal: $1,234.56\nTax: $100.00",
            "filename": "invoice.pdf"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let data = result["extracted_data"].as_object().unwrap();
        assert!(data.contains_key("invoice_number"));
        assert!(data.contains_key("date"));
        assert!(data.contains_key("total"));
        assert!(result["llm_enhanced"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_regex_fallback() {
        let agent = InvoiceReaderAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "content": "Beta Inc\nINV-002\n2024-02-01\n$500.00",
            "filename": "inv.txt"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["extracted_data"].is_object());
    }

    #[tokio::test]
    async fn test_missing_content() {
        let agent = InvoiceReaderAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "filename": "test.pdf" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm_enhances_data() {
        let agent = InvoiceReaderAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "content": "Some Corp\nInvoice #X-123\nTotal: $500",
            "filename": "test.pdf"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_enhanced"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_empty_content() {
        let agent = InvoiceReaderAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "content": "", "filename": "empty.pdf" });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["extracted_data"].is_object());
    }

    #[test]
    fn test_extract_date() {
        assert_eq!(
            extract_date("Date: 2024-01-15"),
            Some("2024-01-15".to_string())
        );
        assert_eq!(extract_date("01/15/2024"), Some("01/15/2024".to_string()));
        assert_eq!(extract_date("no date here"), None);
    }

    #[test]
    fn test_extract_amount() {
        let amount = extract_amount("Total: $1,234.56", &["total"]);
        assert!(amount.is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = InvoiceReaderAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.invoice-reader");
        assert!(m.permissions.network_llm);
    }
}
