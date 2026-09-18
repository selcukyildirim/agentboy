use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct JournalEntryAgent;

impl JournalEntryAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for JournalEntryAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.journal-entry".to_string(),
            version: "2.0.0".to_string(),
            name: "Journal Entry".to_string(),
            department: "Accounting".to_string(),
            description: "Validate journal entries for double-entry balance, detect anomalies, suggest corrections".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
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
        let entries_csv = input["entries"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'entries' CSV".to_string()))?;

        let entries = csv_util::parse_csv_to_maps(entries_csv)?;
        if entries.is_empty() {
            return Err(AppError::Validation("No journal entries found".to_string()));
        }

        let mut total_debit = 0.0;
        let mut total_credit = 0.0;
        let mut by_account = std::collections::HashMap::new();
        let mut issues = Vec::new();

        for (i, entry) in entries.iter().enumerate() {
            let debit = csv_util::record_get_f64(entry, "debit");
            let credit = csv_util::record_get_f64(entry, "credit");
            let account = csv_util::record_get_str(entry, "account").to_string();

            if debit > 0.0 && credit > 0.0 {
                issues.push(serde_json::json!({
                    "row": i + 1,
                    "type": "both_debit_credit",
                    "severity": "high",
                    "account": account,
                    "description": "Entry has both debit and credit amounts",
                }));
            }
            if debit == 0.0 && credit == 0.0 {
                issues.push(serde_json::json!({
                    "row": i + 1,
                    "type": "zero_amount",
                    "severity": "medium",
                    "account": account,
                    "description": "Entry has zero debit and credit",
                }));
            }

            total_debit += debit;
            total_credit += credit;

            let acc_entry = by_account.entry(account.clone()).or_insert((0.0, 0.0));
            acc_entry.0 += debit;
            acc_entry.1 += credit;
        }

        let is_balanced = (total_debit - total_credit).abs() < 0.01;

        let account_summary: Vec<serde_json::Value> = by_account
            .iter()
            .map(|(account, (dr, cr))| {
                serde_json::json!({
                    "account": account,
                    "total_debit": dr,
                    "total_credit": cr,
                    "net": dr - cr,
                })
            })
            .collect();

        let system_prompt = "You are a senior accountant. Analyze the journal entries and identify errors, missing entries, or unusual patterns. Be concise.";
        let user_prompt = format!(
            "Journal Entry Validation:\n- Total entries: {}\n- Total debit: {:.2}\n- Total credit: {:.2}\n- Balanced: {}\n- Issues: {}\n\nAccount Summary:\n{}\n\nIssue Details:\n{}\n\nProvide accounting analysis.",
            entries.len(), total_debit, total_credit, is_balanced, issues.len(),
            serde_json::to_string_pretty(&account_summary).unwrap_or_default(),
            serde_json::to_string_pretty(&issues).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "status": if is_balanced && issues.is_empty() { "valid" } else if is_balanced { "valid_with_issues" } else { "unbalanced" },
            "summary": {
                "total_entries": entries.len(),
                "total_debit": total_debit,
                "total_credit": total_credit,
                "difference": total_debit - total_credit,
                "is_balanced": is_balanced,
                "issue_count": issues.len(),
            },
            "account_summary": account_summary,
            "issues": issues,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for JournalEntryAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Journal entries are balanced."))
    }

    #[tokio::test]
    async fn test_balanced_entries() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nCash,1000,0\nRevenue,0,1000"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "valid");
        assert!(result["summary"]["is_balanced"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_unbalanced_entries() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nCash,1000,0\nRevenue,0,500"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "unbalanced");
        assert_eq!(result["summary"]["difference"], 500.0);
    }

    #[tokio::test]
    async fn test_both_debit_credit_issue() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nCash,1000,500\nRevenue,0,1500"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["issue_count"], 1);
    }

    #[tokio::test]
    async fn test_zero_amount_issue() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nCash,0,0\nRevenue,0,0"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["issue_count"], 2);
    }

    #[tokio::test]
    async fn test_missing_entries() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_entries() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "entries": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_account_summary() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nCash,500,0\nExpense,500,0\nRevenue,0,1000"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let summary = result["account_summary"].as_array().unwrap();
        assert_eq!(summary.len(), 3);
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = JournalEntryAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "entries": "account,debit,credit\nA,100,0\nB,0,100"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = JournalEntryAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.journal-entry");
        assert!(m.permissions.network_llm);
    }
}