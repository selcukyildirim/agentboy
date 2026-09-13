use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub key: String,
    pub value: T,
    pub created_at: String,
}

pub struct MemoryCache<T> {
    cache: Cache<String, CacheEntry<T>>,
}

impl<T: Clone + Send + Sync + 'static> MemoryCache<T> {
    pub fn new(max_entries: usize, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_entries as u64)
            .time_to_live(ttl)
            .build();

        Self { cache }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        self.cache.get(key).await.map(|entry| entry.value)
    }

    pub async fn insert(&self, key: String, value: T) {
        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.cache.insert(key, entry).await;
    }

    pub async fn remove(&self, key: &str) {
        self.cache.remove(key).await;
    }

    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
