use agent_common::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub key: String,
    pub entry_type: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

pub struct PersistentCache {
    pool: SqlitePool,
}

impl PersistentCache {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM cache_entries WHERE key = ? AND (expires_at IS NULL OR expires_at > datetime('now'))"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|(v,)| v))
    }

    pub async fn insert(
        &self,
        key: &str,
        entry_type: &str,
        value: &str,
        ttl_seconds: Option<i64>,
    ) -> AppResult<()> {
        let expires_at = ttl_seconds.map(|ttl| format!("datetime('now', '+{} seconds')", ttl));

        sqlx::query(
            "INSERT OR REPLACE INTO cache_entries (key, value, entry_type, created_at, expires_at)
             VALUES (?, ?, ?, datetime('now'), ?)",
        )
        .bind(key)
        .bind(value)
        .bind(entry_type)
        .bind(expires_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn remove(&self, key: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM cache_entries WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn invalidate_by_type(&self, entry_type: &str) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM cache_entries WHERE entry_type = ?")
            .bind(entry_type)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(result.rows_affected())
    }

    pub async fn cleanup_expired(&self) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM cache_entries WHERE expires_at IS NOT NULL AND expires_at < datetime('now')"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(result.rows_affected())
    }

    pub async fn stats(&self) -> AppResult<CacheStats> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cache_entries")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(CacheStats {
            total_entries: row.0 as u64,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persistent_cache() {
        let db = agent_common::db::Database::new(":memory:").await.unwrap();
        let cache = PersistentCache::new(db.pool().clone());

        cache
            .insert("test-key", "test", "hello world", None)
            .await
            .unwrap();
        let value = cache.get("test-key").await.unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "hello world");

        cache.remove("test-key").await.unwrap();
        let removed = cache.get("test-key").await.unwrap();
        assert!(removed.is_none());
    }

    #[tokio::test]
    async fn test_persistent_cache_ttl() {
        let db = agent_common::db::Database::new(":memory:").await.unwrap();
        let cache = PersistentCache::new(db.pool().clone());

        cache
            .insert("ttl-key", "test", "expires soon", Some(1))
            .await
            .unwrap();
        let value = cache.get("ttl-key").await.unwrap();
        assert!(value.is_some());
    }

    #[tokio::test]
    async fn test_invalidate_by_type() {
        let db = agent_common::db::Database::new(":memory:").await.unwrap();
        let cache = PersistentCache::new(db.pool().clone());

        cache.insert("k1", "type_a", "v1", None).await.unwrap();
        cache.insert("k2", "type_b", "v2", None).await.unwrap();

        let removed = cache.invalidate_by_type("type_a").await.unwrap();
        assert_eq!(removed, 1);

        let v1 = cache.get("k1").await.unwrap();
        assert!(v1.is_none());
        let v2 = cache.get("k2").await.unwrap();
        assert!(v2.is_some());
    }
}
