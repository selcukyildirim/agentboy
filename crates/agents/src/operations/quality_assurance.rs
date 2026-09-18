use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct QualityAssuranceAgent;
impl QualityAssuranceAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for QualityAssuranceAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "operations.quality-assurance".to_string(),
            version: "2.0.0".to_string(),
            name: "Quality Assurance".to_string(),
            department: "Operations".to_string(),
            description:
                "Track defect rates, analyze quality trends, and recommend process improvements"
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
                InputField::new("defects", "Defects", InputKind::File, true),
                InputField::new(
                    "total_inspected",
                    "Total Inspected",
                    InputKind::Number,
                    true,
                )
                .with_example("1000"),
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
        let csv = input["defects"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'defects' CSV".to_string()))?;
        let defects = csv_util::parse_csv_to_maps(csv)?;

        if csv_util::all_empty(&defects) {
            return Ok(csv_util::empty_response(
                "operations.quality-assurance",
                &["defects"],
            ));
        }

        let total_inspected = input["total_inspected"]
            .as_f64()
            .unwrap_or(defects.len() as f64);
        let total_defects: f64 = defects
            .iter()
            .map(|d| csv_util::record_get_f64(d, "count"))
            .sum();
        let defect_rate = if total_inspected > 0.0 {
            total_defects / total_inspected * 100.0
        } else {
            0.0
        };

        let by_category = csv_util::sum_by(&defects, "category", "count");
        let by_line = csv_util::sum_by(&defects, "production_line", "count");

        let worst_line = by_line
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
        let user_prompt = format!("Quality Report: {:.2}% defect rate ({}/{} inspected).\nBy category: {:?}\nBy line: {:?}\n\nRecommendations?", defect_rate, total_defects, total_inspected, by_category, by_line);
        let llm = ctx
            .call_llm(
                "Analyze quality defects and recommend improvements.",
                &user_prompt,
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_inspected": total_inspected, "total_defects": total_defects, "defect_rate_pct": defect_rate, "worst_line": worst_line.map(|(k,_)| k.as_str()) }, "by_category": by_category, "by_line": by_line, "llm_analysis": llm }),
        )
    }
}
impl Default for QualityAssuranceAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response(
            "Line B has highest defects.",
        ))
    }

    #[tokio::test]
    async fn test_quality() {
        let input = serde_json::json!({ "defects": "category,production_line,count\nScratch,Line A,5\nDent,Line B,8\nScratch,Line B,3", "total_inspected": 1000 });
        let r = QualityAssuranceAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["total_defects"], 16.0);
        assert!(r["summary"]["defect_rate_pct"].as_f64().unwrap() > 0.0);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(QualityAssuranceAgent::new()
            .execute_with_context(serde_json::json!({}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            QualityAssuranceAgent::new().manifest().id,
            "operations.quality-assurance"
        );
    }
}
