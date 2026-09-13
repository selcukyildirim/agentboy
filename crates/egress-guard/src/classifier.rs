use agent_common::types::DataClassification;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct DataClassifier {
    pii_patterns: Vec<Regex>,
    secret_patterns: Vec<Regex>,
}

impl DataClassifier {
    pub fn new() -> Self {
        let pii_patterns = vec![
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
            Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
            Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
            Regex::new(r"\b\d{16,19}\b").unwrap(),
        ];

        let secret_patterns = vec![
            Regex::new(r"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|password)\s*[:=]\s*\S+").unwrap(),
            Regex::new(r"(?i)(sk-[a-zA-Z0-9]{20,})").unwrap(),
            Regex::new(r"(?i)(ghp_[a-zA-Z0-9]{36})").unwrap(),
            Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap(),
        ];

        Self {
            pii_patterns,
            secret_patterns,
        }
    }

    pub fn classify(&self, content: &str, has_secrets: bool, has_pii: bool) -> DataClassification {
        if has_secrets || self.detect_secrets(content).is_some() {
            return DataClassification::L0LocalOnly;
        }
        if has_pii || self.detect_pii(content).is_some() {
            return DataClassification::L2Sanitized;
        }
        if content.len() < 100 {
            DataClassification::L1MetadataOnly
        } else {
            DataClassification::L3MinimumRequired
        }
    }

    pub fn detect_pii(&self, content: &str) -> Option<Vec<String>> {
        let mut found = Vec::new();
        for pattern in &self.pii_patterns {
            for mat in pattern.find_iter(content) {
                found.push(mat.as_str().to_string());
            }
        }
        if found.is_empty() { None } else { Some(found) }
    }

    pub fn detect_secrets(&self, content: &str) -> Option<Vec<String>> {
        let mut found = Vec::new();
        for pattern in &self.secret_patterns {
            for mat in pattern.find_iter(content) {
                found.push(mat.as_str().to_string());
            }
        }
        if found.is_empty() { None } else { Some(found) }
    }

    pub fn sanitize(&self, content: &str) -> String {
        let mut result = content.to_string();

        for pattern in &self.pii_patterns {
            result = pattern.replace_all(&result, "[REDACTED_PII]").to_string();
        }

        for pattern in &self.secret_patterns {
            result = pattern.replace_all(&result, "[REDACTED_SECRET]").to_string();
        }

        result
    }
}

impl Default for DataClassifier {
    fn default() -> Self {
        Self::new()
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
        assert!(pii.unwrap().len() == 1);
    }

    #[test]
    fn test_detect_phone() {
        let classifier = DataClassifier::new();
        let pii = classifier.detect_pii("Call 555-123-4567");
        assert!(pii.is_some());
    }

    #[test]
    fn test_detect_api_key() {
        let classifier = DataClassifier::new();
        let secrets = classifier.detect_secrets("api_key=sk-abc123def456ghi789jkl0");
        assert!(secrets.is_some());
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
        let class = classifier.classify("secret data with sk-abc123def456ghi789jkl0", false, false);
        assert!(matches!(class, DataClassification::L0LocalOnly));
    }
}