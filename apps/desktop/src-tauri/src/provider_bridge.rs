use agent_common::error::AppResult;
use agent_runtime::context::{
    LlmCompletionRequest, LlmCompletionResponse, LlmProvider, LlmUsage,
};
use async_trait::async_trait;
use cache_core::llm_cache::{CachedLlmResult, LlmResultCache};
use llm_gateway::gateway::LlmProvider as GatewayProvider;
use llm_gateway::types::{CompletionRequest, Message, Role};
use std::sync::Arc;

use crate::resilience;

pub struct GatewayProviderBridge {
    inner: Arc<dyn GatewayProvider>,
    model: String,
}

impl GatewayProviderBridge {
    pub fn new(provider: Arc<dyn GatewayProvider>, model: String) -> Self {
        Self { inner: provider, model }
    }
}

fn convert_role(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User,
    }
}

#[async_trait]
impl LlmProvider for GatewayProviderBridge {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn provider_id(&self) -> &str {
        self.inner.id()
    }

    async fn complete(&self, request: LlmCompletionRequest) -> AppResult<LlmCompletionResponse> {
        let gw_messages: Vec<Message> = request
            .messages
            .iter()
            .map(|m| Message {
                role: convert_role(&m.role),
                content: m.content.clone(),
            })
            .collect();

        let model = request.model.clone().unwrap_or_else(|| self.model.clone());
        let temperature = request.temperature.map(|t| t as f32);

        let gw_request = CompletionRequest {
            model: model.clone(),
            messages: gw_messages,
            temperature,
            max_tokens: request.max_tokens,
            tools: None,
            response_format: None,
        };

        let provider = self.inner.id().to_string();

        // Deterministic-only caching (temperature 0, no tools) via cache-core.
        let cacheable = LlmResultCache::is_cacheable(temperature.unwrap_or(0.0), false);
        let cache_key = if cacheable {
            let system = request
                .messages
                .iter()
                .find(|m| m.role == "system")
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let user = request
                .messages
                .iter()
                .find(|m| m.role == "user")
                .map(|m| m.content.as_str())
                .unwrap_or("");
            let prompt_hash = LlmResultCache::prompt_hash(system, user);
            Some(
                LlmResultCache::new().cache_key(
                    &provider,
                    &model,
                    "v1",
                    &prompt_hash,
                    temperature.unwrap_or(0.0),
                ),
            )
        } else {
            None
        };

        if let Some(key) = &cache_key {
            if let Some(hit) = resilience::cache_get(key).await {
                tracing::debug!(provider = %provider, "LLM cache hit");
                return Ok(LlmCompletionResponse {
                    content: hit.response,
                    model: hit.model,
                    usage: None,
                });
            }
        }

        let inner = self.inner.clone();
        let gw_response = resilience::call_with_resilience(&provider, || {
            let inner = inner.clone();
            let req = gw_request.clone();
            async move { inner.complete(req).await }
        })
        .await?;

        if let (Some(key), true) = (cache_key, cacheable) {
            resilience::cache_put(
                key,
                CachedLlmResult {
                    provider: provider.clone(),
                    model: gw_response.model.clone(),
                    prompt_hash: String::new(),
                    response: gw_response.content.clone(),
                    cached_at: chrono::Utc::now().to_rfc3339(),
                },
            )
            .await;
        }

        let usage = Some(LlmUsage {
            prompt_tokens: gw_response.usage.input_tokens,
            completion_tokens: gw_response.usage.output_tokens,
            total_tokens: gw_response.usage.input_tokens + gw_response.usage.output_tokens,
        });

        Ok(LlmCompletionResponse {
            content: gw_response.content,
            model: gw_response.model,
            usage,
        })
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        self.inner.validate_credentials().await
    }
}
