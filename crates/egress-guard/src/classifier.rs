use agent_common::types::DataClassification;
use regex::Regex;
use std::sync::OnceLock;

/// PII patterns are compiled once and shared for the process lifetime.
fn pii_patterns() -> &'static [Regex] {
    static P: OnceLock<Vec<Regex>> = OnceLock::new();
    P.get_or_init(|| {
        vec![
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap(),
            Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
            Regex::new(r"\b\d{16,19}\b").unwrap(),
        ]
    })
}

fn secret_patterns() -> &'static [Regex] {
    static S: OnceLock<Vec<Regex>> = OnceLock::new();
    S.get_or_init(|| {
        vec![
            Regex::new(r"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|password)\s*[:=]\s*\S+")
                .unwrap(),
            Regex::new(r"(?i)(sk-[a-zA-Z0-9]{20,})").unwrap(),
            Regex::new(r"(?i)(ghp_[a-zA-Z0-9]{36})").unwrap(),
            Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap(),
        ]
    })
}

/// Stateless classifier; pattern compilation is shared via statics.
#[derive(Debug, Clone, Default)]
pub struct DataClassifier;

impl DataClassifier {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Classify content in a single pass, detecting PII and secrets once.
    #[must_use]
    pub fn classify(&self, content: &str) -> DataClassification {
        if self.detect_secrets(content).is_some() {
            return DataClassification::L0LocalOnly;
        }
        if self.detect_pii(content).is_some() {
            return DataClassification::L2Sanitized;
        }
        if content.len() < 100 {
            DataClassification::L1MetadataOnly
        } else {
            DataClassification::L3MinimumRequired
        }
    }

    #[must_use]
    pub fn detect_pii(&self, content: &str) -> Option<Vec<String>> {
        let mut found = Vec::new();
        for pattern in pii_patterns() {
            for mat in pattern.find_iter(content) {
                found.push(mat.as_str().to_string());
            }
        }
        if found.is_empty() {
            None
        } else {
            Some(found)
        }
    }

    #[must_use]
    pub fn detect_secrets(&self, content: &str) -> Option<Vec<String>> {
        let mut found = Vec::new();
        for pattern in secret_patterns() {
            for mat in pattern.find_iter(content) {
                found.push(mat.as_str().to_string());
            }
        }
        if found.is_empty() {
            None
        } else {
            Some(found)
        }
    }

    #[must_use]
    pub fn sanitize(&self, content: &str) -> String {
        let mut result = content.to_string();
        for pattern in pii_patterns() {
            result = pattern.replace_all(&result, "[REDACTED_PII]").to_string();
        }
        for pattern in secret_patterns() {
            result = pattern
                .replace_all(&result, "[REDACTED_SECRET]")
                .to_string();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_email() {
        let classifier = DataClassifier::new();
        let pii = classifier.detect_pii("Contact me at user@example.com");
        assert!(pii.is_some());
        assert_eq!(pii.unwrap().len(), 1);
    }

    #[test]
    fn test_detect_phone() {
        let classifier = DataClassifier::new();
        assert!(classifier.detect_pii("Call 555-123-4567").is_some());
    }

    #[test]
    fn test_detect_api_key() {
        let classifier = DataClassifier::new();
        assert!(classifier
            .detect_secrets("api_key=sk-abc123def456ghi789jkl0")
            .is_some());
    }

    #[test]
    fn test_sanitize() {
        let classifier = DataClassifier::new();
        let sanitized = classifier.sanitize("Email user@example.com and call 555-123-4567");
        assert!(sanitized.contains("[REDACTED_PII]"));
        assert!(!sanitized.contains("user@example.com"));
    }

    #[test]
    fn test_classify_with_secrets() {
        let classifier = DataClassifier::new();
        let class = classifier.classify("secret data with sk-abc123def456ghi789jkl0");
        assert!(matches!(class, DataClassification::L0LocalOnly));
    }

    #[test]
    fn test_classify_pii() {
        let classifier = DataClassifier::new();
        let class = classifier.classify("contact user@example.com");
        assert!(matches!(class, DataClassification::L2Sanitized));
    }

    #[test]
    fn test_classify_metadata_vs_content() {
        let classifier = DataClassifier::new();
        assert!(matches!(
            classifier.classify("short"),
            DataClassification::L1MetadataOnly
        ));
        let long = "a".repeat(200);
        assert!(matches!(
            classifier.classify(&long),
            DataClassification::L3MinimumRequired
        ));
    }
}
