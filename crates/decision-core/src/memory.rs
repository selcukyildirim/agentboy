use crate::context::DecisionContext;
use crate::recommendation::Recommendation;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub decision_type: String,
    pub context: DecisionContext,
    pub recommendation: Recommendation,
    pub outcome: Option<DecisionOutcome>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub selected_option: String,
    pub actual_result: Option<String>,
    pub satisfaction_score: Option<f32>,
    pub lessons_learned: Option<String>,
}

pub struct DecisionMemory {
    pool: SqlitePool,
}

impl DecisionMemory {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        context: &DecisionContext,
        recommendation: &Recommendation,
    ) -> Result<String, agent_common::error::AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let context_json = serde_json::to_string(context)
            .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
        let rec_json = serde_json::to_string(recommendation)
            .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

        sqlx::query(
            "INSERT INTO decision_memory (id, decision_type, context_json, recommendation_json, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))"
        )
        .bind(&id)
        .bind(&context.decision_type.to_string())
        .bind(&context_json)
        .bind(&rec_json)
        .execute(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        Ok(id)
    }

    pub async fn get(
        &self,
        id: &str,
    ) -> Result<Option<DecisionRecord>, agent_common::error::AppError> {
        let row: Option<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, decision_type, context_json, recommendation_json, created_at FROM decision_memory WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        match row {
            Some((id, decision_type, context_json, rec_json, created_at)) => {
                let context: DecisionContext = serde_json::from_str(&context_json)
                    .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
                let recommendation: Recommendation = serde_json::from_str(&rec_json)
                    .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

                Ok(Some(DecisionRecord {
                    id,
                    decision_type,
                    context,
                    recommendation,
                    outcome: None,
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn list_by_type(
        &self,
        decision_type: &str,
        limit: u32,
    ) -> Result<Vec<DecisionRecord>, agent_common::error::AppError> {
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, decision_type, context_json, recommendation_json, created_at
             FROM decision_memory WHERE decision_type = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(decision_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        let mut records = Vec::new();
        for (id, decision_type, context_json, rec_json, created_at) in rows {
            let context: DecisionContext = serde_json::from_str(&context_json)
                .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
            let recommendation: Recommendation = serde_json::from_str(&rec_json)
                .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

            records.push(DecisionRecord {
                id,
                decision_type,
                context,
                recommendation,
                outcome: None,
                created_at,
            });
        }

        Ok(records)
    }

    pub async fn search_similar(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<DecisionRecord>, agent_common::error::AppError> {
        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, decision_type, context_json, recommendation_json, created_at
             FROM decision_memory
             WHERE recommendation_json LIKE ?
             ORDER BY created_at DESC LIMIT ?",
        )
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| agent_common::error::AppError::Database(e.to_string()))?;

        let mut records = Vec::new();
        for (id, decision_type, context_json, rec_json, created_at) in rows {
            let context: DecisionContext = serde_json::from_str(&context_json)
                .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;
            let recommendation: Recommendation = serde_json::from_str(&rec_json)
                .map_err(|e| agent_common::error::AppError::Internal(e.into()))?;

            records.push(DecisionRecord {
                id,
                decision_type,
                context,
                recommendation,
                outcome: None,
                created_at,
            });
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision_type::DecisionType;

    #[tokio::test]
    async fn test_decision_memory_save_and_get() {
        let db = agent_common::db::Database::new(":memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS decision_memory (
                id TEXT PRIMARY KEY,
                decision_type TEXT NOT NULL,
                context_json TEXT NOT NULL,
                recommendation_json TEXT NOT NULL,
                outcome_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(db.pool())
        .await
        .unwrap();

        let memory = DecisionMemory::new(db.pool().clone());

        let mut builder = crate::context::DecisionContextBuilder::new(DecisionType::Procurement);
        builder.add_fact(
            "vendor_quotes",
            serde_json::json!([{"vendor": "A"}]),
            "doc1",
            0.9,
        );
        let context = builder.build();

        let rec = crate::recommendation::Recommendation::new("procurement", "Choose Vendor A");

        let id = memory.save(&context, &rec).await.unwrap();
        let retrieved = memory.get(&id).await.unwrap();
        assert!(retrieved.is_some());
    }
}
