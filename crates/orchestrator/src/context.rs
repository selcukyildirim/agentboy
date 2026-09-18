use agent_common::types::ExecutionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: ExecutionId,
    pub agent_id: String,
    pub agent_version: String,
    pub workflow_id: Option<String>,
    pub step: u32,
    pub max_steps: u32,
    pub created_at: String,
    pub timeout_seconds: u64,
}

impl ExecutionContext {
    pub fn new(
        execution_id: ExecutionId,
        agent_id: String,
        agent_version: String,
        max_steps: u32,
        timeout_seconds: u64,
    ) -> Self {
        Self {
            execution_id,
            agent_id,
            agent_version,
            workflow_id: None,
            step: 0,
            max_steps,
            created_at: chrono::Utc::now().to_rfc3339(),
            timeout_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let created = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        let elapsed = chrono::Utc::now() - created;
        elapsed.num_seconds() as u64 > self.timeout_seconds
    }
}
