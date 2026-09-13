use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

static mut PROVIDER_CONFIGS: Option<std::collections::HashMap<String, ProviderConfig>> = None;

fn get_configs() -> &'static mut std::collections::HashMap<String, ProviderConfig> {
    unsafe {
        if PROVIDER_CONFIGS.is_none() {
            PROVIDER_CONFIGS = Some(std::collections::HashMap::new());
        }
        PROVIDER_CONFIGS.as_mut().unwrap()
    }
}

#[tauri::command]
pub fn configure_provider(config: ProviderConfig) -> Result<ProviderStatus, String> {
    let provider = config.provider.clone();
    let model = config.model.clone();

    get_configs().insert(provider.clone(), config);

    Ok(ProviderStatus {
        provider,
        configured: true,
        model,
    })
}

#[tauri::command]
pub fn test_provider(provider: String) -> Result<bool, String> {
    let configs = get_configs();
    if configs.contains_key(&provider) {
        Ok(true)
    } else {
        Err(format!("Provider {} not configured", provider))
    }
}

#[tauri::command]
pub fn get_provider_status() -> Vec<ProviderStatus> {
    let configs = get_configs();
    configs.iter().map(|(name, config)| {
        ProviderStatus {
            provider: name.clone(),
            configured: true,
            model: config.model.clone(),
        }
    }).collect()
}