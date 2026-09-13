use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct BankReconciliationAgent;

impl BankReconciliationAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for BankReconciliationAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "finance.bank-reconciliation".to_string(),
            version: "2.0.0".to_string(),
            name: "Bank Reconciliation".to_string(),
            department: "Finance".to_string(),
            description: "Match bank statement records with ledger entries and identify discrepancies with LLM-powered analysis".to_string(),
            tier: AgentTier::Free,
            skills: vec![
                "spreadsheet.parse".to_string(),
                "spreadsheet.compare".to_string(),
                "llm.analysis".to_string(),
            ],
            permissions: AgentPermissions {
                filesystem_read: true,
                filesystem_write: false,
                network_llm: true,
            },
            execution: ExecutionLimits {
                max_steps: 50,
                timeout_seconds: 300,
            },
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
        let bank_csv = input["bank_statement"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'bank_statement' CSV string".to_string()))?;
        let ledger_csv = input["ledger"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'ledger' CSV string".to_string()))?;
        let tolerance = input["tolerance"].as_f64().unwrap_or(0.01);

        let bank_records = csv_util::parse_csv_to_maps(bank_csv)?;
        let ledger_records = csv_util::parse_csv_to_maps(ledger_csv)?;

        let mut matched = Vec::new();
        let mut unmatched_bank = Vec::new();
        let mut unmatched_ledger = Vec::new();
        let mut ledger_used = vec![false; ledger_records.len()];

        for bank_rec in &bank_records {
            let bank_amount = csv_util::record_get_f64(bank_rec, "amount");
            let bank_date = csv_util::record_get_str(bank_rec, "date");
            let bank_ref = csv_util::record_get_str(bank_rec, "reference");

            let mut found = false;
            for (i, ledger_rec) in ledger_records.iter().enumerate() {
                if ledger_used[i] {
                    continue;
                }
                let ledger_amount = csv_util::record_get_f64(ledger_rec, "amount");
                let ledger_date = csv_util::record_get_str(ledger_rec, "date");

                let amount_match = (bank_amount - ledger_amount).abs() <= tolerance;
                let date_match = bank_date == ledger_date;
                let ref_match = if !bank_ref.is_empty() {
                    bank_ref == csv_util::record_get_str(ledger_rec, "reference")
                } else {
                    true
                };

                if amount_match && date_match && ref_match {
                    matched.push(serde_json::json!({
                        "bank": csv_util::csv_record_to_json(bank_rec),
                        "ledger": csv_util::csv_record_to_json(ledger_rec),
                        "match_type": if bank_ref == csv_util::record_get_str(ledger_rec, "reference") { "exact" } else { "fuzzy" },
                        "amount": bank_amount,
                    }));
                    ledger_used[i] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                unmatched_bank.push(csv_util::csv_record_to_json(bank_rec));
            }
        }

        for (i, ledger_rec) in ledger_records.iter().enumerate() {
            if !ledger_used[i] {
                unmatched_ledger.push(csv_util::csv_record_to_json(ledger_rec));
            }
        }

        let total_bank: f64 = bank_records.iter().map(|r| csv_util::record_get_f64(r, "amount")).sum();
        let total_ledger: f64 = ledger_records.iter().map(|r| csv_util::record_get_f64(r, "amount")).sum();
        let difference = total_bank - total_ledger;

        let system_prompt = "You are a financial reconciliation expert. Analyze the bank reconciliation results and provide insights on discrepancies, potential causes, and recommended actions. Be concise and professional.";

        let user_prompt = format!(
            "Bank Ledger Reconciliation Results:\n\
             - Total bank transactions: {} (amount: {:.2})\n\
             - Total ledger transactions: {} (amount: {:.2})\n\
             - Amount difference: {:.2}\n\
             - Matched: {} transactions\n\
             - Unmatched bank: {} transactions\n\
             - Unmatched ledger: {} transactions\n\n\
             Unmatched Bank Records: {}\n\
             Unmatched Ledger Records: {}\n\n\
             Provide a brief analysis of the reconciliation status and any concerns.",
            bank_records.len(), total_bank,
            ledger_records.len(), total_ledger,
            difference,
            matched.len(),
            unmatched_bank.len(),
            unmatched_ledger.len(),
            serde_json::to_string_pretty(&unmatched_bank).unwrap_or_default(),
            serde_json::to_string_pretty(&unmatched_ledger).unwrap_or_default(),
        );

        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "status": if unmatched_bank.is_empty() && unmatched_ledger.is_empty() { "reconciled" } else { "discrepancies_found" },
            "summary": {
                "total_bank_transactions": bank_records.len(),
                "total_ledger_transactions": ledger_records.len(),
                "total_bank_amount": total_bank,
                "total_ledger_amount": total_ledger,
                "difference": difference,
            },
            "matched_count": matched.len(),
            "matched": matched,
            "unmatched_bank_count": unmatched_bank.len(),
            "unmatched_bank": unmatched_bank,
            "unmatched_ledger_count": unmatched_ledger.len(),
            "unmatched_ledger": unmatched_ledger,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for BankReconciliationAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Reconciliation looks good. 1 unmatched transaction on each side suggests timing differences."))
    }

    #[tokio::test]
    async fn test_full_reconciliation() {
        let agent = BankReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "bank_statement": "date,amount,reference\n2024-01-01,100,REF001\n2024-01-02,200,REF002\n2024-01-03,150,REF003",
            "ledger": "date,amount,reference\n2024-01-01,100,REF001\n2024-01-02,200,REF002\n2024-01-04,300,REF004"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "discrepancies_found");
        assert_eq!(result["matched_count"], 2);
        assert_eq!(result["unmatched_bank_count"], 1);
        assert_eq!(result["unmatched_ledger_count"], 1);
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_perfect_reconciliation() {
        let agent = BankReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "bank_statement": "date,amount,reference\n2024-01-01,100,REF001",
            "ledger": "date,amount,reference\n2024-01-01,100,REF001"
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "reconciled");
        assert_eq!(result["matched_count"], 1);
    }

    #[tokio::test]
    async fn test_tolerance_matching() {
        let agent = BankReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "bank_statement": "date,amount,reference\n2024-01-01,100.05,REF001",
            "ledger": "date,amount,reference\n2024-01-01,100.00,REF001",
            "tolerance": 0.1
        });

        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["matched_count"], 1);
    }

    #[tokio::test]
    async fn test_missing_bank_statement() {
        let agent = BankReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "ledger": "date,amount\n2024-01-01,100" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_csv() {
        let agent = BankReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "bank_statement": "date,amount,reference",
            "ledger": "date,amount,reference"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "reconciled");
        assert_eq!(result["matched_count"], 0);
    }

    #[test]
    fn test_manifest() {
        let agent = BankReconciliationAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "finance.bank-reconciliation");
        assert_eq!(m.tier, AgentTier::Free);
        assert!(m.permissions.network_llm);
    }
}