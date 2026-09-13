use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::*;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn id(&self) -> &str {
        "ollama"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "ollama".to_string(),
                message: e.to_string(),
            })?;

        if response.status().is_success() {
            let _ = response.json::<serde_json::Value>().await;
            Ok(vec![])
        } else {
            Ok(vec![])
        }
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "ollama".to_string(),
                message: e.to_string(),
            })?;

        Ok(response.status().is_success())
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let _ = req;
        Err(AppError::Provider {
            provider: "ollama".to_string(),
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
