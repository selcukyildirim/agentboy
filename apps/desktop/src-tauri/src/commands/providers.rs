use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub provider: String,
    pub configured: bool,
    pub model: Option<String>,
}

fn get_configs() -> &'static Mutex<HashMap<String, ProviderConfig>> {
    static CONFIGS: OnceLock<Mutex<HashMap<String, ProviderConfig>>> = OnceLock::new();
    CONFIGS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub fn configure_provider(config: ProviderConfig) -> Result<ProviderStatus, String> {
    let provider = config.provider.clone();
    let model = config.model.clone();

    let mut configs = get_configs().lock().unwrap();
    configs.insert(provider.clone(), config);

    tracing::info!(provider = %provider, "Provider configured");

    Ok(ProviderStatus {
        provider,
        configured: true,
        model,
    })
}

#[tauri::command]
pub async fn test_provider(provider: String) -> Result<ProviderStatus, String> {
    let config = {
        let configs = get_configs().lock().unwrap();
        configs.get(&provider).cloned()
    };

    let config = config.ok_or_else(|| format!("Provider {} not configured", provider))?;

    let base_url = config.base_url.as_deref().unwrap_or(match provider.as_str() {
        "openai" => "https://api.openai.com/v1",
        "anthropic" => "https://api.anthropic.com",
        "gemini" => "https://generativelanguage.googleapis.com",
        _ => "",
    });

    let api_key = config.api_key.as_deref().unwrap_or("");

    if api_key.is_empty() {
        return Err(format!("Provider {} has no API key configured", provider));
    }

    let client = reqwest::Client::new();
    let model_works = match provider.as_str() {
        "openai" | "openai-compatible" => {
            let url = format!("{}/models", base_url);
            match client
                .get(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => resp.status().is_success(),
                Err(e) => {
                    tracing::warn!(provider = %provider, error = %e, "Provider test failed");
                    false
                }
            }
        }
        "anthropic" => {
            let url = format!("{}/v1/messages", base_url);
            match client
                .post(&url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    status == 200 || status == 400
                }
                Err(e) => {
                    tracing::warn!(provider = %provider, error = %e, "Provider test failed");
                    false
                }
            }
        }
        "gemini" => {
            let url = format!("{}/v1beta/models?key={}", base_url, api_key);
            match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => resp.status().is_success(),
                Err(e) => {
                    tracing::warn!(provider = %provider, error = %e, "Provider test failed");
                    false
                }
            }
        }
        "ollama" => {
            let url = format!("{}/api/tags", base_url);
            match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) => resp.status().is_success(),
                Err(e) => {
                    tracing::warn!(provider = %provider, error = %e, "Ollama test failed");
                    false
                }
            }
        }
        _ => false,
    };

    if model_works {
        tracing::info!(provider = %provider, "Provider test passed");
        Ok(ProviderStatus {
            provider,
            configured: true,
            model: config.model,
        })
    } else {
        Err(format!(
            "Provider {} test failed — check API key and network connectivity",
            provider
        ))
    }
}

#[tauri::command]
pub fn get_provider_status() -> Vec<ProviderStatus> {
    let configs = get_configs().lock().unwrap();
    configs
        .iter()
        .map(|(name, config)| ProviderStatus {
            provider: name.clone(),
            configured: true,
            model: config.model.clone(),
        })
        .collect()
}
