use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use llm_gateway::gateway::LlmProvider;
use llm_gateway::types::*;

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn base_url(&self) -> String {
        format!("https://generativelanguage.googleapis.com/v1beta",)
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn id(&self) -> &str {
        "gemini"
    }

    async fn list_models(&self) -> AppResult<Vec<ModelInfo>> {
        let url = format!("{}/models?key={}", self.base_url(), self.api_key);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "gemini".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(AppError::Provider {
                provider: "gemini".to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "gemini".to_string(),
            message: e.to_string(),
        })?;

        let models = body["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let name = m["name"].as_str()?.to_string();
                        let id = name.strip_prefix("models/").unwrap_or(&name).to_string();
                        let display = m["displayName"].as_str().unwrap_or(&id).to_string();
                        Some(ModelInfo {
                            id,
                            name: display,
                            capabilities: ProviderCapabilities {
                                text: true,
                                vision: true,
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
            .unwrap_or_default();

        Ok(models)
    }

    async fn validate_credentials(&self) -> AppResult<bool> {
        let url = format!("{}/models?key={}", self.base_url(), self.api_key);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "gemini".to_string(),
                message: e.to_string(),
            })?;

        Ok(response.status().is_success())
    }

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let model = &req.model;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url(),
            model,
            self.api_key
        );

        let contents: Vec<serde_json::Value> = req
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::User => "user",
                        Role::Assistant => "model",
                        _ => "user",
                    },
                    "parts": [{"text": m.content}]
                })
            })
            .collect();

        let mut body = serde_json::json!({
            "contents": contents,
        });

        if let Some(tools) = &req.tools {
            body["tools"] = serde_json::json!([{
                "function_declarations": tools.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    })
                }).collect::<Vec<_>>()
            }]);
        }

        if let Some(system) = req.messages.iter().find(|m| m.role == Role::System) {
            body["systemInstruction"] = serde_json::json!({
                "parts": [{"text": system.content}]
            });
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider {
                provider: "gemini".to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Provider {
                provider: "gemini".to_string(),
                message: text,
            });
        }

        let body: serde_json::Value = response.json().await.map_err(|e| AppError::Provider {
            provider: "gemini".to_string(),
            message: e.to_string(),
        })?;

        let content = body["candidates"][0]["content"]["parts"]
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|p| p["text"].is_string())
                    .and_then(|p| p["text"].as_str())
            })
            .unwrap_or("")
            .to_string();

        let tool_calls = body["candidates"][0]["content"]["parts"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|p| p["functionCall"].is_object())
                    .enumerate()
                    .map(|(i, p)| ToolCall {
                        id: format!("call_{}", i),
                        name: p["functionCall"]["name"].as_str().unwrap_or("").to_string(),
                        arguments: p["functionCall"]["args"].clone(),
                    })
                    .collect()
            });

        let usage_meta = &body["usageMetadata"];
        let usage = Usage {
            input_tokens: usage_meta["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            output_tokens: usage_meta["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        Ok(CompletionResponse {
            content,
            tool_calls,
            usage,
            model: req.model,
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            text: true,
            vision: true,
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
    fn test_provider_id() {
        let provider = GeminiProvider::new("test-key".to_string());
        assert_eq!(provider.id(), "gemini");
    }
}
