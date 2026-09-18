use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub department: String,
    pub description: String,
    pub tier: AgentTier,
    pub skills: Vec<String>,
    pub permissions: AgentPermissions,
    pub execution: ExecutionLimits,
    #[serde(default)]
    pub rag_enabled: bool,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub max_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentTier {
    Free,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub filesystem_read: bool,
    pub filesystem_write: bool,
    pub network_llm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub max_steps: u32,
    pub timeout_seconds: u64,
}