use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind};
use crate::csv_util;

pub struct BudgetTrackerAgent;
impl BudgetTrackerAgent { pub fn new() -> Self { Self } }

#[async_trait::async_trait]
impl Agent for BudgetTrackerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest { id: "management.budget-tracker".to_string(), version: "2.0.0".to_string(), name: "Budget Tracker".to_string(), department: "Management".to_string(), description: "Track budget execution across departments and forecast year-end position".to_string(), tier: AgentTier::Free, skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()], permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true }, execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 }, rag_enabled: false, output_schema: None, max_cost_usd: None,
            input_schema: vec![
                InputField::new("budgets", "Budgets", InputKind::File, true).with_example("department,budget,spent\nSales,100000,40000"),
            ],
        }
    }
    fn supports_context(&self) -> bool { true }
    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let csv = input["budgets"].as_str().ok_or_else(|| AppError::Validation("Missing 'budgets' CSV".to_string()))?;
        let budgets = csv_util::parse_csv_to_maps(csv)?;
        if budgets.is_empty() { return Err(AppError::Validation("No budgets".to_string())); }

        let mut analysis: Vec<serde_json::Value> = budgets.iter().map(|b| {
            let allocated = csv_util::record_get_f64(b, "allocated");
            let spent = csv_util::record_get_f64(b, "spent");
            let remaining = allocated - spent;
            let pct_used = if allocated > 0.0 { spent / allocated * 100.0 } else { 0.0 };
            let status = if pct_used > 100.0 { "over" } else if pct_used > 90.0 { "warning" } else { "ok" };
            serde_json::json!({ "department": csv_util::record_get_str(b, "department"), "allocated": allocated, "spent": spent, "remaining": remaining, "pct_used": pct_used, "status": status })
        }).collect();

        let total_allocated: f64 = analysis.iter().map(|a| a["allocated"].as_f64().unwrap_or(0.0)).sum();
        let total_spent: f64 = analysis.iter().map(|a| a["spent"].as_f64().unwrap_or(0.0)).sum();
        let over = analysis.iter().filter(|a| a["status"] == "over").count();

        let llm = ctx.call_llm("Analyze budget execution.", &format!("Budget ({} departments, total: {:.0}, spent: {:.0}, {} over):\n{}\n\nActions?", budgets.len(), total_allocated, total_spent, over, serde_json::to_string_pretty(&analysis).unwrap_or_default())).await?;
        Ok(serde_json::json!({ "summary": { "total_departments": budgets.len(), "total_allocated": total_allocated, "total_spent": total_spent, "over_budget": over }, "departments": analysis, "llm_analysis": llm }))
    }
}
impl Default for BudgetTrackerAgent { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext { MockAgentContext::new(MockLlmProvider::with_response("Marketing is over budget.")) }

    #[tokio::test]
    async fn test_budget() {
        let input = serde_json::json!({ "budgets": "department,allocated,spent\nEng,100000,85000\nMkt,50000,55000" });
        let r = BudgetTrackerAgent::new().execute_with_context(input, &ctx()).await.unwrap();
        assert_eq!(r["summary"]["over_budget"], 1);
    }
    #[tokio::test]
    async fn test_empty() { assert!(BudgetTrackerAgent::new().execute_with_context(serde_json::json!({"budgets":""}), &ctx()).await.is_err()); }
    #[test] fn test_manifest() { assert_eq!(BudgetTrackerAgent::new().manifest().id, "management.budget-tracker"); }
}