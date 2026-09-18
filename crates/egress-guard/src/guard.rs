use crate::classifier::DataClassifier;
use crate::manifest::EgressManifest;
use agent_common::error::AppResult;
use agent_common::types::DataClassification;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub provider: String,
    pub classification: DataClassification,
    pub action: String,
    pub blocked: bool,
    pub reason: Option<String>,
}

pub struct EgressGuard {
    max_classification: DataClassification,
    classifier: DataClassifier,
    audit_log: Vec<AuditEntry>,
}

impl EgressGuard {
    #[must_use]
    pub const fn new(max_classification: DataClassification) -> Self {
        Self {
            max_classification,
            classifier: DataClassifier::new(),
            audit_log: Vec::new(),
        }
    }

    pub fn check_classification(&self, requested: &DataClassification) -> AppResult<()> {
        if requested > &self.max_classification {
            return Err(agent_common::error::AppError::EgressBlocked {
                reason: format!(
                    "Requested classification {:?} exceeds allowed {:?}",
                    requested, self.max_classification
                ),
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn classify_content(&self, content: &str) -> DataClassification {
        self.classifier.classify(content)
    }

    pub fn check_and_sanitize(
        &mut self,
        content: &str,
        provider: &str,
    ) -> AppResult<(String, DataClassification)> {
        let classification = self.classify_content(content);

        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider: provider.to_string(),
            classification: classification.clone(),
            action: "check".to_string(),
            blocked: false,
            reason: None,
        };
        self.audit_log.push(entry);

        if let Err(e) = self.check_classification(&classification) {
            let block_entry = AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                provider: provider.to_string(),
                classification: classification.clone(),
                action: "block".to_string(),
                blocked: true,
                reason: Some(e.to_string()),
            };
            self.audit_log.push(block_entry);
            return Err(e);
        }

        let sanitized = self.classifier.sanitize(content);
        Ok((sanitized, classification))
    }

    #[must_use]
    pub fn get_audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    #[must_use]
    pub const fn classifier(&self) -> &DataClassifier {
        &self.classifier
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn build_manifest(
        &self,
        provider: &str,
        classification: DataClassification,
        files_uploaded: u32,
        fields_included: Vec<String>,
        pii_removed: bool,
        secrets_removed: bool,
        reason: &str,
    ) -> EgressManifest {
        EgressManifest {
            provider: provider.to_string(),
            classification,
            files_uploaded,
            fields_included,
            pii_removed,
            secrets_removed,
            reason: reason.to_string(),
        }
    }
}

impl Default for EgressGuard {
    fn default() -> Self {
        Self::new(DataClassification::L3MinimumRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_classification_allowed() {
        let guard = EgressGuard::new(DataClassification::L3MinimumRequired);
        assert!(guard
            .check_classification(&DataClassification::L1MetadataOnly)
            .is_ok());
    }

    #[test]
    fn test_check_classification_blocked() {
        let guard = EgressGuard::new(DataClassification::L1MetadataOnly);
        assert!(guard
            .check_classification(&DataClassification::L3MinimumRequired)
            .is_err());
    }

    #[test]
    fn test_classify_content() {
        let guard = EgressGuard::new(DataClassification::L3MinimumRequired);
        let class = guard.classify_content("normal text content that is longer than one hundred characters to trigger the minimum required classification level for testing purposes");
        assert!(matches!(class, DataClassification::L3MinimumRequired));
    }

    #[test]
    fn test_sanitize_pii() {
        let mut guard = EgressGuard::new(DataClassification::L3MinimumRequired);
        let (sanitized, _class) = guard
            .check_and_sanitize("Email user@example.com for info", "openai")
            .unwrap();
        assert!(sanitized.contains("[REDACTED_PII]"));
    }

    #[test]
    fn test_audit_log() {
        let mut guard = EgressGuard::new(DataClassification::L3MinimumRequired);
        let _ = guard.check_and_sanitize("test content", "openai");
        assert!(!guard.get_audit_log().is_empty());
    }
}
