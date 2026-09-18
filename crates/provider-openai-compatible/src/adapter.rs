use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::{
    CompletionRequest, CompletionResponse, Message, ModelInfo, ProviderCapabilities, Role, Usage,
};

pub struct OpenAiCompatibleProvider {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    #[must_use]
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = self.api_key.as_ref().filter(|k| !k.is_empty()) {
            if let Ok(value) = format!("Bearer {key}").parse() {
                headers.insert("Authorization", value);
            }
        }
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }
}

fn role_str(role: &Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &'static str {
        "openai-compatible"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai-compatible".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(AppError::Provider {
                provider: "openai-compatible".to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "openai-compatible".to_string(),
            message: e.to_string(),
        })?;

        Ok(body["data"]
            .as_array()
            .map(|arr: &Vec<serde_json::Value>| {
                arr.iter()
                    .filter_map(|m| {
                        let id = m["id"].as_str()?.to_string();
                        Some(ModelInfo {
                            name: id.clone(),
                            id,
                            capabilities: ProviderCapabilities {
                                text: true,
                                vision: false,
                                tools: true,
                                structured_output: true,
                                reasoning: false,
                                prompt_cache: false,
                                embeddings: false,
                            },
                        })
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai-compatible".to_string(),
                message: e.to_string(),
            })?;
        Ok(response.status().is_success())
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let mut body = serde_json::json!({
            "model": req.model,
            "messages": req.messages.iter().map(|m: &Message| {
                serde_json::json!({
                    "role": role_str(&m.role),
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

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .headers(self.headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "openai-compatible".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(AppError::Provider {
                provider: "openai-compatible".to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "openai-compatible".to_string(),
            message: e.to_string(),
        })?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(CompletionResponse {
            content,
            tool_calls: None,
            usage: Usage {
                input_tokens: json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                output_tokens: json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
            model: json["model"].as_str().unwrap_or("unknown").to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_id_and_capabilities() {
        let provider = OpenAiCompatibleProvider::new(
            "http://localhost:8000/v1/".to_string(),
            Some("key".to_string()),
        );
        assert_eq!(provider.id(), "openai-compatible");
        assert!(provider.capabilities().text);
    }

    #[test]
    fn test_base_url_normalised() {
        let provider = OpenAiCompatibleProvider::new("http://x/v1/".to_string(), None);
        assert_eq!(provider.base_url, "http://x/v1");
    }

    #[test]
    fn test_headers_without_key() {
        let provider = OpenAiCompatibleProvider::new("http://x".to_string(), None);
        assert!(!provider.headers().contains_key("authorization"));
    }
}
