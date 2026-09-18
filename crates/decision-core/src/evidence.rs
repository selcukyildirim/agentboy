use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceSource {
    Document,
    Spreadsheet,
    Database,
    UserInput,
    API,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceReliability {
    High,
    Medium,
    Low,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub claim: String,
    pub source: EvidenceSource,
    pub source_name: String,
    pub value: serde_json::Value,
    pub reliability: EvidenceReliability,
    pub timestamp: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Evidence {
    pub fn new(
        claim: &str,
        source: EvidenceSource,
        source_name: &str,
        value: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            claim: claim.to_string(),
            source,
            source_name: source_name.to_string(),
            value,
            reliability: EvidenceReliability::Unverified,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_reliability(mut self, reliability: EvidenceReliability) -> Self {
        self.reliability = reliability;
        self
    }
}

pub struct EvidenceValidator;

impl EvidenceValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, evidence: &Evidence) -> ValidationResult {
        let mut issues = Vec::new();

        if evidence.timestamp.is_empty() {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Missing timestamp".to_string(),
            });
        }

        if matches!(evidence.reliability, EvidenceReliability::Unverified) {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Evidence reliability not verified".to_string(),
            });
        }

        let is_valid = issues.iter().all(|i| i.severity != IssueSeverity::Critical);

        ValidationResult {
            evidence_id: evidence.id.clone(),
            is_valid,
            issues,
            verified_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn cross_validate(&self, evidences: &[Evidence]) -> CrossValidationResult {
        let claims: Vec<&str> = evidences.iter().map(|e| e.claim.as_str()).collect();
        let unique_claims: std::collections::HashSet<&str> = claims.iter().copied().collect();

        let consistency_score = if unique_claims.len() <= 1 {
            1.0
        } else {
            1.0 - (unique_claims.len() as f32 / claims.len() as f32)
        };

        let avg_reliability = self.average_reliability(evidences);

        CrossValidationResult {
            evidence_count: evidences.len(),
            unique_claims: unique_claims.len(),
            consistency_score,
            avg_reliability,
            is_consistent: consistency_score > 0.5,
        }
    }

    fn average_reliability(&self, evidences: &[Evidence]) -> f32 {
        if evidences.is_empty() {
            return 0.0;
        }
        let total: u32 = evidences
            .iter()
            .map(|e| match e.reliability {
                EvidenceReliability::High => 4,
                EvidenceReliability::Medium => 3,
                EvidenceReliability::Low => 2,
                EvidenceReliability::Unverified => 1,
            })
            .sum();
        total as f32 / evidences.len() as f32 / 4.0
    }
}

impl Default for EvidenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub evidence_id: String,
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResult {
    pub evidence_count: usize,
    pub unique_claims: usize,
    pub consistency_score: f32,
    pub avg_reliability: f32,
    pub is_consistent: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_creation() {
        let evidence = Evidence::new(
            "Price is $100",
            EvidenceSource::Document,
            "quote.pdf",
            serde_json::json!({"price": 100}),
        );
        assert_eq!(evidence.claim, "Price is $100");
        assert!(matches!(evidence.source, EvidenceSource::Document));
    }

    #[test]
    fn test_validate_evidence() {
        let validator = EvidenceValidator::new();
        let evidence = Evidence::new(
            "Test",
            EvidenceSource::UserInput,
            "user",
            serde_json::json!({}),
        );
        let result = validator.validate(&evidence);
        assert!(result.is_valid);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_cross_validate() {
        let validator = EvidenceValidator::new();
        let e1 = Evidence::new(
            "Price is $100",
            EvidenceSource::Document,
            "doc1",
            serde_json::json!({}),
        );
        let e2 = Evidence::new(
            "Price is $100",
            EvidenceSource::API,
            "api1",
            serde_json::json!({}),
        );
        let result = validator.cross_validate(&[e1, e2]);
        assert!(result.is_consistent);
    }
}
