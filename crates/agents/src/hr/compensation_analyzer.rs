use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct CompensationAnalyzerAgent;

impl CompensationAnalyzerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for CompensationAnalyzerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.compensation-analyzer".to_string(),
            version: "2.0.0".to_string(),
            name: "Compensation Analyzer".to_string(),
            department: "HR".to_string(),
            description: "Analyze compensation equity, identify pay gaps, and benchmark against market rates".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("employees", "Employees", InputKind::File, true).with_example("employee_id,name,department,salary,performance_score\nE1,Alice,Engineering,100000,4.2"),
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
        let employees_csv = input["employees"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'employees' CSV".to_string()))?;

        let employees = csv_util::parse_csv_to_maps(employees_csv)?;
        if employees.is_empty() {
            return Err(AppError::Validation("No employees found".to_string()));
        }

        let by_role = csv_util::sum_by(&employees, "role", "salary");
        let role_count = csv_util::group_by(&employees, "role");

        let mut equity_gaps = Vec::new();
        let mut avg_ratios = Vec::new();

        for (role, total_salary) in &by_role {
            let count = role_count.get(role).map(|v| v.len()).unwrap_or(1) as f64;
            let avg_salary = total_salary / count;

            if let Some(role_employees) = role_count.get(role) {
                let market_rate = role_employees
                    .iter()
                    .filter_map(|e| e.get("market_rate").and_then(|v| v.parse::<f64>().ok()))
                    .next()
                    .unwrap_or(avg_salary);

                let market_ratio = avg_salary / market_rate;
                avg_ratios.push(market_ratio);

                let min_sal: f64 = role_employees
                    .iter()
                    .filter_map(|e| e.get("salary").and_then(|v| v.parse::<f64>().ok()))
                    .fold(f64::INFINITY, f64::min);
                let max_sal: f64 = role_employees
                    .iter()
                    .filter_map(|e| e.get("salary").and_then(|v| v.parse::<f64>().ok()))
                    .fold(f64::NEG_INFINITY, f64::max);

                if min_sal != max_sal && min_sal > 0.0 {
                    let pay_gap = (max_sal - min_sal) / min_sal * 100.0;
                    equity_gaps.push(serde_json::json!({
                        "role": role,
                        "avg_salary": avg_salary,
                        "min_salary": min_sal,
                        "max_salary": max_sal,
                        "pay_spread_pct": format!("{:.1}%", pay_gap),
                        "market_ratio": format!("{:.2}", market_ratio),
                        "employee_count": role_employees.len(),
                    }));
                }
            }
        }

        equity_gaps.sort_by(|a, b| {
            b["pay_spread_pct"]
                .as_str()
                .unwrap_or("0%")
                .partial_cmp(a["pay_spread_pct"].as_str().unwrap_or("0%"))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let total_salary: f64 = employees
            .iter()
            .map(|e| csv_util::record_get_f64(e, "salary"))
            .sum();
        let avg_market_ratio = if !avg_ratios.is_empty() {
            avg_ratios.iter().sum::<f64>() / avg_ratios.len() as f64
        } else {
            1.0
        };

        let system_prompt =
            "You are a compensation equity analyst. Analyze pay gaps and benchmarking. Be concise.";
        let user_prompt = format!(
            "Compensation Analysis ({} employees, total salary: {:.0}):\n- Avg market ratio: {:.2}\n- Roles with equity gaps: {}\n\nGaps:\n{}\n\nProvide compensation recommendations.",
            employees.len(), total_salary, avg_market_ratio, equity_gaps.len(),
            serde_json::to_string_pretty(&equity_gaps).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_employees": employees.len(), "total_salary": total_salary, "avg_market_ratio": format!("{:.2}", avg_market_ratio), "roles_with_gaps": equity_gaps.len() },
            "equity_gaps": equity_gaps,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for CompensationAnalyzerAgent {
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
            "Significant pay gap in Engineering. Recommend review.",
        ))
    }

    #[tokio::test]
    async fn test_compensation() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,role,salary,market_rate\nAlice,Eng,80000,90000\nBob,Eng,120000,90000\nCarol,Sales,70000,75000"
        });
        let result = CompensationAnalyzerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_employees"], 3);
        assert!(result["summary"]["roles_with_gaps"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_no_gap() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,role,salary,market_rate\nA,Eng,100000,100000\nB,Eng,100000,100000"
        });
        let result = CompensationAnalyzerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["roles_with_gaps"], 0);
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(CompensationAnalyzerAgent::new()
            .execute_with_context(serde_json::json!({ "employees": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,role,salary,market_rate\nA,Eng,100000,100000"
        });
        let result = CompensationAnalyzerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = CompensationAnalyzerAgent::new().manifest();
        assert_eq!(m.id, "hr.compensation-analyzer");
    }
}
