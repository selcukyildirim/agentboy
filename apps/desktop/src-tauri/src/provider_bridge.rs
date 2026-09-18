use agent_common::error::AppResult;
use agent_runtime::context::{
    LlmCompletionRequest, LlmCompletionResponse, LlmProvider, LlmUsage,
};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider as GatewayProvider;
use llm_gateway::types::{CompletionRequest, Message, Role};
use std::sync::Arc;

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

fn _convert_role_back(role: &Role) -> String {
    match role {
        Role::System => "system".to_string(),
        Role::User => "user".to_string(),
        Role::Assistant => "assistant".to_string(),
        Role::Tool => "tool".to_string(),
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

        let gw_request = CompletionRequest {
            model: request.model.unwrap_or_else(|| self.model.clone()),
            messages: gw_messages,
            temperature: request.temperature.map(|t| t as f32),
            max_tokens: request.max_tokens,
            tools: None,
            response_format: None,
        };

        let gw_response = self.inner.complete(gw_request).await?;

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
