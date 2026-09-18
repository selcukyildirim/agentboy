use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::{
    CompletionRequest, CompletionResponse, ModelInfo, ProviderCapabilities, Role, ToolCall, Usage,
};

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-api-key", self.api_key.parse().unwrap());
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn id(&self) -> &'static str {
        "anthropic"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4".to_string(),
                capabilities: ProviderCapabilities {
                    text: true,
                    vision: true,
                    tools: true,
                    structured_output: true,
                    reasoning: true,
                    prompt_cache: true,
                    embeddings: false,
                },
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".to_string(),
                name: "Claude 3.5 Haiku".to_string(),
                capabilities: ProviderCapabilities {
                    text: true,
                    vision: true,
                    tools: true,
                    structured_output: true,
                    reasoning: false,
                    prompt_cache: false,
                    embeddings: false,
                },
            },
        ])
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .headers(self.headers())
            .json(&serde_json::json!({
                "model": "claude-3-5-haiku-20241022",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "hi"}]
            }))
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "anthropic".to_string(),
                message: e.to_string(),
            })?;

        Ok(response.status().is_success() || response.status().as_u16() == 400)
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let system_msg = req
            .messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let messages: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "user",
                        _ => "user",
                    },
                    "content": m.content
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "model": req.model,
            "max_tokens": req.max_tokens.unwrap_or(4096),
            "messages": messages,
        });

        if let Some(system) = system_msg {
            body["system"] = serde_json::json!(system);
        }
        if let Some(temp) = req.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(tools) = &req.tools {
            body["tools"] = serde_json::json!(tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters
                    })
                })
                .collect::<Vec<_>>());
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "anthropic".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => AppError::Provider {
                    provider: "anthropic".to_string(),
                    message: "Invalid API key".to_string(),
                },
                429 => AppError::Provider {
                    provider: "anthropic".to_string(),
                    message: "Rate limited".to_string(),
                },
                _ => AppError::Provider {
                    provider: "anthropic".to_string(),
                    message: text,
                },
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "anthropic".to_string(),
            message: e.to_string(),
        })?;

        let content = body["content"]
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|block| block["type"] == "text")
                    .and_then(|block| block["text"].as_str())
            })
            .unwrap_or("")
            .to_string();

        let tool_calls = body["content"].as_array().map(|arr| {
            arr.iter()
                .filter(|block| block["type"] == "tool_use")
                .map(|block| ToolCall {
                    id: block["id"].as_str().unwrap_or("").to_string(),
                    name: block["name"].as_str().unwrap_or("").to_string(),
                    arguments: block["input"].clone(),
                })
                .collect()
        });

        let usage = Usage {
            input_tokens: body["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: body["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            cache_read_tokens: body["usage"]["cache_read_input_tokens"]
                .as_u64()
                .unwrap_or(0) as u32,
            cache_write_tokens: 0,
        };

        Ok(CompletionResponse {
            content,
            tool_calls,
            usage,
            model: body["model"].as_str().unwrap_or(&req.model).to_string(),
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            text: true,
            vision: true,
            tools: true,
            structured_output: true,
            reasoning: true,
            prompt_cache: true,
            embeddings: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id() {
        let provider = AnthropicProvider::new("test-key".to_string());
        assert_eq!(provider.id(), "anthropic");
    }
}
