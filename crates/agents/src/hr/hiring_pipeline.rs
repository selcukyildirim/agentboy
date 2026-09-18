use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct HiringPipelineAgent;

impl HiringPipelineAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for HiringPipelineAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.hiring-pipeline".to_string(),
            version: "2.0.0".to_string(),
            name: "Hiring Pipeline".to_string(),
            department: "HR".to_string(),
            description:
                "Analyze hiring pipeline efficiency, time-to-hire, and conversion rates by stage"
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
                max_steps: 30,
                timeout_seconds: 120,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![InputField::new(
                "candidates",
                "Candidates",
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
        let candidates_csv = input["candidates"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'candidates' CSV".to_string()))?;

        let candidates = csv_util::parse_csv_to_maps(candidates_csv)?;
        if candidates.is_empty() {
            return Err(AppError::Validation("No candidates found".to_string()));
        }

        let stages = vec![
            "applied",
            "screening",
            "interview",
            "offer",
            "hired",
            "rejected",
        ];
        let mut stage_counts = std::collections::HashMap::new();

        for c in &candidates {
            let stage = csv_util::record_get_str(c, "stage");
            *stage_counts.entry(stage.to_string()).or_insert(0u64) += 1;
        }

        let applied = stage_counts.get("applied").copied().unwrap_or(0);
        let screening = stage_counts.get("screening").copied().unwrap_or(0);
        let interview = stage_counts.get("interview").copied().unwrap_or(0);
        let offer = stage_counts.get("offer").copied().unwrap_or(0);
        let hired = stage_counts.get("hired").copied().unwrap_or(0);
        let rejected = stage_counts.get("rejected").copied().unwrap_or(0);

        let conversion = |from: u64, to: u64| {
            if from > 0 {
                to as f64 / from as f64 * 100.0
            } else {
                0.0
            }
        };

        let time_to_hires: Vec<f64> = candidates
            .iter()
            .filter(|c| csv_util::record_get_str(c, "stage") == "hired")
            .filter_map(|c| c.get("days_to_hire").and_then(|v| v.parse::<f64>().ok()))
            .collect();
        let avg_time_to_hire = if !time_to_hires.is_empty() {
            time_to_hires.iter().sum::<f64>() / time_to_hires.len() as f64
        } else {
            0.0
        };

        let by_position = csv_util::group_by(&candidates, "position");
        let position_summary: Vec<serde_json::Value> = by_position.iter().map(|(pos, cand)| {
            let pos_applied = cand.iter().filter(|c| csv_util::record_get_str(c, "stage") == "applied").count();
            let pos_hired = cand.iter().filter(|c| csv_util::record_get_str(c, "stage") == "hired").count();
            serde_json::json!({
                "position": pos,
                "applied": pos_applied,
                "hired": pos_hired,
                "fill_rate": if pos_applied > 0 { pos_hired as f64 / pos_applied as f64 * 100.0 } else { 0.0 },
            })
        }).collect();

        let stage_summary: Vec<serde_json::Value> = stages
            .iter()
            .filter_map(|s| {
                let count = stage_counts.get(*s).copied().unwrap_or(0);
                if count == 0 {
                    return None;
                }
                Some(serde_json::json!({ "stage": s, "count": count }))
            })
            .collect();

        let system_prompt = "You are a hiring pipeline analyst. Analyze recruitment efficiency and recommend improvements. Be concise.";
        let user_prompt = format!(
            "Hiring Pipeline ({} candidates):\n- Applied: {}, Screened: {}, Interviewed: {}, Offered: {}, Hired: {}, Rejected: {}\n- Conversion: Applied→Interview {:.1}%, Interview→Hired {:.1}%\n- Avg time to hire: {:.0} days\n\nBy Position:\n{}\n\nProvide hiring optimization recommendations.",
            candidates.len(), applied, screening, interview, offer, hired, rejected,
            conversion(applied, interview), conversion(interview, hired), avg_time_to_hire,
            serde_json::to_string_pretty(&position_summary).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": {
                "total_candidates": candidates.len(),
                "applied": applied, "screening": screening, "interview": interview,
                "offer": offer, "hired": hired, "rejected": rejected,
                "avg_time_to_hire": avg_time_to_hire,
            },
            "stages": stage_summary,
            "by_position": position_summary,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for HiringPipelineAgent {
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
            "Interview-to-offer conversion is low. Improve screening.",
        ))
    }

    #[tokio::test]
    async fn test_pipeline() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "candidates": "name,position,stage,days_to_hire\nA,Eng,applied,\nB,Eng,interview,\nC,Eng,hired,30\nD,Sales,hired,25\nE,Sales,rejected,"
        });
        let result = HiringPipelineAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_candidates"], 5);
        assert_eq!(result["summary"]["hired"], 2);
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(HiringPipelineAgent::new()
            .execute_with_context(serde_json::json!({ "candidates": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_by_position() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "candidates": "name,position,stage,days_to_hire\nA,Eng,applied,\nB,Eng,hired,30\nC,Sales,applied,"
        });
        let result = HiringPipelineAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        let positions = result["by_position"].as_array().unwrap();
        assert_eq!(positions.len(), 2);
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "candidates": "name,position,stage,days_to_hire\nA,Eng,hired,20"
        });
        let result = HiringPipelineAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = HiringPipelineAgent::new().manifest();
        assert_eq!(m.id, "hr.hiring-pipeline");
    }
}
