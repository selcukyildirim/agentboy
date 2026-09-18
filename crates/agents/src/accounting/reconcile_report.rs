use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct ReconcileReportAgent;

impl ReconcileReportAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for ReconcileReportAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.reconcile-report".to_string(),
            version: "2.0.0".to_string(),
            name: "Reconcile Report".to_string(),
            department: "Accounting".to_string(),
            description:
                "Generate reconciliation report across multiple accounts, flag unresolved items"
                    .to_string(),
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
            input_schema: vec![InputField::new(
                "accounts",
                "Accounts",
                InputKind::File,
                true,
            )],
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
        let accounts_csv = input["accounts"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'accounts' CSV".to_string()))?;

        let accounts = csv_util::parse_csv_to_maps(accounts_csv)?;
        if accounts.is_empty() {
            return Err(AppError::Validation("No accounts found".to_string()));
        }

        let mut account_reports = Vec::new();
        let mut total_discrepancies = 0.0;
        let mut reconciled_count = 0;
        let mut unreconciled_count = 0;

        for account in &accounts {
            let account_name = csv_util::record_get_str(account, "account");
            let gl_balance = csv_util::record_get_f64(account, "gl_balance");
            let subledger_balance = csv_util::record_get_f64(account, "subledger_balance");
            let bank_balance = csv_util::record_get_f64(account, "bank_balance");
            let status = csv_util::record_get_str(account, "status");

            let difference = gl_balance - subledger_balance;
            let abs_diff = difference.abs();
            total_discrepancies += abs_diff;

            let account_status = if status == "reconciled" {
                "reconciled"
            } else if abs_diff < 0.01 {
                "reconciled"
            } else if abs_diff < 10.0 {
                "minor_discrepancy"
            } else {
                "major_discrepancy"
            };

            if account_status == "reconciled" {
                reconciled_count += 1;
            } else {
                unreconciled_count += 1;
            }

            account_reports.push(serde_json::json!({
                "account": account_name,
                "gl_balance": gl_balance,
                "subledger_balance": subledger_balance,
                "bank_balance": bank_balance,
                "difference": difference,
                "abs_difference": abs_diff,
                "status": account_status,
            }));
        }

        let total_accounts = accounts.len();
        let reconciliation_rate = if total_accounts > 0 {
            reconciled_count as f64 / total_accounts as f64 * 100.0
        } else {
            0.0
        };

        let system_prompt = "You are a financial controller. Analyze the reconciliation report and highlight critical accounts needing attention. Be concise.";
        let user_prompt = format!(
            "Reconciliation Report:\n- Total accounts: {}\n- Reconciled: {} ({:.0}%)\n- Unreconciled: {}\n- Total discrepancies: {:.2}\n\nAccount Details:\n{}\n\nProvide summary and priority actions.",
            total_accounts, reconciled_count, reconciliation_rate, unreconciled_count, total_discrepancies,
            serde_json::to_string_pretty(&account_reports).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_accounts": total_accounts,
                "reconciled": reconciled_count,
                "unreconciled": unreconciled_count,
                "reconciliation_rate": format!("{:.1}%", reconciliation_rate),
                "total_discrepancies": total_discrepancies,
            },
            "accounts": account_reports,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for ReconcileReportAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("All accounts reconciled."))
    }

    #[tokio::test]
    async fn test_all_reconciled() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "accounts": "account,gl_balance,subledger_balance,bank_balance,status\nCash,10000,10000,10000,reconciled\nAR,5000,5000,5000,reconciled"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["reconciled"], 2);
        assert_eq!(result["summary"]["unreconciled"], 0);
    }

    #[tokio::test]
    async fn test_discrepancies() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "accounts": "account,gl_balance,subledger_balance,bank_balance,status\nCash,10000,10000,10000,reconciled\nAR,5000,4800,0,pending"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["reconciled"], 1);
        assert_eq!(result["summary"]["unreconciled"], 1);
        assert!(result["summary"]["total_discrepancies"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_missing_accounts() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({});
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_empty_accounts() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "accounts": "" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_llm_analysis() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "accounts": "account,gl_balance,subledger_balance,bank_balance,status\nA,100,100,100,reconciled"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_reconciliation_rate() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "accounts": "account,gl_balance,subledger_balance,bank_balance,status\nA,100,100,100,reconciled\nB,200,200,200,reconciled\nC,300,250,0,pending"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["reconciled"], 2);
        assert_eq!(result["summary"]["unreconciled"], 1);
    }

    #[tokio::test]
    async fn test_account_details() {
        let agent = ReconcileReportAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "accounts": "account,gl_balance,subledger_balance,bank_balance,status\nCash,1000,1000,1000,reconciled"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        let accts = result["accounts"].as_array().unwrap();
        assert_eq!(accts.len(), 1);
        assert_eq!(accts[0]["account"], "Cash");
    }

    #[test]
    fn test_manifest() {
        let agent = ReconcileReportAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.reconcile-report");
        assert!(m.permissions.network_llm);
    }
}
