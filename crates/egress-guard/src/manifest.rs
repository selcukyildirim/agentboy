use agent_common::types::DataClassification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressManifest {
    pub provider: String,
    pub classification: DataClassification,
    pub files_uploaded: u32,
    pub fields_included: Vec<String>,
    pub pii_removed: bool,
    pub secrets_removed: bool,
    pub reason: String,
}
