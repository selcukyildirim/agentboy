use agent_common::error::{AppError, AppResult};
use agent_runtime::context::LlmProvider as AgentLlmProvider;
use llm_gateway::gateway::LlmProvider as GatewayLlmProvider;
use std::sync::Arc;

use crate::provider_bridge::GatewayProviderBridge;

#[derive(Debug, Clone)]
pub struct ProviderSpec {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
}

impl ProviderSpec {
    pub fn default_model_for(provider: &str) -> String {
        match provider {
            "openai" => "gpt-4o-mini",
            "anthropic" => "claude-3-5-haiku-20241022",
            "gemini" => "gemini-2.0-flash",
            "ollama" => "llama3.1",
            _ => "gpt-4o-mini",
        }
        .to_string()
    }
}

fn require_key(provider: &str, key: Option<String>) -> AppResult<String> {
    key.filter(|k| !k.trim().is_empty())
        .ok_or_else(|| AppError::Provider {
            provider: provider.to_string(),
            message: "API key not configured".to_string(),
        })
}

/// Build an agent-runtime LLM provider from a provider spec.
pub fn build(spec: &ProviderSpec) -> AppResult<Arc<dyn AgentLlmProvider>> {
    let provider = spec.provider.as_str();

    let inner: Arc<dyn GatewayLlmProvider> = match provider {
        "openai" => {
            let key = require_key(provider, spec.api_key.clone())?;
            match spec.base_url.as_deref().filter(|b| !b.trim().is_empty()) {
                Some(base) => Arc::new(provider_openai::adapter::OpenAiProvider::with_base_url(
                    key,
                    base.to_string(),
                )),
                None => Arc::new(provider_openai::adapter::OpenAiProvider::new(key)),
            }
        }
        "anthropic" => {
            let key = require_key(provider, spec.api_key.clone())?;
            Arc::new(provider_anthropic::adapter::AnthropicProvider::new(key))
        }
        "gemini" => {
            let key = require_key(provider, spec.api_key.clone())?;
            Arc::new(provider_gemini::adapter::GeminiProvider::new(key))
        }
        "openai-compatible" => {
            let base = spec
                .base_url
                .clone()
                .filter(|b| !b.trim().is_empty())
                .ok_or_else(|| AppError::Provider {
                    provider: provider.to_string(),
                    message: "base_url is required for openai-compatible".to_string(),
                })?;
            Arc::new(
                provider_openai_compatible::adapter::OpenAiCompatibleProvider::new(
                    base,
                    spec.api_key.clone(),
                ),
            )
        }
        "ollama" => Arc::new(provider_ollama::adapter::OllamaProvider::new(
            spec.base_url.clone(),
        )),
        other => {
            return Err(AppError::Provider {
                provider: other.to_string(),
                message: "Unknown provider".to_string(),
            })
        }
    };

    Ok(Arc::new(GatewayProviderBridge::new(
        inner,
        spec.model.clone(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_key_missing() {
        let result = build(&ProviderSpec {
            provider: "openai".to_string(),
            api_key: None,
            base_url: None,
            model: "gpt-4o-mini".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_openai_builds_with_key() {
        let result = build(&ProviderSpec {
            provider: "openai".to_string(),
            api_key: Some("sk-test".to_string()),
            base_url: None,
            model: "gpt-4o-mini".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_provider_errors() {
        let result = build(&ProviderSpec {
            provider: "nope".to_string(),
            api_key: Some("x".to_string()),
            base_url: None,
            model: "m".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_ollama_needs_no_key() {
        let result = build(&ProviderSpec {
            provider: "ollama".to_string(),
            api_key: None,
            base_url: None,
            model: "llama3.1".to_string(),
        });
        assert!(result.is_ok());
    }
}
