use agent_common::error::{AppError, AppResult};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

/// Single local database file used by every subsystem (executions, audit,
/// workflows, decisions, cache).
pub fn db_path() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
    } else {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
    };
    base.join(".agentboy").join("agentboy.db")
}

pub fn db_url() -> String {
    format!("sqlite:{}?mode=rwc", db_path().display())
}

pub async fn init_pool() -> AppResult<SqlitePool> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Database(format!("create dir: {e}")))?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url())
        .await
        .map_err(|e| AppError::Database(format!("connect db: {e}")))?;

    tracing::info!(path = %path.display(), "Local database ready");
    Ok(pool)
}

/// Create tables owned by the library subsystems (decision memory, workflow
/// history, cache) on the shared pool.
pub async fn init_schema(pool: &SqlitePool) -> AppResult<()> {
    let statements = [
        "CREATE TABLE IF NOT EXISTS decision_memory (
            id TEXT PRIMARY KEY,
            decision_type TEXT NOT NULL,
            context_json TEXT NOT NULL,
            recommendation_json TEXT NOT NULL,
            outcome_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        "CREATE TABLE IF NOT EXISTS workflow_executions (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            workflow_version INTEGER NOT NULL,
            status TEXT NOT NULL,
            data_json TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT
        )",
        "CREATE TABLE IF NOT EXISTS workflow_versions (
            workflow_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            data_json TEXT NOT NULL,
            changelog TEXT,
            created_at TEXT NOT NULL,
            PRIMARY KEY (workflow_id, version)
        )",
        "CREATE TABLE IF NOT EXISTS cache_entries (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            entry_type TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at TEXT
        )",
    ];

    for stmt in statements {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("schema init: {e}")))?;
    }
    Ok(())
}
