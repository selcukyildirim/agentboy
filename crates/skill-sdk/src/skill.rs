use crate::manifest::SkillManifest;
use agent_common::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait Skill: Send + Sync {
    fn manifest(&self) -> &SkillManifest;
    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value>;
}
