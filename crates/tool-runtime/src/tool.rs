use crate::manifest::ToolManifest;
use agent_common::error::AppResult;
use agent_common::types::ToolRisk;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn manifest(&self) -> &ToolManifest;
    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value>;
    fn risk_level(&self) -> ToolRisk {
        self.manifest().risk.clone()
    }
}
