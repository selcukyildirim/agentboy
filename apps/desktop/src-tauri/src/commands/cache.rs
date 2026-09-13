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

static mut CACHE_STATS: Option<CacheStats> = None;

fn get_stats() -> &'static mut CacheStats {
    unsafe {
        if CACHE_STATS.is_none() {
            CACHE_STATS = Some(CacheStats {
                l1_entries: 0,
                l1_size_bytes: 0,
                l2_entries: 0,
                l2_size_bytes: 0,
                hit_rate: 0.0,
                total_requests: 0,
                total_hits: 0,
            });
        }
        CACHE_STATS.as_mut().unwrap()
    }
}

#[tauri::command]
pub fn get_cache_stats() -> CacheStats {
    get_stats().clone()
}

#[tauri::command]
pub fn clear_cache() -> Result<String, String> {
    let stats = get_stats();
    stats.l1_entries = 0;
    stats.l1_size_bytes = 0;
    stats.l2_entries = 0;
    stats.l2_size_bytes = 0;
    stats.hit_rate = 0.0;
    stats.total_requests = 0;
    stats.total_hits = 0;

    Ok("Cache cleared successfully".to_string())
}