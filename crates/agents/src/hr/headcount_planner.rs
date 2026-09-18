use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct HeadcountPlannerAgent;

impl HeadcountPlannerAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for HeadcountPlannerAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.headcount-planner".to_string(),
            version: "2.0.0".to_string(),
            name: "Headcount Planner".to_string(),
            department: "HR".to_string(),
            description: "Plan headcount needs by department, analyze attrition gaps, and forecast hiring demand".to_string(),
            tier: AgentTier::Free,
            skills: vec!["spreadsheet.parse".to_string(), "spreadsheet.analyze".to_string(), "llm.analysis".to_string()],
            permissions: AgentPermissions { filesystem_read: true, filesystem_write: false, network_llm: true },
            execution: ExecutionLimits { max_steps: 30, timeout_seconds: 120 },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![
                InputField::new("departments", "Departments", InputKind::File, true),
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
        let departments_csv = input["departments"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'departments' CSV".to_string()))?;

        let depts = csv_util::parse_csv_to_maps(departments_csv)?;
        if depts.is_empty() {
            return Err(AppError::Validation(
                "No department records found".to_string(),
            ));
        }

        let mut dept_plans: Vec<serde_json::Value> = depts
            .iter()
            .map(|d| {
                let current = csv_util::record_get_f64(d, "current_headcount") as u64;
                let target = csv_util::record_get_f64(d, "target_headcount") as u64;
                let expected_attrition = csv_util::record_get_f64(d, "expected_attrition") as u64;
                let open_positions = csv_util::record_get_f64(d, "open_positions") as u64;

                let gap = if target > current {
                    target - current
                } else {
                    0
                };
                let net_hiring = gap + expected_attrition;
                let urgency = if net_hiring > open_positions {
                    "critical"
                } else if net_hiring > 0 {
                    "normal"
                } else {
                    "none"
                };

                serde_json::json!({
                    "department": csv_util::record_get_str(d, "department"),
                    "current_headcount": current,
                    "target_headcount": target,
                    "expected_attrition": expected_attrition,
                    "open_positions": open_positions,
                    "gap": gap,
                    "net_hiring_needed": net_hiring,
                    "urgency": urgency,
                })
            })
            .collect();

        let total_current: u64 = dept_plans
            .iter()
            .map(|p| p["current_headcount"].as_u64().unwrap_or(0))
            .sum();
        let total_hiring: u64 = dept_plans
            .iter()
            .map(|p| p["net_hiring_needed"].as_u64().unwrap_or(0))
            .sum();
        let critical: usize = dept_plans
            .iter()
            .filter(|p| p["urgency"] == "critical")
            .count();

        dept_plans.sort_by(|a, b| {
            b["net_hiring_needed"]
                .as_u64()
                .unwrap_or(0)
                .partial_cmp(&a["net_hiring_needed"].as_u64().unwrap_or(0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let system_prompt = "You are an HR workforce planner. Analyze headcount gaps and recommend hiring strategy. Be concise.";
        let user_prompt = format!(
            "Headcount Planning ({} departments):\n- Total current: {}\n- Total net hiring needed: {}\n- Critical departments: {}\n\nDepartment Plans:\n{}\n\nProvide hiring prioritization.",
            depts.len(), total_current, total_hiring, critical,
            serde_json::to_string_pretty(&dept_plans).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_departments": depts.len(), "total_current": total_current, "total_hiring_needed": total_hiring, "critical_departments": critical },
            "departments": dept_plans,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for HeadcountPlannerAgent {
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
            "Engineering has critical hiring gap. Prioritize.",
        ))
    }

    #[tokio::test]
    async fn test_planning() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "departments": "department,current_headcount,target_headcount,expected_attrition,open_positions\nEngineering,20,25,2,3\nSales,10,10,1,2"
        });
        let result = HeadcountPlannerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_departments"], 2);
        assert!(result["summary"]["total_hiring_needed"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_urgency() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "departments": "department,current_headcount,target_headcount,expected_attrition,open_positions\nEng,10,20,5,2"
        });
        let result = HeadcountPlannerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(
            result["departments"].as_array().unwrap()[0]["urgency"],
            "critical"
        );
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(HeadcountPlannerAgent::new()
            .execute_with_context(serde_json::json!({ "departments": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_no_hiring_needed() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "departments": "department,current_headcount,target_headcount,expected_attrition,open_positions\nA,10,8,0,0"
        });
        let result = HeadcountPlannerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(
            result["departments"].as_array().unwrap()[0]["urgency"],
            "none"
        );
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "departments": "department,current_headcount,target_headcount,expected_attrition,open_positions\nA,5,10,1,2"
        });
        let result = HeadcountPlannerAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = HeadcountPlannerAgent::new().manifest();
        assert_eq!(m.id, "hr.headcount-planner");
    }
}
