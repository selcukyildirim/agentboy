use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};
use crate::csv_util;

pub struct KPIReporterAgent;
impl KPIReporterAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for KPIReporterAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "management.kpi-reporter".to_string(), version: "2.0.0".to_string(), name: "KPI Reporter".to_string(), department: "Management".to_string(), description: "Track KPIs across departments, identify trends, and highlight performance gaps".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 }, rag_enabled: false, output_schema: None, max_cost_usd: None,
            input_schema: vec![
                InputField::new("kpis", "KPIs", InputKind::File, true).with_example("kpi,target,actual,unit\nRevenue,100000,95000,USD"),
            ],
        }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["kpis"].as_str().ok_or_else(|| AppError::Validation("Missing 'kpis' CSV".to_string()))?;
        let kpis = csv_util::parse_csv_to_maps(csv)?;
        if kpis.is_empty() { return Err(AppError::Validation("No KPIs".to_string())); }

        let mut analysis: Vec<serde_json::Value> = kpis.iter().map(|k| {
            let current = csv_util::record_get_f64(k, "current_value");
            let target = csv_util::record_get_f64(k, "target_value");
            let variance = if target != 0.0 { (current - target) / target * 100.0 } else { 0.0 };
            let status = if variance >= 0.0 { "on_track" } else if variance >= -10.0 { "at_risk" } else { "off_track" };
            serde_json::json!({ "name": csv_util::record_get_str(k, "name"), "department": csv_util::record_get_str(k, "department"), "current_value": current, "target_value": target, "variance_pct": variance, "status": status })
        }).collect();
        analysis.sort_by(|a, b| a["variance_pct"].as_f64().unwrap_or(0.0).partial_cmp(&b["variance_pct"].as_f64().unwrap_or(0.0)).unwrap_or(std::cmp::Ordering::Equal));

        let off_track = analysis.iter().filter(|a| a["status"] == "off_track").count();
        let on_track = analysis.iter().filter(|a| a["status"] == "on_track").count();

        let llm = ctx.call_llm("Analyze KPI performance.", &format!("KPIs ({} total, {} on track, {} off track):\n{}\n\nActions?", kpis.len(), on_track, off_track, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_kpis": kpis.len(), "on_track": on_track, "off_track": off_track }, "kpis": analysis, "llm_analysis": llm }))
    }
}
impl Default for KPIReporterAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Revenue KPI is off track.")) }

    #[tokio::test]
    async fn test_kpis() {
        let input = serde_json::json!({ "kpis": "name,department,current_value,target_value\nRevenue,Finance,800,1000\nNPS,Sales,45,40\nDefects,Ops,2,5" });
        let r = KPIReporterAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["total_kpis"], 3);
        assert_eq!(r["summary"]["off_track"], 2);
    }
    #[tokio::test]
    async fn test_empty() { assert!(KPIReporterAgent::new().execute_with_context(serde_json::json!({"kpis":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(KPIReporterAgent::new().manifest().id, "management.kpi-reporter"); }
}