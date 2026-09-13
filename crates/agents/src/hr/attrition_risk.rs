use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{AgentManifest, AgentPermissions, AgentTier, ExecutionLimits};

use crate::csv_util;

pub struct AttritionRiskAgent;

impl AttritionRiskAgent {
    pub fn new() -> Self { Self }
}

#[async_trait::async_trait]
impl Agent for AttritionRiskAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.attrition-risk".to_string(),
            version: "2.0.0".to_string(),
            name: "Attrition Risk".to_string(),
            department: "HR".to_string(),
            description: "Predict employee attrition risk based on satisfaction, tenure, performance, and compensation signals".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
        }
    }

    fn supports_context(&self) -> bool { true }

    async fn execute_with_context(&self, input: serde_json::Value, ctx: &dyn AgentContext) -> AppResult<serde_json::Value> {
        let employees_csv = input["employees"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'employees' CSV".to_string()))?;

        let employees = csv_util::parse_csv_to_maps(employees_csv)?;
        if employees.is_empty() {
            return Err(AppError::Validation("No employees found".to_string()));
        }

        let mut scored: Vec<serde_json::Value> = employees.iter().map(|e| {
            let satisfaction = csv_util::record_get_f64(e, "satisfaction_score");
            let tenure = csv_util::record_get_f64(e, "tenure_years");
            let performance = csv_util::record_get_f64(e, "performance_score");
            let salary_ratio = csv_util::record_get_f64(e, "salary_ratio");

            let risk = calculate_attrition_risk(satisfaction, tenure, performance, salary_ratio);
            let tier = if risk > 70 { "high" } else if risk > 40 { "medium" } else { "low" };

            serde_json::json!({
                "name": csv_util::record_get_str(e, "name"),
                "department": csv_util::record_get_str(e, "department"),
                "satisfaction_score": satisfaction,
                "tenure_years": tenure,
                "performance_score": performance,
                "salary_ratio": salary_ratio,
                "risk_score": risk,
                "risk_tier": tier,
            })
        }).collect();

        scored.sort_by(|a, b| b["risk_score"].as_u64().unwrap_or(0).partial_cmp(&a["risk_score"].as_u64().unwrap_or(0)).unwrap_or(std::cmp::Ordering::Equal));

        let high = scored.iter().filter(|s| s["risk_tier"] == "high").count();
        let medium = scored.iter().filter(|s| s["risk_tier"] == "medium").count();
        let low = scored.iter().filter(|s| s["risk_tier"] == "low").count();

        let system_prompt = "You are an HR attrition analyst. Analyze employee flight risk and recommend retention actions. Be concise.";
        let user_prompt = format!(
            "Attrition Risk ({} employees):\n- High: {}, Medium: {}, Low: {}\n\nTop risk employees:\n{}\n\nProvide retention recommendations.",
            employees.len(), high, medium, low,
            serde_json::to_string_pretty(&scored[..scored.len().min(5)]).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total": employees.len(), "high_risk": high, "medium_risk": medium, "low_risk": low },
            "employees": scored,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for AttritionRiskAgent { fn default() -> Self { Self::new() } }

fn calculate_attrition_risk(satisfaction: f64, tenure: f64, performance: f64, salary_ratio: f64) -> u32 {
    let mut risk: u32 = 0;
    if satisfaction < 0.3 { risk += 35; } else if satisfaction < 0.5 { risk += 20; }
    if tenure < 1.0 { risk += 20; } else if tenure < 3.0 { risk += 10; }
    if performance > 0.8 { risk += 15; } else if performance < 0.3 { risk += 20; }
    if salary_ratio < 0.8 { risk += 20; } else if salary_ratio < 1.0 { risk += 5; }
    risk.min(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};

    fn make_ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Alice is high risk. Recommend retention discussion."))
    }

    #[tokio::test]
    async fn test_attrition() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,department,satisfaction_score,tenure_years,performance_score,salary_ratio\nAlice,Eng,0.2,0.5,0.9,0.7\nBob,Sales,0.8,5,0.7,1.1"
        });
        let result = AttritionRiskAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert_eq!(result["summary"]["total"], 2);
        assert!(result["summary"]["high_risk"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_risk_score_calc() {
        assert!(calculate_attrition_risk(0.1, 0.5, 0.9, 0.7) > 60);
        assert!(calculate_attrition_risk(0.9, 5.0, 0.7, 1.1) < 20);
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(AttritionRiskAgent::new().execute_with_context(serde_json::json!({ "employees": "" }), &ctx).await.is_err());
    }

    #[tokio::test]
    async fn test_sorted_by_risk() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,department,satisfaction_score,tenure_years,performance_score,salary_ratio\nLow,D,0.9,10,0.5,1.2\nHigh,D,0.1,0.5,0.9,0.5"
        });
        let result = AttritionRiskAgent::new().execute_with_context(input, &ctx).await.unwrap();
        let emps = result["employees"].as_array().unwrap();
        assert!(emps[0]["risk_score"].as_u64().unwrap() >= emps[1]["risk_score"].as_u64().unwrap());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "employees": "name,department,satisfaction_score,tenure_years,performance_score,salary_ratio\nA,D,0.5,2,0.5,1.0"
        });
        let result = AttritionRiskAgent::new().execute_with_context(input, &ctx).await.unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = AttritionRiskAgent::new().manifest();
        assert_eq!(m.id, "hr.attrition-risk");
        assert!(m.permissions.network_llm);
    }
}