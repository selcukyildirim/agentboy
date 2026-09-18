use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheStats {
    pub l1_entries: usize,
    pub l1_size_bytes: u64,
    pub l2_entries: usize,
    pub l2_size_bytes: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
    pub total_hits: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            l1_entries: 0,
            l1_size_bytes: 0,
            l2_entries: 0,
            l2_size_bytes: 0,
            hit_rate: 0.0,
            total_requests: 0,
            total_hits: 0,
        }
    }
}

#[tauri::command]
pub async fn get_cache_stats() -> Result<CacheStats, String> {
    let entries = crate::resilience::persistent_entry_count().await as usize;
    let m = crate::resilience::cache_metrics_snapshot();

    let hits = m.llm_cache_hits;
    let misses = m.llm_cache_misses;
    let total = hits + misses;
    let hit_rate = if total > 0 {
        hits as f64 / total as f64
    } else {
        0.0
    };

    Ok(CacheStats {
        l1_entries: entries,
        l1_size_bytes: 0,
        l2_entries: entries,
        l2_size_bytes: 0,
        hit_rate,
        total_requests: total,
        total_hits: hits,
    })
}

#[tauri::command]
pub async fn clear_cache() -> Result<String, String> {
    crate::resilience::cache_clear().await;
    Ok("Cache cleared successfully".to_string())
}
