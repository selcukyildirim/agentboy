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
