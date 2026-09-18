use crate::csv_util;
use agent_common::error::{AppError, AppResult};
use agent_runtime::agent::Agent;
use agent_runtime::context::AgentContext;
use agent_runtime::manifest::{
    AgentManifest, AgentPermissions, AgentTier, ExecutionLimits, InputField, InputKind,
};

pub struct ResourceAllocatorAgent;
impl ResourceAllocatorAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Agent for ResourceAllocatorAgent {
    fn manifest(&self) -> AgentManifest {
        AgentManifest {
            id: "pmo.resource-allocator".to_string(),
            version: "2.0.0".to_string(),
            name: "Resource Allocator".to_string(),
            department: "PMO".to_string(),
            description:
                "Optimize resource allocation across projects, identify overallocation and gaps"
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
                "resources",
                "Resources",
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
        let csv = input["resources"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'resources' CSV".to_string()))?;
        let resources = csv_util::parse_csv_to_maps(csv)?;
        if resources.is_empty() {
            return Err(AppError::Validation("No resources".to_string()));
        }

        let mut analysis: Vec<serde_json::Value> = resources.iter().map(|r| {
            let capacity = csv_util::record_get_f64(r, "capacity_hours");
            let allocated = csv_util::record_get_f64(r, "allocated_hours");
            let util = if capacity > 0.0 { allocated / capacity * 100.0 } else { 0.0 };
            let status = if util > 100.0 { "overallocated" } else if util > 80.0 { "optimal" } else { "available" };
            serde_json::json!({ "name": csv_util::record_get_str(r, "name"), "role": csv_util::record_get_str(r, "role"), "capacity_hours": capacity, "allocated_hours": allocated, "utilization_pct": util, "status": status })
        }).collect();

        let overallocated = analysis
            .iter()
            .filter(|a| a["status"] == "overallocated")
            .count();
        let available = analysis
            .iter()
            .filter(|a| a["status"] == "available")
            .count();

        let llm = ctx
            .call_llm(
                "Optimize resource allocation.",
                &format!(
                    "Resources ({} total, {} overallocated, {} available):\n{}\n\nBalance?",
                    resources.len(),
                    overallocated,
                    available,
                    serde_json::to_string_pretty(&analysis).unwrap_or_default()
                ),
            )
            .await?;
        Ok(
            serde_json::json!({ "summary": { "total_resources": resources.len(), "overallocated": overallocated, "available": available }, "resources": analysis, "llm_analysis": llm }),
        )
    }
}
impl Default for ResourceAllocatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_runtime::{MockAgentContext, MockLlmProvider};
    fn ctx() -> MockAgentContext {
        MockAgentContext::new(MockLlmProvider::with_response("Alice is overallocated."))
    }

    #[tokio::test]
    async fn test_resources() {
        let input = serde_json::json!({ "resources": "name,role,capacity_hours,allocated_hours\nAlice,Dev,160,180\nBob,Dev,160,80" });
        let r = ResourceAllocatorAgent::new()
            .execute_with_context(input, &ctx())
            .await
            .unwrap();
        assert_eq!(r["summary"]["overallocated"], 1);
        assert_eq!(r["summary"]["available"], 1);
    }
    #[tokio::test]
    async fn test_empty() {
        assert!(ResourceAllocatorAgent::new()
            .execute_with_context(serde_json::json!({"resources":""}), &ctx())
            .await
            .is_err());
    }
    #[test]
    fn test_manifest() {
        assert_eq!(
            ResourceAllocatorAgent::new().manifest().id,
            "pmo.resource-allocator"
        );
    }
}
