use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct DocumentReviewerAgent;
impl DocumentReviewerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for DocumentReviewerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "legal.document-reviewer".to_string(),
            version: "2.0.0".to_string(),
            name: "Document Reviewer".to_string(),
            department: "Legal".to_string(),
            description:
                "Review legal documents for completeness, consistency, and risk indicators"
                    .to_string(),
            tier: AgentTier::Free,
            skills: vec![
                "document.analyze".to_string(),
                "document.validate".to_string(),
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
                "documents",
                "Documents",
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
        let csv = input["documents"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'documents' CSV".to_string()))?;
        let docs = csv_util::parse_csv_to_maps(csv)?;
        if docs.is_empty() {
            return Err(AppError::Validation("No documents".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = docs.iter().map(|d| {
            let completeness = csv_util::record_get_f64(d, "completeness_score");
            let risk_flags = csv_util::record_get_f64(d, "risk_flags") as u64;
            let status = if completeness > 0.9 && risk_flags == 0 { "approved" } else if completeness > 0.7 { "needs_revision" } else { "rejected" };
            serde_json::json!({ "document_id": csv_util::record_get_str(d, "document_id"), "type": csv_util::record_get_str(d, "type"), "completeness_score": completeness, "risk_flags": risk_flags, "status": status })
        }).collect();

        let approved = analysis
            .iter()
            .filter(|a| a["status"] == "approved")
            .count();
        let needs_revision = analysis
            .iter()
            .filter(|a| a["status"] == "needs_revision")
            .count();

        let llm = ctx
            .call_llm(
                "Review legal documents.",
                &format!(
                    "Documents ({} total, {} approved, {} need revision):\n{}\n\nSummary?",
                    docs.len(),
                    approved,
                    needs_revision,
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_documents": docs.len(), "approved": approved, "needs_revision": needs_revision }, "documents": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for DocumentReviewerAgent {
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
            "Contract D2 needs revision.",
        ))
    }

    #[tokio::test]
    async fn test_documents() {
        let input = serde_json::json!({ "documents": "document_id,type,completeness_score,risk_flags\nD1,Contract,0.95,0\nD2,NDA,0.75,2\nD3,Policy,0.6,1" });
        let r = DocumentReviewerAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["approved"], 1);
        assert_eq!(r["summary"]["needs_revision"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(DocumentReviewerAgent::new()
            .execute_with_context(serde_json::json!({"documents":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            DocumentReviewerAgent::new().manifest().id,
            "legal.document-reviewer"
        );
    }
}
