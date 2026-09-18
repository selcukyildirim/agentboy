use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct BudgetVarianceAgent;

impl BudgetVarianceAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for BudgetVarianceAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.budget-variance".to_string(),
            version: "2.0.0".to_string(),
            name: "Budget Variance".to_string(),
            department: "Finance".to_string(),
            description: "Compare budget vs actual spending, identify material deviations with LLM-powered insights".to_string(),
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
        let budget_csv = input["budget"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'budget' CSV string".to_string()))?;
        let actual_csv = input["actual"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'actual' CSV string".to_string()))?;
        let threshold = input["threshold"].as_f64().unwrap_or(0.1);

        let budget = csv_util::parse_csv_column_f64(budget_csv, "category", "amount")?;
        let actual = csv_util::parse_csv_column_f64(actual_csv, "category", "amount")?;

        let mut variances = Vec::new();
        let mut material_deviations = Vec::new();

        for (category, budget_amount) in &budget {
            let actual_amount = actual.get(category).copied().unwrap_or(0.0);
            let variance = actual_amount - budget_amount;
            let variance_pct = if *budget_amount != 0.0 {
                variance / budget_amount.abs()
            } else {
                0.0
            };
            let is_material = variance_pct.abs() > threshold;

            let entry = serde_json::json!({
                "category": category,
                "budget": budget_amount,
                "actual": actual_amount,
                "variance": variance,
                "variance_pct": format!("{:.1}%", variance_pct * 100.0),
                "is_material": is_material,
                "direction": if variance > 0.0 { "over_budget" } else if variance < 0.0 { "under_budget" } else { "on_track" },
            });

            if is_material {
                material_deviations.push(entry.clone());
            }
            variances.push(entry);
        }

        let total_budget: f64 = budget.values().sum();
        let total_actual: f64 = actual.values().sum();
        let total_variance = total_actual - total_budget;

        let system_prompt = "You are a financial analyst specializing in budget variance analysis. Analyze the variance report and provide actionable insights on material deviations. Be concise and professional.";

        let user_prompt = format!(
            "Budget Variance Report:\n\
             - Total Budget: {:.2}\n\
             - Total Actual: {:.2}\n\
             - Total Variance: {:.2} ({:.1}%)\n\
             - Categories analyzed: {}\n\
             - Material deviations (>{:.0}%): {}\n\n\
             Category Breakdown:\n{}\n\n\
             Material Deviations:\n{}\n\n\
             Provide analysis of key variances and recommended actions.",
            total_budget, total_actual, total_variance,
            if total_budget != 0.0 { total_variance / total_budget.abs() * 100.0 } else { 0.0 },
            variances.len(),
            threshold * 100.0,
            material_deviations.len(),
            serde_json::to_string_pretty(&variances).unwrap_or_default(),
            serde_json::to_string_pretty(&material_deviations).unwrap_or_default(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_budget": total_budget,
                "total_actual": total_actual,
                "total_variance": total_variance,
                "total_variance_pct": format!("{:.1}%", if total_budget != 0.0 { total_variance / total_budget.abs() * 100.0 } else { 0.0 }),
                "categories_analyzed": variances.len(),
                "material_deviations_count": material_deviations.len(),
                "threshold_pct": threshold * 100.0,
            },
            "variances": variances,
            "material_deviations": material_deviations,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for BudgetVarianceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Marketing is 20% over budget. Recommend reviewing ad spend. Engineering is slightly under budget."))
    }

    #[tokio::test]
    async fn test_budget_variance() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "budget": "category,amount\nMarketing,10000\nEngineering,50000\nSales,20000",
            "actual": "category,amount\nMarketing,12000\nEngineering,48000\nSales,25000",
            "threshold": 0.1
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["categories_analyzed"], 3);
        assert_eq!(result["summary"]["material_deviations_count"], 2);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_no_material_deviations() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "budget": "category,amount\nFood,1000\nRent,2000",
            "actual": "category,amount\nFood,1020\nRent,1980",
            "threshold": 0.1
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["material_deviations_count"], 0);
    }

    #[tokio::test]
    async fn test_all_over_budget() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "budget": "category,amount\nA,100\nB,200",
            "actual": "category,amount\nA,150\nB,300",
            "threshold": 0.05
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["material_deviations_count"], 2);
    }

    #[tokio::test]
    async fn test_missing_budget() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "actual": "category,amount\nA,100" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_missing_actual() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "budget": "category,amount\nA,100" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_default_threshold() {
        let agent = BudgetVarianceAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "budget": "category,amount\nA,1000",
            "actual": "category,amount\nA,1005"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["material_deviations_count"], 0);
    }

    #[test]
    fn test_manifest() {
        let agent = BudgetVarianceAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.budget-variance");
        assert!(m.permissions.network_llm);
    }
}