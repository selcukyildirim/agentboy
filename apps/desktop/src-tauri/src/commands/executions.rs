use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub step_type: String,
    pub description: String,
    pub status: String,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Execution {
    pub id: String,
    pub agent_id: String,
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
    pub output_tokens: u32,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub steps: Vec<ExecutionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub count: u64,
    pub success: u64,
    pub failed: u64,
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub p50_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub avg_cost_usd: f64,
    pub daily: Vec<DailyStat>,
}

pub async fn init(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_executions (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            status TEXT NOT NULL,
            input TEXT,
            output TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            duration_ms INTEGER,
            model TEXT,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            cost_usd REAL,
            error TEXT,
            steps TEXT
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn append_execution(pool: &SqlitePool, e: &Execution) -> Result<(), String> {
    let input = serde_json::to_string(&e.input).unwrap_or_else(|_| "null".into());
    let output = e.output.as_ref().map(|o| o.to_string());
    let steps = serde_json::to_string(&e.steps).unwrap_or_else(|_| "[]".into());

    sqlx::query(
        "INSERT OR REPLACE INTO agent_executions
         (id, agent_id, status, input, output, started_at, completed_at, duration_ms,
          model, input_tokens, output_tokens, cost_usd, error, steps)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&e.id)
    .bind(&e.agent_id)
    .bind(&e.status)
    .bind(input)
    .bind(output)
    .bind(&e.started_at)
    .bind(&e.completed_at)
    .bind(e.duration_ms.map(|d| d as i64))
    .bind(&e.model)
    .bind(e.input_tokens as i64)
    .bind(e.output_tokens as i64)
    .bind(e.cost_usd)
    .bind(&e.error)
    .bind(steps)
    .execute(pool)
    .await
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn row_to_execution(row: &sqlx::sqlite::SqliteRow) -> Execution {
    let input: Option<String> = row.get("input");
    let output: Option<String> = row.get("output");
    let steps: Option<String> = row.get("steps");
    Execution {
        id: row.get("id"),
        agent_id: row.get("agent_id"),
        status: row.get("status"),
        input: input
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(serde_json::json!({})),
        output: output.and_then(|s| serde_json::from_str(&s).ok()),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        duration_ms: row.get::<Option<i64>, _>("duration_ms").map(|d| d as u64),
        model: row.get("model"),
        input_tokens: row.get::<Option<i64>, _>("input_tokens").unwrap_or(0) as u32,
        output_tokens: row.get::<Option<i64>, _>("output_tokens").unwrap_or(0) as u32,
        cost_usd: row.get("cost_usd"),
        error: row.get("error"),
        steps: steps
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
    }
}

#[tauri::command]
pub async fn list_executions(
    state: tauri::State<'_, crate::app_state::AppState>,
) -> Result<Vec<Execution>, String> {
    let rows = sqlx::query("SELECT * FROM agent_executions ORDER BY started_at DESC LIMIT 500")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(row_to_execution).collect())
}

#[tauri::command]
pub async fn get_execution(
    state: tauri::State<'_, crate::app_state::AppState>,
    execution_id: String,
) -> Result<Execution, String> {
    let row = sqlx::query("SELECT * FROM agent_executions WHERE id = ?")
        .bind(&execution_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Execution {execution_id} not found"))?;
    Ok(row_to_execution(&row))
}

#[tauri::command]
pub async fn get_execution_steps(
    state: tauri::State<'_, crate::app_state::AppState>,
    execution_id: String,
) -> Result<Vec<ExecutionStep>, String> {
    let execution = get_execution(state, execution_id).await?;
    Ok(execution.steps)
}

#[tauri::command]
pub async fn delete_execution(
    state: tauri::State<'_, crate::app_state::AppState>,
    execution_id: String,
) -> Result<(), String> {
    sqlx::query("DELETE FROM agent_executions WHERE id = ?")
        .bind(&execution_id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_execution_stats(
    state: tauri::State<'_, crate::app_state::AppState>,
    days: Option<u32>,
) -> Result<ExecutionStats, String> {
    let days = days.unwrap_or(7) as i64;
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days)).to_rfc3339();

    let rows =
        sqlx::query("SELECT * FROM agent_executions WHERE started_at >= ? ORDER BY started_at ASC")
            .bind(cutoff)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

    let execs: Vec<Execution> = rows.iter().map(row_to_execution).collect();

    let total = execs.len() as u64;
    let success = execs.iter().filter(|e| e.status == "completed").count() as u64;
    let failed = execs.iter().filter(|e| e.status == "failed").count() as u64;
    let success_rate = if total > 0 {
        success as f64 / total as f64
    } else {
        0.0
    };

    let mut durations: Vec<f64> = execs
        .iter()
        .filter_map(|e| e.duration_ms.map(|d| d as f64))
        .collect();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg_duration_ms = if durations.is_empty() {
        0.0
    } else {
        durations.iter().sum::<f64>() / durations.len() as f64
    };
    let percentile = |p: f64| -> f64 {
        if durations.is_empty() {
            return 0.0;
        }
        let idx = ((durations.len() - 1) as f64 * p).round() as usize;
        durations[idx.min(durations.len() - 1)]
    };

    let total_input_tokens: u64 = execs.iter().map(|e| e.input_tokens as u64).sum();
    let total_output_tokens: u64 = execs.iter().map(|e| e.output_tokens as u64).sum();
    let total_cost_usd: f64 = execs.iter().filter_map(|e| e.cost_usd).sum();
    let avg_cost_usd = if total > 0 {
        total_cost_usd / total as f64
    } else {
        0.0
    };

    let mut by_day: std::collections::BTreeMap<String, DailyStat> =
        std::collections::BTreeMap::new();
    for e in &execs {
        let date = e.started_at.get(0..10).unwrap_or("").to_string();
        let entry = by_day.entry(date.clone()).or_insert(DailyStat {
            date,
            count: 0,
            success: 0,
            failed: 0,
            tokens: 0,
            cost_usd: 0.0,
        });
        entry.count += 1;
        if e.status == "completed" {
            entry.success += 1;
        } else if e.status == "failed" {
            entry.failed += 1;
        }
        entry.tokens += (e.input_tokens + e.output_tokens) as u64;
        entry.cost_usd += e.cost_usd.unwrap_or(0.0);
    }

    Ok(ExecutionStats {
        total,
        success,
        failed,
        success_rate,
        avg_duration_ms,
        p50_duration_ms: percentile(0.50),
        p95_duration_ms: percentile(0.95),
        total_input_tokens,
        total_output_tokens,
        total_cost_usd,
        avg_cost_usd,
        daily: by_day.into_values().collect(),
    })
}
