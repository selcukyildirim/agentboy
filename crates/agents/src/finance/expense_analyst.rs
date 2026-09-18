use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct ExpenseAnalystAgent;

impl ExpenseAnalystAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for ExpenseAnalystAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.expense-analyst".to_string(),
            version: "2.0.0".to_string(),
            name: "Expense Analyst".to_string(),
            department: "Finance".to_string(),
            description: "Analyze expenses by category, detect anomalies, duplicates and unusual patterns with LLM insights".to_string(),
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
                max_steps: 40,
                timeout_seconds: 180,
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
        let expense_csv = input["expenses"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'expenses' CSV string".to_string()))?;

        let expenses = csv_util::parse_csv_to_maps(expense_csv)?;
        if expenses.is_empty() {
            return Err(AppError::Validation("No expense records found".to_string()));
        }

        let total_amount: f64 = expenses
            .iter()
            .map(|r| csv_util::record_get_f64(r, "amount"))
            .sum();

        let category_summary = csv_util::sum_by(&expenses, "category", "amount");

        let duplicates = csv_util::detect_duplicates(&expenses, &["date", "amount", "description"]);

        let amounts: Vec<f64> = expenses
            .iter()
            .map(|r| csv_util::record_get_f64(r, "amount"))
            .collect();
        let avg = if !amounts.is_empty() {
            amounts.iter().sum::<f64>() / amounts.len() as f64
        } else {
            0.0
        };
        let std_dev = csv_util::standard_deviation(&amounts);
        let p95 = csv_util::percentile(&amounts, 95.0);

        let mut anomalies = Vec::new();
        for expense in &expenses {
            let amount = csv_util::record_get_f64(expense, "amount");
            if std_dev > 0.0 && (amount - avg).abs() > 2.0 * std_dev {
                anomalies.push(serde_json::json!({
                    "record": csv_util::csv_record_to_json(expense),
                    "reason": "Amount exceeds 2 standard deviations from mean",
                    "amount": amount,
                    "category_mean": avg,
                    "category_std_dev": std_dev,
                }));
            }
        }

        let system_prompt = "You are a financial expense analyst. Analyze the expense data and provide insights on spending patterns, potential savings, unusual transactions, and policy compliance concerns. Be concise and professional.";

        let user_prompt = format!(
            "Expense Analysis Report:\n\
             - Total expenses: {} records, total amount: {:.2}\n\
             - Average per expense: {:.2}\n\
             - Standard deviation: {:.2}\n\
             - 95th percentile: {:.2}\n\
             - Anomalies detected: {}\n\
             - Potential duplicates: {}\n\n\
             Category Breakdown:\n{}\n\n\
             Anomalies:\n{}\n\n\
             Potential Duplicates:\n{}\n\n\
             Provide analysis of spending patterns and recommendations.",
            expenses.len(), total_amount, avg, std_dev, p95,
            anomalies.len(),
            duplicates.len(),
            serde_json::to_string_pretty(&category_summary).unwrap_or_default(),
            serde_json::to_string_pretty(&anomalies).unwrap_or_default(),
            duplicates.len(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_expenses": expenses.len(),
                "total_amount": total_amount,
                "average_amount": avg,
                "std_dev": std_dev,
                "p95": p95,
                "anomaly_count": anomalies.len(),
                "duplicate_count": duplicates.len(),
            },
            "category_summary": category_summary,
            "anomalies": anomalies,
            "duplicate_count": duplicates.len(),
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for ExpenseAnalystAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Food expenses are high. Consider budgeting."))
    }

    #[tokio::test]
    async fn test_expense_analysis() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "expenses": "date,category,amount,description\n2024-01-01,Food,50,Lunch\n2024-01-02,Transport,30,Taxi\n2024-01-03,Food,60,Dinner\n2024-01-04,Office,200,Supplies"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total_expenses"], 4);
        assert!(result["summary"]["total_amount"].as_f64().unwrap() > 0.0);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "expenses": "date,category,amount,description\n2024-01-01,Food,50,L1\n2024-01-02,Food,52,L2\n2024-01-03,Food,48,L3\n2024-01-04,Food,51,L4\n2024-01-05,Food,49,L5\n2024-01-06,Food,53,L6\n2024-01-07,Food,47,L7\n2024-01-08,Food,50,L8\n2024-01-09,Food,52,L9\n2024-01-10,Food,48,L10\n2024-01-11,Food,50000,Luxury"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["anomaly_count"], 1);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "expenses": "date,category,amount,description\n2024-01-01,Food,50,Lunch\n2024-01-01,Food,50,Lunch\n2024-01-02,Transport,30,Taxi"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["duplicate_count"], 1);
    }

    #[tokio::test]
    async fn test_missing_expenses() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_expenses() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "expenses": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_category_breakdown() {
        let agent = ExpenseAnalystAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "expenses": "date,category,amount,desc\n2024-01-01,Food,100,Lunch\n2024-01-02,Food,200,Dinner\n2024-01-03,Transport,50,Bus"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let cats = result["category_summary"].as_object().unwrap();
        assert!(cats.contains_key("Food"));
        assert!(cats.contains_key("Transport"));
    }

    #[test]
    fn test_manifest() {
        let agent = ExpenseAnalystAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.expense-analyst");
        assert!(m.permissions.network_llm);
    }
}