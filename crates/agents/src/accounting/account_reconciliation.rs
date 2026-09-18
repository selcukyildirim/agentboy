use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct AccountReconciliationAgent;

impl AccountReconciliationAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for AccountReconciliationAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "accounting.account-reconciliation".to_string(),
            version: "2.0.0".to_string(),
            name: "Account Reconciliation".to_string(),
            department: "Accounting".to_string(),
            description: "Reconcile GL accounts with sub-ledger and bank statements, LLM-powered discrepancy analysis".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.compare".to_string(), "llm.analysis".to_string()],
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
                InputField::new("gl_entries", "General Ledger Entries", InputKind::File, true),
                InputField::new("subledger", "Sub-ledger", InputKind::File, true),
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
        let gl_csv = input["gl_entries"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'gl_entries' CSV".to_string()))?;
        let sl_csv = input["subledger"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'subledger' CSV".to_string()))?;

        let gl_entries = csv_util::parse_csv_to_maps(gl_csv)?;
        let sl_entries = csv_util::parse_csv_to_maps(sl_csv)?;

        if csv_util::all_empty(&gl_entries) && csv_util::all_empty(&sl_entries) {
            return Ok(csv_util::empty_response(
                "account.reconciliation",
                &["gl_entries", "subledger"],
            ));
        }

        let gl_total: f64 = gl_entries
            .iter()
            .map(|r| csv_util::record_get_f64(r, "amount"))
            .sum();
        let sl_total: f64 = sl_entries
            .iter()
            .map(|r| csv_util::record_get_f64(r, "amount"))
            .sum();
        let difference = gl_total - sl_total;

        let mut matched = Vec::new();
        let mut unmatched_gl = Vec::new();
        let mut unmatched_sl = Vec::new();
        let mut sl_used = vec![false; sl_entries.len()];

        for gl in &gl_entries {
            let gl_ref = csv_util::record_get_str(gl, "reference");
            let gl_amt = csv_util::record_get_f64(gl, "amount");
            let mut found = false;
            for (i, sl) in sl_entries.iter().enumerate() {
                if sl_used[i] {
                    continue;
                }
                let sl_ref = csv_util::record_get_str(sl, "reference");
                let sl_amt = csv_util::record_get_f64(sl, "amount");
                if gl_ref == sl_ref && (gl_amt - sl_amt).abs() < 0.01 {
                    matched.push(serde_json::json!({
                        "gl": csv_util::csv_record_to_json(gl),
                        "subledger": csv_util::csv_record_to_json(sl),
                        "match_type": "exact"
                    }));
                    sl_used[i] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                unmatched_gl.push(csv_util::csv_record_to_json(gl));
            }
        }
        for (i, sl) in sl_entries.iter().enumerate() {
            if !sl_used[i] {
                unmatched_sl.push(csv_util::csv_record_to_json(sl));
            }
        }

        let system_prompt = "You are an accounting reconciliation expert. Analyze the GL vs subledger reconciliation results and explain potential causes of discrepancies. Be concise.";
        let user_prompt = format!(
            "GL Total: {:.2}, Subledger Total: {:.2}, Difference: {:.2}\nMatched: {}, Unmatched GL: {}, Unmatched SL: {}\n\nUnmatched GL: {}\nUnmatched SL: {}\n\nProvide reconciliation analysis.",
            gl_total, sl_total, difference, matched.len(), unmatched_gl.len(), unmatched_sl.len(),
            serde_json::to_string_pretty(&unmatched_gl).unwrap_or_default(),
            serde_json::to_string_pretty(&unmatched_sl).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "status": if difference.abs() < 0.01 { "reconciled" } else { "discrepancies_found" },
            "gl_total": gl_total,
            "subledger_total": sl_total,
            "difference": difference,
            "matched_count": matched.len(),
            "unmatched_gl_count": unmatched_gl.len(),
            "unmatched_subledger_count": unmatched_sl.len(),
            "matched": matched,
            "unmatched_gl": unmatched_gl,
            "unmatched_subledger": unmatched_sl,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for AccountReconciliationAgent {
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
            "Reconciliation complete. Minor difference in one entry.",
        ))
    }

    #[tokio::test]
    async fn test_reconciled() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "gl_entries": "reference,amount,description\nREF001,1000,Payment\nREF002,500,Invoice",
            "subledger": "reference,amount,description\nREF001,1000,Payment\nREF002,500,Invoice"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "reconciled");
        assert_eq!(result["matched_count"], 2);
    }

    #[tokio::test]
    async fn test_discrepancy() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "gl_entries": "reference,amount,description\nREF001,1000,P\nREF002,500,I",
            "subledger": "reference,amount,description\nREF001,1000,P\nREF002,600,I"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["status"], "discrepancies_found");
        assert!(result["difference"].as_f64().unwrap().abs() > 0.0);
    }

    #[tokio::test]
    async fn test_missing_gl() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "subledger": "reference,amount\nR1,100" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_missing_subledger() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({ "gl_entries": "reference,amount\nR1,100" });
        assert!(agent.execute_with_context(input, &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_partial_match() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "gl_entries": "reference,amount,desc\nREF001,100,A\nREF002,200,B\nREF003,300,C",
            "subledger": "reference,amount,desc\nREF001,100,A\nREF004,400,D"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["matched_count"], 1);
        assert_eq!(result["unmatched_gl_count"], 2);
        assert_eq!(result["unmatched_subledger_count"], 1);
    }

    #[tokio::test]
    async fn test_llm_analysis_present() {
        let agent = AccountReconciliationAgent::new();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "gl_entries": "reference,amount,desc\nR1,100,A",
            "subledger": "reference,amount,desc\nR1,100,A"
        });
        let result = agent.execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let agent = AccountReconciliationAgent::new();
        let m = agent.manifest();
        assert_eq!(m.id, "accounting.account-reconciliation");
        assert!(m.permissions.network_llm);
    }
}
