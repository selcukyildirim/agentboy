use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct DecisionMatrixAgent;
impl DecisionMatrixAgent {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for DecisionMatrixAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "management.decision-matrix".to_string(),
            version: "2.0.0".to_string(),
            name: "Decision Matrix".to_string(),
            department: "Management".to_string(),
            description: "Weighted multi-criteria decision analysis for strategic choices"
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
            input_schema: vec![
                InputField::new("options", "Options (JSON)", InputKind::Json, true).with_example(
                    "[{\"name\":\"Option A\",\"score\":8},{\"name\":\"Option B\",\"score\":6}]",
                ),
                InputField::new("weights", "Weights (JSON)", InputKind::Json, false).with_example(
                    "{\"price\":0.4,\"quality\":0.3,\"delivery\":0.2,\"reliability\":0.1}",
                ),
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
        let options = input["options"]
            .as_array()
            .ok_or_else(|| AppError::Validation("Missing 'options' array".to_string()))?;
        let weights = input["weights"].as_object().cloned().unwrap_or_default();

        let mut scored: Vec<serde_json::Value> = options
            .iter()
            .map(|opt| {
                let name = opt["name"].as_str().unwrap_or("Unknown");
                let mut total = 0.0;
                let mut details = serde_json::Map::new();
                for (criterion, weight) in &weights {
                    let w = weight.as_f64().unwrap_or(0.0);
                    let score = opt
                        .get(criterion)
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.5);
                    total += score * w;
                    details.insert(
                        criterion.clone(),
                        serde_json::json!({ "score": score, "weight": w, "weighted": score * w }),
                    );
                }
                serde_json::json!({ "name": name, "total_score": total, "details": details })
            })
            .collect();
        scored.sort_by(|a, b| {
            b["total_score"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&a["total_score"].as_f64().unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let winner = scored.first().map(|s| s["name"].as_str().unwrap_or(""));

        let llm = ctx
            .call_llm(
                "Analyze decision matrix.",
                &format!(
                    "Decision Matrix ({} options):\n{}\n\nRecommend?",
                    options.len(),
                    serde_json::to_string_pretty(&scored).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_options": options.len(), "recommended": winner }, "options": scored, "llm_analysis": llm }),
        )
    }
}
impl Default for DecisionMatrixAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Option A is best overall."))
    }

    #[tokio::test]
    async fn test_decision() {
        let input = serde_json::json!({ "options": [{"name":"A","cost":0.9,"speed":0.7},{"name":"B","cost":0.6,"speed":0.9}], "weights": {"cost":0.5,"speed":0.5} });
        let r = DecisionMatrixAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["total_options"], 2);
        assert_eq!(r["summary"]["recommended"], "A");
    }
    #[tokio::test]
    async fn test_llm() {
        let input = serde_json::json!({ "options": [{"name":"X","a":0.5}], "weights": {"a":1.0} });
        let r = DecisionMatrixAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert!(r["llm_analysis"].as_str().is_some());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            DecisionMatrixAgent::new().manifest().id,
            "management.decision-matrix"
        );
    }
}
