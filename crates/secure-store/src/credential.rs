use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub provider: String,
    pub secret_ref: String,
    pub created_at: DateTime<Utc>,
    pub last_validated_at: Option<DateTime<Utc>>,
    pub status: CredentialStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CredentialStatus {
    Active,
    Expired,
    Invalid,
    Revoked,
}
