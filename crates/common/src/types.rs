use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ExecutionId = Uuid;
pub type AgentId = String;
pub type SkillId = String;
pub type WorkflowId = Uuid;
pub type TenantId = Uuid;
pub type UserId = Uuid;
pub type DocumentId = Uuid;
pub type ChunkId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataClassification {
    L0LocalOnly,
    L1MetadataOnly,
    L2Sanitized,
    L3MinimumRequired,
    L4FullContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolRisk {
    Read,
    Write,
    Destructive,
    External,
    Financial,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntitlementResult {
    Allow,
    Deny,
    RequireApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub execution_id: ExecutionId,
    pub agent_id: AgentId,
    pub agent_version: String,
    pub workflow_id: Option<WorkflowId>,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub model_calls: Vec<ModelCallRecord>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_id: String,
    pub risk: ToolRisk,
    pub input_hash: Option<String>,
    pub output_hash: Option<String>,
    pub duration_ms: u64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCallRecord {
    pub provider: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_write_tokens: u32,
    pub duration_ms: u64,
    pub estimated_cost_usd: f64,
}
