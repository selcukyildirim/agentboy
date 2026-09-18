use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheUsageMetrics {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub parse_cache_hits: u64,
    pub parse_cache_misses: u64,
    pub embedding_cache_hits: u64,
    pub embedding_cache_misses: u64,
    pub llm_cache_hits: u64,
    pub llm_cache_misses: u64,
    pub prompt_cache_read_tokens: u64,
    pub prompt_cache_write_tokens: u64,
    pub total_tokens_saved: u64,
    pub estimated_cost_saved_usd: f64,
}

impl CacheUsageMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_l1_hit(&mut self) {
        self.l1_hits += 1;
    }

    pub fn record_l1_miss(&mut self) {
        self.l1_misses += 1;
    }

    pub fn record_l2_hit(&mut self) {
        self.l2_hits += 1;
    }

    pub fn record_l2_miss(&mut self) {
        self.l2_misses += 1;
    }

    pub fn record_parse_hit(&mut self) {
        self.parse_cache_hits += 1;
    }

    pub fn record_parse_miss(&mut self) {
        self.parse_cache_misses += 1;
    }

    pub fn record_embedding_hit(&mut self, tokens_saved: u32) {
        self.embedding_cache_hits += 1;
        self.total_tokens_saved += tokens_saved as u64;
    }

    pub fn record_embedding_miss(&mut self) {
        self.embedding_cache_misses += 1;
    }

    pub fn record_llm_hit(&mut self, tokens_saved: u32) {
        self.llm_cache_hits += 1;
        self.total_tokens_saved += tokens_saved as u64;
    }

    pub fn record_llm_miss(&mut self) {
        self.llm_cache_misses += 1;
    }

    pub fn record_prompt_cache(&mut self, read_tokens: u32, write_tokens: u32) {
        self.prompt_cache_read_tokens += read_tokens as u64;
        self.prompt_cache_write_tokens += write_tokens as u64;
    }

    pub fn add_cost_saving(&mut self, usd: f64) {
        self.estimated_cost_saved_usd += usd;
    }

    pub fn hit_ratio(&self) -> f64 {
        let total_hits = self.l1_hits + self.l2_hits;
        let total = total_hits + self.l1_misses + self.l2_misses;
        if total == 0 {
            0.0
        } else {
            total_hits as f64 / total as f64
        }
    }

    pub fn parse_hit_ratio(&self) -> f64 {
        let total = self.parse_cache_hits + self.parse_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.parse_cache_hits as f64 / total as f64
        }
    }

    pub fn embedding_hit_ratio(&self) -> f64 {
        let total = self.embedding_cache_hits + self.embedding_cache_misses;
        if total == 0 {
            0.0
        } else {
            self.embedding_cache_hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_metrics() {
        let mut metrics = CacheUsageMetrics::new();
        metrics.record_l1_hit();
        metrics.record_l1_hit();
        metrics.record_l1_miss();
        assert_eq!(metrics.hit_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_parse_metrics() {
        let mut metrics = CacheUsageMetrics::new();
        metrics.record_parse_hit();
        metrics.record_parse_miss();
        assert_eq!(metrics.parse_hit_ratio(), 0.5);
    }
}
