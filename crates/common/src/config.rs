use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub app: AppSettings,
    pub providers: ProviderSettings,
    pub cache: CacheSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub version: String,
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "AgentBoy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderSettings {
    pub default_provider: Option<String>,
    pub timeout_seconds: u64,
}

impl ProviderSettings {
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds.max(30))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    pub l1_max_entries: usize,
    pub l1_ttl_seconds: u64,
    pub l2_enabled: bool,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            l1_max_entries: 1000,
            l1_ttl_seconds: 300,
            l2_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub path: String,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            path: "agentboy.db".to_string(),
        }
    }
}
