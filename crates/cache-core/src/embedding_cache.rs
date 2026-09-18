use crate::content_hash;

pub struct EmbeddingCache;

impl EmbeddingCache {
    pub fn new() -> Self {
        Self
    }

    pub fn cache_key(&self, content_hash_val: &str, model: &str, chunk_version: &str) -> String {
        let input = format!("{}:{}:{}", content_hash_val, model, chunk_version);
        content_hash::content_hash(input.as_bytes())
    }

    pub fn content_key(&self, content: &[u8], chunk_index: usize) -> String {
        let mut input = content.to_vec();
        input.extend_from_slice(&chunk_index.to_le_bytes());
        content_hash::content_hash(&input)
    }
}

impl Default for EmbeddingCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_cache_key() {
        let cache = EmbeddingCache::new();
        let key = cache.cache_key("content-hash", "gpt-4o", "v1");
        assert!(!key.is_empty());
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_embedding_content_key() {
        let cache = EmbeddingCache::new();
        let key = cache.content_key(b"hello world", 0);
        assert!(!key.is_empty());
    }
}
