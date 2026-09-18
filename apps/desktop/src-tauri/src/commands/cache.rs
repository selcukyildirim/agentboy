use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

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

fn get_stats() -> &'static Mutex<CacheStats> {
    static CACHE: OnceLock<Mutex<CacheStats>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(CacheStats::default()))
}

#[tauri::command]
pub fn get_cache_stats() -> CacheStats {
    get_stats().lock().unwrap().clone()
}

#[tauri::command]
pub fn clear_cache() -> Result<String, String> {
    *get_stats().lock().unwrap() = CacheStats::default();
    Ok("Cache cleared successfully".to_string())
}
