use agent_common::types::ToolRisk;

#[derive(Debug, Clone)]
pub struct ToolRiskClassifier;

impl ToolRiskClassifier {
    pub fn new() -> Self {
        Self
    }

    pub fn classify(tool_id: &str) -> ToolRisk {
        match tool_id {
            id if id.starts_with("filesystem.read") => ToolRisk::Read,
            id if id.starts_with("filesystem.write") => ToolRisk::Write,
            id if id.starts_with("filesystem.delete") => ToolRisk::Destructive,
            id if id.starts_with("csv.") => ToolRisk::Read,
            id if id.starts_with("spreadsheet.") => ToolRisk::Read,
            id if id.starts_with("json.") => ToolRisk::Read,
            id if id.starts_with("pdf.") => ToolRisk::Read,
            id if id.starts_with("docx.") => ToolRisk::Read,
            id if id.starts_with("erp.") => ToolRisk::Financial,
            id if id.starts_with("network.") => ToolRisk::External,
            id if id.starts_with("admin.") => ToolRisk::Admin,
            _ => ToolRisk::Read,
        }
    }

    pub fn requires_approval(risk: &ToolRisk) -> bool {
        matches!(risk, ToolRisk::Destructive | ToolRisk::Financial | ToolRisk::Admin)
    }

    pub fn is_allowed_in_free_tier(risk: &ToolRisk) -> bool {
        !matches!(risk, ToolRisk::Admin)
    }
}

impl Default for ToolRiskClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_read() {
        assert!(matches!(
            ToolRiskClassifier::classify("filesystem.read"),
            ToolRisk::Read
        ));
    }

    #[test]
    fn test_classify_write() {
        assert!(matches!(
            ToolRiskClassifier::classify("filesystem.write"),
            ToolRisk::Write
        ));
    }

    #[test]
    fn test_requires_approval() {
        assert!(ToolRiskClassifier::requires_approval(&ToolRisk::Destructive));
        assert!(ToolRiskClassifier::requires_approval(&ToolRisk::Financial));
        assert!(!ToolRiskClassifier::requires_approval(&ToolRisk::Read));
    }

    #[test]
    fn test_free_tier() {
        assert!(ToolRiskClassifier::is_allowed_in_free_tier(&ToolRisk::Read));
        assert!(!ToolRiskClassifier::is_allowed_in_free_tier(&ToolRisk::Admin));
    }
}