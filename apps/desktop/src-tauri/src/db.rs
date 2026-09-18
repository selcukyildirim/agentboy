use agent_common::db::Database;
use agent_common::error::{AppError, AppResult};
use sqlx::sqlite::SqlitePool;
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

pub fn db_path_string() -> String {
    db_path().to_string_lossy().into_owned()
}

pub fn db_url() -> String {
    format!("sqlite:{}?mode=rwc", db_path().display())
}

/// Initialise the shared database: create the directory, run migrations
/// (agent-common is the single schema source) and apply pragmas.
pub async fn init() -> AppResult<SqlitePool> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Database(format!("create dir: {e}")))?;
    }

    let database = Database::new(&db_path_string()).await?;
    let pool = database.pool().clone();

    apply_pragmas(&pool).await;

    tracing::info!(path = %path.display(), "Local database ready");
    Ok(pool)
}

/// Apply SQLite hardening/performance pragmas.
pub async fn apply_pragmas(pool: &SqlitePool) {
    for pragma in [
        "PRAGMA journal_mode=WAL",
        "PRAGMA foreign_keys=ON",
        "PRAGMA busy_timeout=5000",
        "PRAGMA synchronous=NORMAL",
        "PRAGMA temp_store=MEMORY",
    ] {
        if let Err(e) = sqlx::query(pragma).execute(pool).await {
            tracing::warn!(pragma, error = %e, "pragma failed");
        }
    }
}
