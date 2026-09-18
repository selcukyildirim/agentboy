use crate::event::{AuditEvent, AuditResult};
use agent_common::error::{AppError, AppResult};
use chrono::Utc;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

pub struct SqliteAuditStore {
    pool: SqlitePool,
}

impl SqliteAuditStore {
    pub async fn new(database_url: &str) -> AppResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to audit DB: {e}")))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_events (
                event_id TEXT PRIMARY KEY,
                execution_id TEXT,
                agent_id TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                result TEXT NOT NULL,
                details TEXT,
                created_at TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create audit table: {e}")))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_agent_id ON audit_events(agent_id)")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to create index: {e}")))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_created_at ON audit_events(created_at)")
            .execute(&pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to create index: {e}")))?;

        Ok(Self { pool })
    }

    pub async fn record(&self, event: AuditEvent) -> AppResult<()> {
        let details_json = event
            .details
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());

        let result_str = match event.result {
            AuditResult::Success => "success",
            AuditResult::Failure => "failure",
            AuditResult::Denied => "denied",
        };

        sqlx::query(
            "INSERT INTO audit_events (event_id, execution_id, agent_id, action, resource, result, details, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.event_id.to_string())
        .bind(event.execution_id.map(|u| u.to_string()))
        .bind(&event.agent_id)
        .bind(&event.action)
        .bind(&event.resource)
        .bind(result_str)
        .bind(details_json)
        .bind(event.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to record audit event: {e}")))?;

        Ok(())
    }

    pub async fn query(&self, agent_id: &str, limit: usize) -> AppResult<Vec<AuditEvent>> {
        let rows = sqlx::query_as::<_, AuditRow>(
            "SELECT event_id, execution_id, agent_id, action, resource, result, details, created_at
             FROM audit_events
             WHERE agent_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(agent_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query audit events: {e}")))?;

        rows.into_iter()
            .map(std::convert::TryInto::try_into)
            .collect()
    }

    pub async fn query_all(&self, limit: usize) -> AppResult<Vec<AuditEvent>> {
        let rows = sqlx::query_as::<_, AuditRow>(
            "SELECT event_id, execution_id, agent_id, action, resource, result, details, created_at
             FROM audit_events
             ORDER BY created_at DESC
             LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query audit events: {e}")))?;

        rows.into_iter()
            .map(std::convert::TryInto::try_into)
            .collect()
    }

    pub async fn count(&self) -> AppResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_events")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to count audit events: {e}")))?;
        Ok(row.0)
    }

    pub async fn delete_old(&self, days: i64) -> AppResult<u64> {
        let cutoff = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap_or_else(Utc::now)
            .to_rfc3339();

        let result = sqlx::query("DELETE FROM audit_events WHERE created_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete old events: {e}")))?;

        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    event_id: String,
    execution_id: Option<String>,
    agent_id: String,
    action: String,
    resource: String,
    result: String,
    details: Option<String>,
    created_at: String,
}

impl TryFrom<AuditRow> for AuditEvent {
    type Error = agent_common::error::AppError;

    fn try_from(row: AuditRow) -> Result<Self, Self::Error> {
        let event_id = Uuid::parse_str(&row.event_id)
            .map_err(|e| AppError::Validation(format!("Invalid event_id: {e}")))?;

        let execution_id = row
            .execution_id
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| AppError::Validation(format!("Invalid execution_id: {e}")))?;

        let result = match row.result.as_str() {
            "success" => AuditResult::Success,
            "failure" => AuditResult::Failure,
            "denied" => AuditResult::Denied,
            _ => AuditResult::Failure,
        };

        let details = row.details.and_then(|s| serde_json::from_str(&s).ok());

        let timestamp = chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc));

        Ok(Self {
            event_id,
            execution_id,
            agent_id: row.agent_id,
            action: row.action,
            resource: row.resource,
            result,
            details,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_store() -> SqliteAuditStore {
        SqliteAuditStore::new("sqlite::memory:").await.unwrap()
    }

    fn make_event(agent_id: &str, action: &str) -> AuditEvent {
        AuditEvent {
            event_id: Uuid::new_v4(),
            execution_id: Some(Uuid::new_v4()),
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            resource: "/test/resource".to_string(),
            result: AuditResult::Success,
            details: Some(serde_json::json!({"key": "value"})),
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_record_and_query() {
        let store = setup_store().await;
        let event = make_event("agent.finance", "analyze");
        store.record(event.clone()).await.unwrap();

        let results = store.query("agent.finance", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].agent_id, "agent.finance");
        assert_eq!(results[0].action, "analyze");
    }

    #[tokio::test]
    async fn test_query_empty() {
        let store = setup_store().await;
        let results = store.query("nonexistent", 10).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_count() {
        let store = setup_store().await;
        assert_eq!(store.count().await.unwrap(), 0);

        store.record(make_event("a", "action1")).await.unwrap();
        store.record(make_event("a", "action2")).await.unwrap();
        assert_eq!(store.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_query_all() {
        let store = setup_store().await;
        store.record(make_event("a1", "action1")).await.unwrap();
        store.record(make_event("a2", "action2")).await.unwrap();

        let results = store.query_all(10).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_old() {
        let store = setup_store().await;
        store.record(make_event("a", "action")).await.unwrap();

        let deleted = store.delete_old(0).await.unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(store.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_failure_result() {
        let store = setup_store().await;
        let mut event = make_event("a", "action");
        event.result = AuditResult::Failure;
        store.record(event).await.unwrap();

        let results = store.query("a", 10).await.unwrap();
        assert_eq!(results[0].result, AuditResult::Failure);
    }
}
