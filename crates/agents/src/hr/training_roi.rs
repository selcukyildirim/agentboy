use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

use crate::csv_util;

pub struct TrainingROIAgent;

impl TrainingROIAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for TrainingROIAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "hr.training-roi".to_string(),
            version: "2.0.0".to_string(),
            name: "Training ROI".to_string(),
            department: "HR".to_string(),
            description: "Measure training program effectiveness and return on investment"
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
                "programs",
                "Training Programs",
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
        let programs_csv = input["programs"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'programs' CSV".to_string()))?;

        let programs = csv_util::parse_csv_to_maps(programs_csv)?;
        if programs.is_empty() {
            return Err(AppError::Validation(
                "No training programs found".to_string(),
            ));
        }

        let mut program_analysis: Vec<serde_json::Value> = programs
            .iter()
            .map(|p| {
                let cost = csv_util::record_get_f64(p, "cost");
                let participants = csv_util::record_get_f64(p, "participants");
                let completion_rate = csv_util::record_get_f64(p, "completion_rate");
                let productivity_gain = csv_util::record_get_f64(p, "productivity_gain_pct");
                let satisfaction = csv_util::record_get_f64(p, "satisfaction_score");

                let cost_per_participant = if participants > 0.0 {
                    cost / participants
                } else {
                    0.0
                };
                let roi = if cost > 0.0 {
                    (productivity_gain * cost / 100.0) / cost
                } else {
                    0.0
                };

                let effectiveness = if productivity_gain > 20.0 && completion_rate > 0.8 {
                    "high"
                } else if productivity_gain > 10.0 {
                    "medium"
                } else {
                    "low"
                };

                serde_json::json!({
                    "program": csv_util::record_get_str(p, "program"),
                    "cost": cost,
                    "participants": participants,
                    "completion_rate": completion_rate,
                    "productivity_gain_pct": productivity_gain,
                    "satisfaction_score": satisfaction,
                    "cost_per_participant": cost_per_participant,
                    "roi": format!("{:.1}", roi),
                    "effectiveness": effectiveness,
                })
            })
            .collect();

        let total_cost: f64 = program_analysis
            .iter()
            .map(|p| p["cost"].as_f64().unwrap_or(0.0))
            .sum();
        let total_participants: u64 = program_analysis
            .iter()
            .map(|p| p["participants"].as_u64().unwrap_or(0))
            .sum();
        let high_effectiveness = program_analysis
            .iter()
            .filter(|p| p["effectiveness"] == "high")
            .count();

        program_analysis.sort_by(|a, b| {
            b["productivity_gain_pct"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["productivity_gain_pct"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let system_prompt = "You are a training ROI analyst. Analyze program effectiveness and recommend training investments. Be concise.";
        let user_prompt = format!(
            "Training ROI ({} programs):\n- Total cost: {:.0}\n- Total participants: {}\n- High effectiveness: {}\n\nPrograms:\n{}\n\nProvide training strategy recommendations.",
            programs.len(), total_cost, total_participants, high_effectiveness,
            serde_json::to_string_pretty(&program_analysis).unwrap_or_default(),
        );
        let llm_analysis = ctx.call_llm(system_prompt, &user_prompt).await?;

        Ok(serde_json::json!({
            "summary": { "total_programs": programs.len(), "total_cost": total_cost, "total_participants": total_participants, "high_effectiveness": high_effectiveness },
            "programs": program_analysis,
            "llm_analysis": llm_analysis,
        }))
    }
}

impl Default for TrainingROIAgent {
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
            "Leadership program has highest ROI. Expand it.",
        ))
    }

    #[tokio::test]
    async fn test_training_roi() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "programs": "program,cost,participants,completion_rate,productivity_gain_pct,satisfaction_score\nLeadership,50000,20,0.9,25,4.5\nSafety,10000,100,0.95,5,4.0"
        });
        let result = TrainingROIAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert_eq!(result["summary"]["total_programs"], 2);
        assert!(result["summary"]["total_cost"].as_f64().unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_effectiveness() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "programs": "program,cost,participants,completion_rate,productivity_gain_pct,satisfaction_score\nHigh,10000,10,0.9,25,4.5\nLow,10000,10,0.5,5,3.0"
        });
        let result = TrainingROIAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        let progs = result["programs"].as_array().unwrap();
        assert_eq!(progs[0]["effectiveness"], "high");
        assert_eq!(progs[1]["effectiveness"], "low");
    }

    #[tokio::test]
    async fn test_empty() {
        let ctx = make_ctx();
        assert!(TrainingROIAgent::new()
            .execute_with_context(serde_json::json!({ "programs": "" }), &ctx)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_llm() {
        let ctx = make_ctx();
        let input = serde_json::json!({
            "programs": "program,cost,participants,completion_rate,productivity_gain_pct,satisfaction_score\nA,1000,5,0.8,15,4.0"
        });
        let result = TrainingROIAgent::new()
            .execute_with_context(input, &ctx)
            .await
            .unwrap();
        assert!(result["llm_analysis"].as_str().is_some());
    }

    #[test]
    fn test_manifest() {
        let m = TrainingROIAgent::new().manifest();
        assert_eq!(m.id, "hr.training-roi");
    }
}
