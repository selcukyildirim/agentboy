use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};
use std::collections::HashMap;

use crate::csv_util;

pub struct FinancialRiskAgent;

impl FinancialRiskAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for FinancialRiskAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.financial-risk".to_string(),
            version: "2.0.0".to_string(),
            name: "Financial Risk".to_string(),
            department: "Finance".to_string(),
            description: "Assess financial risk from transaction data, identify concentration risk, volatility, and exposure patterns".to_string(),
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

        let transactions = csv_util::parse_csv_to_maps(transactions_csv)?;
        if transactions.is_empty() {
            return Err(AppError::Validation("No transactions found".to_string()));
        }

        let amounts: Vec<f64> = transactions
            .iter()
            .map(|r| csv_util::record_get_f64(r, "amount").abs())
            .collect();
        let total_exposure: f64 = amounts.iter().sum();
        let avg_transaction = if amounts.is_empty() {
            0.0
        } else {
            total_exposure / amounts.len() as f64
        };
        let max_transaction = amounts.iter().copied().fold(f64::MIN, f64::max);
        let std_dev = csv_util::standard_deviation(&amounts);
        let volatility = if avg_transaction > 0.0 {
            std_dev / avg_transaction
        } else {
            0.0
        };

        let by_counterparty = csv_util::sum_by(&transactions, "counterparty", "amount");

        let concentration_risk = if let Some((name, amount)) =
            by_counterparty.iter().max_by(|a, b| {
                a.1.abs()
                    .partial_cmp(&b.1.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
            let pct = if total_exposure > 0.0 {
                amount.abs() / total_exposure
            } else {
                0.0
            };
            serde_json::json!({
                "counterparty": name,
                "amount": amount,
                "percentage_of_total": format!("{:.1}%", pct * 100.0),
                "is_high_concentration": pct > 0.3,
            })
        } else {
            serde_json::json!(null)
        };

        let by_category = csv_util::sum_by(&transactions, "category", "amount");

        let risk_score =
            calculate_risk_score(&amounts, volatility, &by_counterparty, total_exposure);

        let system_prompt = "You are a financial risk analyst. Analyze the transaction data and provide a risk assessment covering concentration risk, volatility, exposure patterns, and recommended mitigations. Be concise and professional.";

        let user_prompt = format!(
            "Financial Risk Assessment:\n\
             - Total transactions: {}\n\
             - Total exposure: {:.2}\n\
             - Average transaction: {:.2}\n\
             - Max transaction: {:.2}\n\
             - Volatility (CV): {:.2}\n\
             - Risk Score: {}/100\n\n\
             Concentration by Counterparty:\n{}\n\n\
             Category Exposure:\n{}\n\n\
             Provide a risk assessment and recommended mitigations.",
            transactions.len(),
            total_exposure,
            avg_transaction,
            max_transaction,
            volatility,
            risk_score,
            serde_json::to_string_pretty(&concentration_risk).unwrap_or_default(),
            serde_json::to_string_pretty(&by_category).unwrap_or_default(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "risk_score": risk_score,
            "risk_level": if risk_score > 70 { "high" } else if risk_score > 40 { "medium" } else { "low" },
            "summary": {
                "total_transactions": transactions.len(),
                "total_exposure": total_exposure,
                "avg_transaction": avg_transaction,
                "max_transaction": max_transaction,
                "volatility": volatility,
            },
            "concentration_risk": concentration_risk,
            "counterparty_breakdown": by_counterparty,
            "category_exposure": by_category,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for FinancialRiskAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_risk_score(
    amounts: &[f64],
    volatility: f64,
    counterparty_amounts: &HashMap<String, f64>,
    total_exposure: f64,
) -> u32 {
    let mut score: u32 = 0;

    score += (volatility * 30.0).min(30.0) as u32;

    if let Some(max_cp) = counterparty_amounts.values().max_by(|a, b| {
        a.abs()
            .partial_cmp(&b.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        let pct = if total_exposure > 0.0 {
            max_cp.abs() / total_exposure
        } else {
            0.0
        };
        score += (pct * 40.0).min(40.0) as u32;
    }

    let max_amount = amounts.iter().copied().fold(f64::MIN, f64::max);
    let avg = if amounts.is_empty() {
        0.0
    } else {
        total_exposure / amounts.len() as f64
    };
    if avg > 0.0 {
        let size_ratio = max_amount / avg;
        score += (size_ratio * 10.0).min(20.0) as u32;
    }

    if amounts.len() > 10 {
        score += 5;
    }

    if counterparty_amounts.len() <= 2 && !counterparty_amounts.is_empty() {
        score += 5;
    }

    score.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "High concentration risk with Acme Corp at 45%. Recommend diversifying suppliers.",
        ))
    }

    #[tokio::test]
    async fn test_risk_assessment() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,counterparty,category\n2024-01-01,-5000,Acme,Supplier\n2024-01-02,-3000,Beta,Supplier\n2024-01-03,-2000,Gamma,Utilities"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["risk_score"].as_u64().is_some());
        assert!(result["summary"]["total_exposure"].as_f64().unwrap() > 0.0);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_high_concentration() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,counterparty,category\n2024-01-01,-9000,Acme,Supplier\n2024-01-02,-1000,Beta,Utilities"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let conc = &result["concentration_risk"];
        assert_eq!(conc["is_high_concentration"], true);
    }

    #[tokio::test]
    async fn test_low_risk() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,counterparty,category\n2024-01-01,-100,A,Food\n2024-01-02,-100,B,Food\n2024-01-03,-100,C,Food\n2024-01-04,-100,D,Food"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["risk_level"], "low");
    }

    #[tokio::test]
    async fn test_missing_transactions() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_transactions() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "transactions": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_counterparty_breakdown() {
        let agent = FinancialRiskAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "transactions": "date,amount,counterparty,category\n2024-01-01,-500,X,Supplies\n2024-01-02,-300,Y,Services"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let cp = result["counterparty_breakdown"].as_object().unwrap();
        assert!(cp.contains_key("X"));
        assert!(cp.contains_key("Y"));
    }

    #[test]
    fn test_risk_score_calculation() {
        let amounts = vec![100.0, 100.0, 100.0];
        let mut cp = std::collections::HashMap::new();
        cp.insert("A".to_string(), 300.0);
        let score = calculate_risk_score(&amounts, 0.0, &cp, 300.0);
        assert!(score <= 100);
    }

    #[test]
    fn test_manifest() {
        let agent = FinancialRiskAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.financial-risk");
        assert!(m.permissions.network_llm);
    }
}
