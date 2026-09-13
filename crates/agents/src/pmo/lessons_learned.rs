use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct LessonsLearnedAgent;
impl LessonsLearnedAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for LessonsLearnedAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "pmo.lessons-learned".to_string(), version: "2.0.0".to_string(), name: "Lessons Learned".to_string(), department: "PMO".to_string(), description: "Capture and analyze project lessons learned, identify patterns and improvements".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["lessons"].as_str().ok_or_else(|| AppError::Validation("Missing 'lessons' CSV".to_string()))?;
        let lessons = csv_util::parse_csv_to_maps(csv)?;
        if lessons.is_empty() { return Err(AppError::Validation("No lessons".to_string())); }

        let by_category = csv_util::group_by(&lessons, "category");
        let by_project = csv_util::group_by(&lessons, "project");

        let category_summary: Vec<serde_json::Value> = by_category.iter().map(|(cat, items)| {
            serde_json::json!({ "category": cat, "count": items.len(), "examples": items.iter().take(2).map(|l| csv_util::record_get_str(l, "description")).collect::<Vec<_>>() })
        }).collect();

        let llm = ctx.call_llm("Analyze lessons learned.", &format!("Lessons Learned ({} total, {} categories, {} projects):\n{}\n\nKey patterns?", lessons.len(), by_category.len(), by_project.len(), serde_json::to_string_pretty(&category_summary).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_lessons": lessons.len(), "categories": by_category.len(), "projects": by_project.len() }, "by_category": category_summary, "llm_analysis": llm }))
    }
}
impl Default for LessonsLearnedAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Communication is a recurring theme.")) }

    #[tokio::test]
    async fn test_lessons() {
        let input = serde_json::json!({ "lessons": "project,category,description\nA,Communication,Need daily standups\nA,Planning,Underestimated testing\nB,Communication,Stakeholders left in dark" });
        let r = LessonsLearnedAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["total_lessons"], 3);
        assert_eq!(r["summary"]["categories"], 2);
    }
    #[tokio::test]
    async fn test_empty() { assert!(LessonsLearnedAgent::new().execute_with_context(serde_json::json!({"lessons":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(LessonsLearnedAgent::new().manifest().id, "pmo.lessons-learned"); }
}