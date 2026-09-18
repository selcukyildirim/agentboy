use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct CashFlowForecastAgent;

impl CashFlowForecastAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for CashFlowForecastAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.cashflow-forecast".to_string(),
            version: "2.0.0".to_string(),
            name: "Cash Flow Forecast".to_string(),
            department: "Finance".to_string(),
            description: "Forecast cash flow based on historical transactions, identify gaps and liquidity risks".to_string(),
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
            input_schema: vec![
                InputField::new("transactions", "Transactions", InputKind::File, true).with_example("date,amount,category\n2024-01-01,1000,inflow\n2024-01-02,500,outflow"),
                InputField::new("opening_balance", "Opening Balance", InputKind::Number, true).with_example("10000"),
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
        let transactions_csv = input["transactions"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'transactions' CSV string".to_string()))?;
        let opening_balance = input["opening_balance"].as_f64().unwrap_or(0.0);

        let transactions = csv_util::parse_csv_to_maps(transactions_csv)?;
        if transactions.is_empty() {
            return Err(AppError::Validation("No transactions found".to_string()));
        }

        let mut inflows = 0.0;
        let mut outflows = 0.0;
        let mut by_category = std::collections::HashMap::new();
        let mut monthly = std::collections::HashMap::<String, (f64, f64)>::new();

        for txn in &transactions {
            let amount = csv_util::record_get_f64(txn, "amount");
            let date = csv_util::record_get_str(txn, "date");
            let category = csv_util::record_get_str(txn, "category");

            if amount > 0.0 {
                inflows += amount;
            } else {
                outflows += amount.abs();
            }

            *by_category.entry(category.to_string()).or_insert(0.0) += amount;

            if date.len() >= 7 {
                let month = &date[..7];
                let entry = monthly.entry(month.to_string()).or_insert((0.0, 0.0));
                if amount > 0.0 {
                    entry.0 += amount;
                } else {
                    entry.1 += amount.abs();
                }
            }
        }

        let net_cashflow = inflows - outflows;
        let ending_balance = opening_balance + net_cashflow;
        let avg_monthly_inflow = if !monthly.is_empty() {
            monthly.values().map(|(i, _)| i).sum::<f64>() / monthly.len() as f64
        } else {
            0.0
        };
        let avg_monthly_outflow = if !monthly.is_empty() {
            monthly.values().map(|(_, o)| o).sum::<f64>() / monthly.len() as f64
        } else {
            0.0
        };

        let months_of_runway = if avg_monthly_outflow > 0.0 {
            ending_balance / avg_monthly_outflow
        } else {
            f64::INFINITY
        };

        let lowest_month_balance = monthly
            .values()
            .scan(opening_balance, |balance, (inflow, outflow)| {
                *balance += inflow - outflow;
                Some(*balance)
            })
            .fold(f64::INFINITY, f64::min);

        let system_prompt = "You are a cash flow analyst. Analyze the cash flow data and provide insights on liquidity position, cash flow trends, seasonal patterns, and funding needs. Be concise and professional.";

        let user_prompt = format!(
            "Cash Flow Forecast:\n\
             - Opening balance: {:.2}\n\
             - Total inflows: {:.2}\n\
             - Total outflows: {:.2}\n\
             - Net cash flow: {:.2}\n\
             - Ending balance: {:.2}\n\
             - Months of runway: {:.1}\n\
             - Lowest projected balance: {:.2}\n\n\
             Monthly Trend:\n{}\n\n\
             Category Breakdown:\n{}\n\n\
             Provide cash flow analysis, liquidity assessment, and recommendations.",
            opening_balance,
            inflows,
            outflows,
            net_cashflow,
            ending_balance,
            if months_of_runway.is_infinite() {
                999.0
            } else {
                months_of_runway
            },
            lowest_month_balance,
            serde_json::to_string_pretty(&monthly).unwrap_or_default(),
            serde_json::to_string_pretty(&by_category).unwrap_or_default(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "opening_balance": opening_balance,
                "total_inflows": inflows,
                "total_outflows": outflows,
                "net_cash_flow": net_cashflow,
                "ending_balance": ending_balance,
                "months_of_runway": if months_of_runway.is_infinite() { serde_json::Value::Null } else { serde_json::json!(months_of_runway) },
                "lowest_balance": lowest_month_balance,
                "liquidity_risk": if ending_balance < 0.0 { "critical" } else if months_of_runway < 3.0 { "high" } else if months_of_runway < 6.0 { "medium" } else { "low" },
            },
            "monthly_trend": monthly,
            "category_breakdown": by_category,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for CashFlowForecastAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "Cash flow is healthy with 6 months runway. Consider building reserves.",
        ))
    }

    #[tokio::test]
    async fn test_cashflow_forecast() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,category\n2024-01-01,5000,Revenue\n2024-01-15,-3000,Expenses\n2024-02-01,6000,Revenue\n2024-02-15,-3500,Expenses",
            "opening_balance": 10000
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["opening_balance"], 10000.0);
        assert_eq!(result["summary"]["total_inflows"], 11000.0);
        assert_eq!(result["summary"]["total_outflows"], 6500.0);
        assert_eq!(result["summary"]["net_cash_flow"], 4500.0);
        assert_eq!(result["summary"]["ending_balance"], 14500.0);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_negative_balance() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,category\n2024-01-01,1000,Rev\n2024-01-15,-5000,Exp",
            "opening_balance": 2000
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["ending_balance"], -2000.0);
        assert_eq!(result["summary"]["liquidity_risk"], "critical");
    }

    #[tokio::test]
    async fn test_monthly_trend() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,category\n2024-01-01,1000,Rev\n2024-01-15,-500,Exp\n2024-02-01,1200,Rev\n2024-02-15,-600,Exp",
            "opening_balance": 5000
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let trend = result["monthly_trend"].as_object().unwrap();
        assert!(trend.contains_key("2024-01"));
        assert!(trend.contains_key("2024-02"));
    }

    #[tokio::test]
    async fn test_missing_transactions() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_transactions() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "transactions": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_category_breakdown() {
        let agent = CashFlowForecastAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,category\n2024-01-01,1000,Sales\n2024-01-02,500,Services\n2024-01-03,-300,Rent",
            "opening_balance": 0
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let cats = result["category_breakdown"].as_object().unwrap();
        assert!(cats.contains_key("Sales"));
        assert!(cats.contains_key("Rent"));
    }

    #[test]
    fn test_manifest() {
        let agent = CashFlowForecastAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.cashflow-forecast");
        assert!(m.permissions.network_llm);
    }
}
