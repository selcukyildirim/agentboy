use crate::types::{CompletionRequest, CompletionResponse, ModelInfo, ProviderCapabilities};
use agent_common::error::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn list_models(&self) -> AppResult<Vec<ModelInfo>>;
    async fn validate_credentials(&self) -> AppResult<bool>;
    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse>;
    fn capabilities(&self) -> ProviderCapabilities;
}
