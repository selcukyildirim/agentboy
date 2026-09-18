use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct StakeholderReportAgent;
impl StakeholderReportAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for StakeholderReportAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "pmo.stakeholder-report".to_string(),
            version: "2.0.0".to_string(),
            name: "Stakeholder Report".to_string(),
            department: "PMO".to_string(),
            description:
                "Generate stakeholder-specific project status reports with tailored messaging"
                    .to_string(),
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
                max_steps: 30,
                timeout_seconds: 120,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("projects", "Projects", InputKind::File, true).with_example(
                    "project_id,name,status,budget,spent\nP1,Alpha,active,100000,40000",
                ),
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
        let csv = input["projects"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'projects' CSV".to_string()))?;
        let projects = csv_util::parse_csv_to_maps(csv)?;
        if projects.is_empty() {
            return Err(AppError::Validation("No projects".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = projects.iter().map(|p| {
            let progress = csv_util::record_get_f64(p, "progress_pct");
            let budget_var = csv_util::record_get_f64(p, "budget_variance");
            let schedule_var = csv_util::record_get_f64(p, "schedule_variance_days");
            let health = if schedule_var > 5.0 || budget_var > 0.1 { "red" } else if schedule_var > 0.0 { "amber" } else { "green" };
            serde_json::json!({ "name": csv_util::record_get_str(p, "name"), "progress_pct": progress, "budget_variance": budget_var, "schedule_variance_days": schedule_var, "health": health })
        }).collect();

        let green = analysis.iter().filter(|a| a["health"] == "green").count();
        let red = analysis.iter().filter(|a| a["health"] == "red").count();

        let llm = ctx
            .call_llm(
                "Generate stakeholder report.",
                &format!(
                    "Portfolio ({} projects, {} green, {} red):\n{}\n\nDraft stakeholder update.",
                    projects.len(),
                    green,
                    red,
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_projects": projects.len(), "green": green, "red": red }, "projects": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for StakeholderReportAgent {
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
            "Project Alpha is red. Escalate.",
        ))
    }

    #[tokio::test]
    async fn test_stakeholder() {
        let input = serde_json::json!({ "projects": "name,progress_pct,budget_variance,schedule_variance_days\nAlpha,30,0.15,10\nBeta,80,0.02,0" });
        let r = StakeholderReportAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["red"], 1);
        assert_eq!(r["summary"]["green"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(StakeholderReportAgent::new()
            .execute_with_context(serde_json::json!({"projects":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            StakeholderReportAgent::new().manifest().id,
            "pmo.stakeholder-report"
        );
    }
}
