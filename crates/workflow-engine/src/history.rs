use crate::runner::WorkflowExecution;
use crate::workflow::Workflow;
use agent_common::error::AppResult;
use sqlx::sqlite::SqlitePool;

pub struct WorkflowHistory {
    pool: SqlitePool,
}

impl WorkflowHistory {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save_execution(&self, execution: &WorkflowExecution) -> AppResult<()> {
        let json = serde_json::to_string(execution)
            .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO workflow_executions (id, workflow_id, workflow_version, status, data_json, started_at, completed_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&execution.id)
        .bind(&execution.workflow_id)
        .bind(execution.workflow_version)
        .bind(format!("{:?}", execution.status))
        .bind(&json)
        .bind(&execution.started_at)
        .bind(&execution.completed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_execution(&self, id: &str) -> AppResult<Option<WorkflowExecution>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT data_json FROM workflow_executions WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        match row {
            Some((json,)) => {
                let execution: WorkflowExecution = serde_json::from_str(&json)
                    .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
                Ok(Some(execution))
            }
            None => Ok(None),
        }
    }

    pub async fn list_by_workflow(
        &self,
        workflow_id: &str,
        limit: u32,
    ) -> AppResult<Vec<WorkflowExecution>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT data_json FROM workflow_executions WHERE workflow_id = ? ORDER BY started_at DESC LIMIT ?"
        )
        .bind(workflow_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        let mut executions = Vec::new();
        for (json,) in rows {
            if let Ok(exec) = serde_json::from_str::<WorkflowExecution>(&json) {
                executions.push(exec);
            }
        }
        Ok(executions)
    }

    pub async fn save_version(&self, workflow: &Workflow, changelog: &str) -> AppResult<()> {
        let json = serde_json::to_string(workflow)
            .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

        sqlx::query(
            "INSERT INTO workflow_versions (workflow_id, version, data_json, changelog, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind(&workflow.id)
        .bind(workflow.version)
        .bind(&json)
        .bind(changelog)
        .execute(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_version(
        &self,
        workflow_id: &str,
        version: u32,
    ) -> AppResult<Option<Workflow>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT data_json FROM workflow_versions WHERE workflow_id = ? AND version = ?",
        )
        .bind(workflow_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        match row {
            Some((json,)) => {
                let workflow: Workflow = serde_json::from_str(&json)
                    .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
                Ok(Some(workflow))
            }
            None => Ok(None),
        }
    }

    pub async fn list_versions(&self, workflow_id: &str) -> AppResult<Vec<u32>> {
        let rows: Vec<(u32,)> = sqlx::query_as(
            "SELECT version FROM workflow_versions WHERE workflow_id = ? ORDER BY version DESC",
        )
        .bind(workflow_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(v,)| v).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::ExecutionStatus;

    #[tokio::test]
    async fn test_workflow_history() {
        let db = agent_common::db::Database::new(":memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow_executions (
                id TEXT PRIMARY KEY,
                workflow_id TEXT NOT NULL,
                workflow_version INTEGER NOT NULL,
                status TEXT NOT NULL,
                data_json TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT
            )",
        )
        .execute(db.pool())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workflow_versions (
                workflow_id TEXT NOT NULL,
                version INTEGER NOT NULL,
                data_json TEXT NOT NULL,
                changelog TEXT,
                created_at TEXT NOT NULL,
                PRIMARY KEY (workflow_id, version)
            )",
        )
        .execute(db.pool())
        .await
        .unwrap();

        let history = WorkflowHistory::new(db.pool().clone());

        let execution = WorkflowExecution {
            id: "exec1".to_string(),
            workflow_id: "wf1".to_string(),
            workflow_version: 1,
            status: ExecutionStatus::Completed,
            input: serde_json::json!({}),
            output: Some(serde_json::json!({"result": "ok"})),
            step_results: vec![],
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
            duration_ms: Some(100),
            error: None,
        };

        history.save_execution(&execution).await.unwrap();
        let retrieved = history.get_execution("exec1").await.unwrap();
        assert!(retrieved.is_some());
    }
}
