use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PromptCacheMode {
    Disabled,
    Auto,
    ProviderNative,
    Explicit { ttl_seconds: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCacheMetrics {
    pub cache_hit: bool,
    pub cache_read_tokens: u32,
    pub cache_write_tokens: u32,
    pub cache_provider: String,
    pub cache_ttl: Option<u64>,
    pub estimated_saving_usd: f64,
}

pub struct PromptCacheAbstraction {
    mode: PromptCacheMode,
}

impl PromptCacheAbstraction {
    pub fn new(mode: PromptCacheMode) -> Self {
        Self { mode }
    }

    pub fn mode(&self) -> &PromptCacheMode {
        &self.mode
    }

    pub fn is_enabled(&self) -> bool {
        self.mode != PromptCacheMode::Disabled
    }

    pub fn normalize_metrics(
        &self,
        cache_hit: bool,
        read_tokens: u32,
        write_tokens: u32,
        provider: &str,
    ) -> PromptCacheMetrics {
        let cost_per_1k = match provider {
            "openai" => 0.00003,
            "anthropic" => 0.0000375,
            _ => 0.00003,
        };
        let saving = if cache_hit {
            (read_tokens as f64 / 1000.0) * cost_per_1k
        } else {
            0.0
        };

        PromptCacheMetrics {
            cache_hit,
            cache_read_tokens: read_tokens,
            cache_write_tokens: write_tokens,
            cache_provider: provider.to_string(),
            cache_ttl: None,
            estimated_saving_usd: saving,
        }
    }
}

impl Default for PromptCacheAbstraction {
    fn default() -> Self {
        Self::new(PromptCacheMode::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_cache_mode() {
        let abstraction = PromptCacheAbstraction::new(PromptCacheMode::Auto);
        assert!(abstraction.is_enabled());

        let disabled = PromptCacheAbstraction::new(PromptCacheMode::Disabled);
        assert!(!disabled.is_enabled());
    }

    #[test]
    fn test_normalize_metrics() {
        let abstraction = PromptCacheAbstraction::default();
        let metrics = abstraction.normalize_metrics(true, 1000, 0, "openai");
        assert!(metrics.cache_hit);
        assert!(metrics.estimated_saving_usd > 0.0);
    }
}