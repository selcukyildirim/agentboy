use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};
use crate::csv_util;

pub struct OKRAgent;
impl OKRAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for OKRAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "management.okr".to_string(), version: "2.0.0".to_string(), name: "OKR".to_string(), department: "Management".to_string(), description: "Track OKR progress across teams, identify at-risk key results, and align priorities".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 } }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["okrs"].as_str().ok_or_else(|| AppError::Validation("Missing 'okrs' CSV".to_string()))?;
        let okrs = csv_util::parse_csv_to_maps(csv)?;
        if okrs.is_empty() { return Err(AppError::Validation("No OKRs".to_string())); }

        let mut analysis: Vec<serde_json::Value> = okrs.iter().map(|o| {
            let progress = csv_util::record_get_f64(o, "progress_pct");
            let confidence = csv_util::record_get_f64(o, "confidence");
            let status = if progress >= 70.0 { "on_track" } else if progress >= 40.0 { "at_risk" } else { "behind" };
            serde_json::json!({ "objective": csv_util::record_get_str(o, "objective"), "team": csv_util::record_get_str(o, "team"), "progress_pct": progress, "confidence": confidence, "status": status })
        }).collect();
        analysis.sort_by(|a, b| a["progress_pct"].as_f64().unwrap_or(0.0).partial_cmp(&b["progress_pct"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let behind = analysis.iter().filter(|a| a["status"] == "behind").count();
        let on_track = analysis.iter().filter(|a| a["status"] == "on_track").count();

        let llm = ctx.call_llm("Analyze OKR progress.", &format!("OKRs ({} total, {} on track, {} behind):\n{}\n\nFocus areas?", okrs.len(), on_track, behind, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_okrs": okrs.len(), "on_track": on_track, "behind": behind }, "okrs": analysis, "llm_analysis": llm }))
    }
}
impl Default for OKRAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Focus on the behind OKR.")) }

    #[tokio::test]
    async fn test_okrs() {
        let input = serde_json::json!({ "okrs": "objective,team,progress_pct,confidence\nGrow Revenue,Sales,80,0.9\nReduce Churn,CS,20,0.5" });
        let r = OKRAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["on_track"], 1);
        assert_eq!(r["summary"]["behind"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(OKRAgent::new().execute_with_context(serde_json::json!({"okrs":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(OKRAgent::new().manifest().id, "management.okr"); }
}