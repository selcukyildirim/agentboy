use agent_runtime::agent::Agent;
use agent_runtime::{MockAgentContext, MockLlmProvider};
use agents::{BankReconciliationAgent, ExpenseAnalystAgent, SupplierComparisonAgent};
use std::fs;

fn fixture(name: &str) -> String {
    let path = format!(
        "{}/../../test-fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {name}: {e}"))
}

fn mock_ctx() -> MockAgentContext {
    MockAgentContext::new(MockLlmProvider::with_response(
        "Golden analysis: reconciliation looks healthy.",
    ))
}

#[tokio::test]
async fn golden_bank_reconciliation() {
    let agent = BankReconciliationAgent::new();
    let ctx = mock_ctx();

    let input = serde_json::json!({
        "bank_statement": fixture("bank_statement.csv"),
        "ledger": fixture("ledger_entries.csv"),
    });

    let result = agent.execute_with_context(input, &ctx).await.unwrap();

    assert_eq!(result["summary"]["total_bank_transactions"], 10);
    assert_eq!(result["summary"]["total_ledger_transactions"], 8);
    assert!(result["matched_count"].as_u64().unwrap() >= 7);
    assert!(result["unmatched_bank_count"].as_u64().unwrap() >= 2);
    let total = result["summary"]["total_bank_amount"].as_f64().unwrap();
    assert!((total - 26552.00).abs() < 0.01, "unexpected total: {total}");
    assert!(result["llm_analysis"].as_str().is_some());
}

#[tokio::test]
async fn golden_expense_analyst() {
    let agent = ExpenseAnalystAgent::new();
    let ctx = mock_ctx();

    let input = serde_json::json!({
        "expenses": fixture("bank_statement.csv"),
    });

    let result = agent.execute_with_context(input, &ctx).await.unwrap();
    assert!(
        result.get("summary").is_some()
            || result.get("total").is_some()
            || result.get("by_category").is_some()
    );
}

#[tokio::test]
async fn golden_supplier_comparison() {
    let agent = SupplierComparisonAgent::new();
    let ctx = mock_ctx();

    let input = serde_json::json!({
        "suppliers": [
            {"name": "Acme Corp", "price_score": 0.8, "quality_score": 0.95, "delivery_score": 0.9, "reliability_score": 0.92},
            {"name": "Beta LLC", "price_score": 0.9, "quality_score": 0.85, "delivery_score": 0.8, "reliability_score": 0.85},
            {"name": "Gamma Inc", "price_score": 0.7, "quality_score": 0.98, "delivery_score": 0.95, "reliability_score": 0.96},
        ],
    });

    let result = agent.execute_with_context(input, &ctx).await.unwrap();
    assert!(result.is_object());
    assert!(
        result.get("ranking").is_some()
            || result.get("suppliers").is_some()
            || result.get("recommendation").is_some()
    );
}

#[tokio::test]
async fn golden_empty_reconciliation() {
    let agent = BankReconciliationAgent::new();
    let ctx = mock_ctx();

    let input = serde_json::json!({
        "bank_statement": "date,amount,reference",
        "ledger": "date,amount,reference",
    });

    let result = agent.execute_with_context(input, &ctx).await.unwrap();
    assert_eq!(result["matched_count"], 0);
}
