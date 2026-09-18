use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::{
    CompletionRequest, CompletionResponse, ModelInfo, ProviderCapabilities, Role, ToolCall, Usage,
};

pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(AppError::Provider {
                provider: "openai".to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "openai".to_string(),
            message: e.to_string(),
        })?;

        let models = body["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let id = m["id"].as_str()?.to_string();
                        let name = id.clone();
                        let capabilities = ProviderCapabilities {
                            text: true,
                            vision: id.contains("vision") || id.contains("4o"),
                            tools: !id.contains("mini"),
                            structured_output: true,
                            reasoning: id.contains("4o"),
                            prompt_cache: true,
                            embeddings: false,
                        };
                        Some(ModelInfo {
                            id,
                            name,
                            capabilities,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai".to_string(),
                message: e.to_string(),
            })?;

        Ok(response.status().is_success())
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let mut body = serde_json::json!({
            "model": req.model,
            "messages": req.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
        });

        if let Some(temp) = req.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        if let Some(max) = req.max_tokens {
            body["max_tokens"] = serde_json::json!(max);
        }
        if let Some(tools) = &req.tools {
            body["tools"] = serde_json::json!(tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters
                        }
                    })
                })
                .collect::<Vec<_>>());
        }
        if let Some(fmt) = &req.response_format {
            body["response_format"] = serde_json::json!({
                "type": fmt.r#type,
                "schema": fmt.schema
            });
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(match status.as_u16() {
                401 => AppError::Provider {
                    provider: "openai".to_string(),
                    message: "Invalid API key".to_string(),
                },
                429 => AppError::Provider {
                    provider: "openai".to_string(),
                    message: "Rate limited".to_string(),
                },
                _ => AppError::Provider {
                    provider: "openai".to_string(),
                    message: text,
                },
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "openai".to_string(),
            message: e.to_string(),
        })?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let tool_calls = body["choices"][0]["message"]["tool_calls"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tc| {
                        Some(ToolCall {
                            id: tc["id"].as_str()?.to_string(),
                            name: tc["function"]["name"].as_str()?.to_string(),
                            arguments: serde_json::from_str(tc["function"]["arguments"].as_str()?)
                                .ok()?,
                        })
                    })
                    .collect()
            });

        let usage = Usage {
            input_tokens: body["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: body["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            cache_read_tokens: body["usage"]["prompt_tokens_details"]["cached_tokens"]
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
        let provider = OpenAiProvider::new("test-key".to_string());
        assert_eq!(provider.id(), "openai");
    }

    #[test]
    fn test_capabilities() {
        let provider = OpenAiProvider::new("test-key".to_string());
        let caps = provider.capabilities();
        assert!(caps.text);
        assert!(caps.vision);
        assert!(caps.tools);
        assert!(caps.structured_output);
    }
}
