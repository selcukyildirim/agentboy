use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct EmployeeEngagementAgent;

impl EmployeeEngagementAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for EmployeeEngagementAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.employee-engagement".to_string(),
            version: "2.0.0".to_string(),
            name: "Employee Engagement".to_string(),
            department: "HR".to_string(),
            description: "Analyze employee engagement survey data, identify trends, and recommend improvements".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("surveys", "Engagement Surveys", InputKind::File, true),
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
        let surveys_csv = input["surveys"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'surveys' CSV".to_string()))?;

        let surveys = csv_util::parse_csv_to_maps(surveys_csv)?;
        if surveys.is_empty() {
            return Err(AppError::Validation(
                "No survey responses found".to_string(),
            ));
        }

        let by_dept = csv_util::group_by(&surveys, "department");

        let mut dept_scores: Vec<serde_json::Value> = by_dept
            .iter()
            .map(|(dept, responses)| {
                let overall: Vec<f64> = responses
                    .iter()
                    .filter_map(|r| r.get("overall_score").and_then(|v| v.parse::<f64>().ok()))
                    .collect();
                let worklife: Vec<f64> = responses
                    .iter()
                    .filter_map(|r| {
                        r.get("work_life_balance")
                            .and_then(|v| v.parse::<f64>().ok())
                    })
                    .collect();
                let growth: Vec<f64> = responses
                    .iter()
                    .filter_map(|r| {
                        r.get("growth_opportunity")
                            .and_then(|v| v.parse::<f64>().ok())
                    })
                    .collect();
                let management: Vec<f64> = responses
                    .iter()
                    .filter_map(|r| {
                        r.get("management_score")
                            .and_then(|v| v.parse::<f64>().ok())
                    })
                    .collect();

                let avg = |vals: &[f64]| {
                    if vals.is_empty() {
                        0.0
                    } else {
                        vals.iter().sum::<f64>() / vals.len() as f64
                    }
                };

                let overall_avg = avg(&overall);
                let engagement_level = if overall_avg >= 4.0 {
                    "high"
                } else if overall_avg >= 3.0 {
                    "moderate"
                } else {
                    "low"
                };

                serde_json::json!({
                    "department": dept,
                    "response_count": responses.len(),
                    "overall_score": overall_avg,
                    "work_life_balance": avg(&worklife),
                    "growth_opportunity": avg(&growth),
                    "management_score": avg(&management),
                    "engagement_level": engagement_level,
                })
            })
            .collect();

        dept_scores.sort_by(|a, b| {
            a["overall_score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&b["overall_score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let all_overall: Vec<f64> = surveys
            .iter()
            .filter_map(|r| r.get("overall_score").and_then(|v| v.parse::<f64>().ok()))
            .collect();
        let company_avg = if all_overall.is_empty() {
            0.0
        } else {
            all_overall.iter().sum::<f64>() / all_overall.len() as f64
        };

        let system_prompt = "You are an employee engagement analyst. Analyze survey results and recommend engagement improvements. Be concise.";
        let user_prompt = format!(
            "Employee Engagement ({} responses, {} departments):\n- Company avg: {:.2}\n\nDepartment Scores:\n{}\n\nProvide engagement improvement recommendations.",
            surveys.len(), by_dept.len(), company_avg,
            serde_json::to_string_pretty(&dept_scores).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_responses": surveys.len(), "total_departments": by_dept.len(), "company_avg_score": company_avg },
            "departments": dept_scores,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for EmployeeEngagementAgent {
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
            "Engineering has lowest engagement. Focus on work-life balance.",
        ))
    }

    #[tokio::test]
    async fn test_engagement() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "surveys": "department,overall_score,work_life_balance,growth_opportunity,management_score\nEng,3.0,2.5,3.5,3.0\nEng,4.0,3.5,4.0,4.0\nSales,4.5,4.0,4.0,4.5"
        });
        let result = EmployeeEngagementAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_responses"], 3);
        assert!(result["summary"]["company_avg_score"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_levels() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "surveys": "department,overall_score,work_life_balance,growth_opportunity,management_score\nA,4.5,4,4,4\nB,2.5,2,2,2"
        });
        let result = EmployeeEngagementAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        let depts = result["departments"].as_array().unwrap();
        assert_eq!(depts[0]["engagement_level"], "low");
        assert_eq!(depts[1]["engagement_level"], "high");
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(EmployeeEngagementAgent::new()
            .execute_with_context(serde_json::json!({ "surveys": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "surveys": "department,overall_score,work_life_balance,growth_opportunity,management_score\nA,3.5,3,4,3.5"
        });
        let result = EmployeeEngagementAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = EmployeeEngagementAgent::new().manifest();
        assert_eq!(m.id, "hr.employee-engagement");
    }
}
