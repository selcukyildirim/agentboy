use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::*;

pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &str {
        "openai-compatible"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        Ok(vec![])
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        Ok(true)
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let _ = req;
        Err(AppError::Provider {
            provider: "openai-compatible".to_string(),
            message: "Not implemented yet".to_string(),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            text: true,
            vision: false,
            tools: true,
            structured_output: true,
            reasoning: false,
            prompt_cache: false,
            embeddings: false,
        }
    }
}
