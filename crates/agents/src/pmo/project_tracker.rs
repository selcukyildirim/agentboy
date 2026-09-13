use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct ProjectTrackerAgent;
impl ProjectTrackerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for ProjectTrackerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "pmo.project-tracker".to_string(), version: "2.0.0".to_string(), name: "Project Tracker".to_string(), department: "PMO".to_string(), description: "Track project milestones, schedule variance, and critical path items".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["projects"].as_str().ok_or_else(|| AppError::Validation("Missing 'projects' CSV".to_string()))?;
        let projects = csv_util::parse_csv_to_maps(csv)?;
        if projects.is_empty() { return Err(AppError::Validation("No projects".to_string())); }

        let mut analysis: Vec<serde_json::Value> = projects.iter().map(|p| {
            let progress = csv_util::record_get_f64(p, "progress_pct");
            let schedule_var = csv_util::record_get_f64(p, "schedule_variance_days");
            let budget_var = csv_util::record_get_f64(p, "budget_variance");
            let status = if schedule_var > 5.0 || budget_var > 0.1 { "critical" } else if schedule_var > 0.0 { "at_risk" } else { "on_track" };
            serde_json::json!({ "name": csv_util::record_get_str(p, "name"), "manager": csv_util::record_get_str(p, "manager"), "progress_pct": progress, "schedule_variance_days": schedule_var, "budget_variance": budget_var, "status": status })
        }).collect();
        analysis.sort_by(|a, b| a["schedule_variance_days"].as_f64().unwrap_or(0.0).partial_cmp(&b["schedule_variance_days"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let critical = analysis.iter().filter(|a| a["status"] == "critical").count();
        let llm = ctx.call_llm("Analyze project portfolio.", &format!("Projects ({} total, {} critical):\n{}\n\nActions?", projects.len(), critical, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_projects": projects.len(), "critical": critical }, "projects": analysis, "llm_analysis": llm }))
    }
}
impl Default for ProjectTrackerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Project Alpha is critical.")) }

    #[tokio::test]
    async fn test_projects() {
        let input = serde_json::json!({ "projects": "name,manager,progress_pct,schedule_variance_days,budget_variance\nAlpha,Alice,30,10,0.15\nBeta,Bob,70,0,0.02" });
        let r = ProjectTrackerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["critical"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(ProjectTrackerAgent::new().execute_with_context(serde_json::json!({"projects":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(ProjectTrackerAgent::new().manifest().id, "pmo.project-tracker"); }
}