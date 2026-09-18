use crate::content_hash::content_hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLlmResult {
    pub provider: String,
    pub model: String,
    pub prompt_hash: String,
    pub response: String,
    pub cached_at: String,
}

pub struct LlmResultCache;

impl LlmResultCache {
    pub fn new() -> Self {
        Self
    }

    pub fn cache_key(
        &self,
        provider: &str,
        model: &str,
        system_prompt_version: &str,
        prompt_hash: &str,
        temperature: f32,
    ) -> String {
        let input = format!(
            "{}:{}:{}:{}:{:.1}",
            provider, model, system_prompt_version, prompt_hash, temperature
        );
        content_hash(input.as_bytes())
    }

    pub fn is_cacheable(temperature: f32, has_tools: bool) -> bool {
        temperature == 0.0 && !has_tools
    }

    pub fn prompt_hash(system_prompt: &str, user_prompt: &str) -> String {
        let combined = format!("{}:{}", system_prompt, user_prompt);
        content_hash(combined.as_bytes())
    }
}

impl Default for LlmResultCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_cache_key() {
        let cache = LlmResultCache::new();
        let key = cache.cache_key("openai", "gpt-4o", "v1", "hash123", 0.0);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_is_cacheable() {
        assert!(LlmResultCache::is_cacheable(0.0, false));
        assert!(!LlmResultCache::is_cacheable(0.7, false));
        assert!(!LlmResultCache::is_cacheable(0.0, true));
    }

    #[test]
    fn test_prompt_hash() {
        let hash = LlmResultCache::prompt_hash("system", "user");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
    }
}
