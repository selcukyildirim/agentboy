use crate::content_hash::content_hash;

pub struct ParseCache {
    parser_version: String,
}

impl ParseCache {
    pub fn new(parser_version: &str) -> Self {
        Self {
            parser_version: parser_version.to_string(),
        }
    }

    pub fn cache_key(&self, document_hash: &str, filename: &str) -> String {
        let input = format!("{}:{}:{}", self.parser_version, document_hash, filename);
        content_hash(input.as_bytes())
    }

    pub fn is_valid(&self, cached_hash: &str, current_hash: &str) -> bool {
        cached_hash == current_hash
    }

    pub fn invalidate_key(&self, _old_parser_version: &str) -> String {
        self.parser_version.clone()
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new("1.0.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cache_key() {
        let cache = ParseCache::new("1.0.0");
        let key = cache.cache_key("doc-hash-123", "test.pdf");
        assert!(!key.is_empty());
        assert_eq!(key.len(), 64);
    }

    #[test]
    fn test_parse_cache_validity() {
        let cache = ParseCache::new("1.0.0");
        assert!(cache.is_valid("same-hash", "same-hash"));
        assert!(!cache.is_valid("old-hash", "new-hash"));
    }
}