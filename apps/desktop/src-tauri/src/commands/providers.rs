use chrono::Utc;
use secure_store::credential::{CredentialStatus, StoredCredential};
use secure_store::store::{OsKeychainStore, SecureStore};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderStatus {
    pub provider: String,
    pub configured: bool,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
struct ProviderMeta {
    base_url: Option<String>,
    model: Option<String>,
}

fn get_meta() -> &'static Mutex<HashMap<String, ProviderMeta>> {
    static META: OnceLock<Mutex<HashMap<String, ProviderMeta>>> = OnceLock::new();
    META.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_active() -> &'static Mutex<Option<String>> {
    static ACTIVE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(None))
}

/// The currently selected provider plus its base_url and model.
pub fn active_provider_meta() -> Option<(String, Option<String>, Option<String>)> {
    let active = get_active().lock().unwrap().clone();
    let meta = get_meta().lock().unwrap();

    let provider = match active {
        Some(p) if meta.contains_key(&p) => p,
        _ => meta.keys().next().cloned()?,
    };

    let m = meta.get(&provider)?;
    Some((provider, m.base_url.clone(), m.model.clone()))
}

#[tauri::command]
pub fn set_active_provider(provider: String) -> Result<(), String> {
    if !get_meta().lock().unwrap().contains_key(&provider) {
        return Err(format!("Provider {provider} is not configured"));
    }
    *get_active().lock().unwrap() = Some(provider);
    Ok(())
}

#[tauri::command]
pub fn get_active_provider() -> Option<String> {
    get_active().lock().unwrap().clone()
}

fn secret_ref(provider: &str) -> String {
    format!("keyring:{provider}")
}

async fn store_secret(provider: &str, secret: &str) -> Result<(), String> {
    let store = OsKeychainStore::new();
    let cred = StoredCredential {
        provider: provider.to_string(),
        secret_ref: secret_ref(provider),
        secret: Some(secret.to_string()),
        created_at: Utc::now(),
        last_validated_at: None,
        status: CredentialStatus::Active,
    };
    store
        .save_credential(&cred)
        .await
        .map_err(|e| e.to_string())
}

pub async fn read_secret(provider: &str) -> Option<String> {
    let store = OsKeychainStore::new();
    match store.get_credential(provider).await {
        Ok(Some(cred)) => cred.secret,
        _ => None,
    }
}

async fn delete_secret(provider: &str) -> Result<(), String> {
    let store = OsKeychainStore::new();
    store
        .delete_credential(provider)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn configure_provider(config: ProviderConfig) -> Result<ProviderStatus, String> {
    let provider = config.provider.clone();
    let model = config.model.clone();
    let base_url = config.base_url.clone();

    if let Some(key) = config.api_key.as_ref().filter(|k| !k.trim().is_empty()) {
        store_secret(&provider, key).await?;
    }

    get_meta().lock().unwrap().insert(
        provider.clone(),
        ProviderMeta {
            base_url: base_url.clone(),
            model: model.clone(),
        },
    );
    *get_active().lock().unwrap() = Some(provider.clone());

    tracing::info!(provider = %provider, "Provider configured (secret stored in keychain)");

    Ok(ProviderStatus {
        provider,
        configured: true,
        model,
        base_url,
    })
}

#[tauri::command]
pub async fn remove_provider_credential(provider: String) -> Result<(), String> {
    delete_secret(&provider).await?;
    get_meta().lock().unwrap().remove(&provider);
    tracing::info!(provider = %provider, "Provider credential removed");
    Ok(())
}

#[tauri::command]
pub async fn get_provider_status() -> Vec<ProviderStatus> {
    let meta = get_meta().lock().unwrap().clone();
    meta.into_iter()
        .map(|(provider, m)| ProviderStatus {
            provider,
            configured: true,
            model: m.model,
            base_url: m.base_url,
        })
        .collect()
}

fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "anthropic" => "https://api.anthropic.com",
        "gemini" => "https://generativelanguage.googleapis.com",
        "ollama" => "http://localhost:11434",
        _ => "",
    }
}

/// Lightweight connectivity/credential check.
#[tauri::command]
pub async fn test_provider(
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<ProviderStatus, String> {
    let key = match api_key.filter(|k| !k.trim().is_empty()) {
        Some(k) => Some(k),
        None => read_secret(&provider).await,
    };

    let base = base_url
        .filter(|b| !b.trim().is_empty())
        .or_else(|| {
            get_meta()
                .lock()
                .unwrap()
                .get(&provider)
                .and_then(|m| m.base_url.clone())
        })
        .unwrap_or_else(|| default_base_url(&provider).to_string());

    let client = reqwest::Client::new();
    let ok = match provider.as_str() {
        "openai" | "openai-compatible" => {
            let key = key.as_deref().unwrap_or("");
            if key.is_empty() && provider == "openai" {
                return Err("OpenAI requires an API key".to_string());
            }
            let url = format!("{}/models", base);
            let mut req = client.get(&url).timeout(std::time::Duration::from_secs(10));
            if !key.is_empty() {
                req = req.header("Authorization", format!("Bearer {key}"));
            }
            matches!(req.send().await, Ok(r) if r.status().is_success())
        }
        "anthropic" => {
            let key = key.ok_or_else(|| "Anthropic requires an API key".to_string())?;
            let url = format!("{}/v1/messages", base);
            match client
                .post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(r) => {
                    let s = r.status().as_u16();
                    s == 200 || s == 400
                }
                Err(_) => false,
            }
        }
        "gemini" => {
            let key = key.ok_or_else(|| "Gemini requires an API key".to_string())?;
            let url = format!("{}/v1beta/models?key={}", base, key);
            matches!(
                client.get(&url).timeout(std::time::Duration::from_secs(10)).send().await,
                Ok(r) if r.status().is_success()
            )
        }
        "ollama" => {
            let url = format!("{}/api/tags", base);
            matches!(
                client.get(&url).timeout(std::time::Duration::from_secs(5)).send().await,
                Ok(r) if r.status().is_success()
            )
        }
        _ => false,
    };

    if ok {
        let model = get_meta()
            .lock()
            .unwrap()
            .get(&provider)
            .and_then(|m| m.model.clone());
        Ok(ProviderStatus {
            provider,
            configured: true,
            model,
            base_url: Some(base),
        })
    } else {
        Err(format!(
            "Provider {provider} test failed — check credentials and connectivity"
        ))
    }
}

/// Fetch the list of model ids available for a provider.
#[tauri::command]
pub async fn list_provider_models(provider: String) -> Result<Vec<String>, String> {
    let key = read_secret(&provider).await;
    let base = get_meta()
        .lock()
        .unwrap()
        .get(&provider)
        .and_then(|m| m.base_url.clone())
        .unwrap_or_else(|| default_base_url(&provider).to_string());

    let client = reqwest::Client::new();
    match provider.as_str() {
        "openai" | "openai-compatible" => {
            let key = key.ok_or_else(|| "API key required".to_string())?;
            let url = format!("{}/models", base);
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {key}"))
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = json["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(models)
        }
        "ollama" => {
            let url = format!("{}/api/tags", base);
            let resp = client
                .get(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = json["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Ok(models)
        }
        _ => Ok(vec![]),
    }
}
