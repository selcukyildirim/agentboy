use agent_common::error::{AppError, AppResult};
use std::collections::HashSet;

pub const KNOWN_SKILLS: &[&str] = &[
    "spreadsheet.parse",
    "spreadsheet.analyze",
    "spreadsheet.compare",
    "llm.analysis",
    "document.parse",
    "document.extract",
    "document.analyze",
    "document.validate",
    "decision.analyze",
    "rag.query",
    "file.read",
    "file.write",
];

#[derive(Debug, Clone, Default)]
pub struct SkillCatalog {
    known: HashSet<String>,
}

impl SkillCatalog {
    pub fn new() -> Self {
        Self {
            known: KNOWN_SKILLS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn is_known(&self, skill: &str) -> bool {
        self.known.contains(skill)
    }

    pub fn validate(&self, skills: &[String]) -> AppResult<()> {
        let unknown: Vec<&String> = skills.iter().filter(|s| !self.is_known(s)).collect();
        if !unknown.is_empty() {
            return Err(AppError::Validation(format!(
                "Unknown skill(s): {}",
                unknown
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<&str> {
        self.known.iter().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_skill() {
        let catalog = SkillCatalog::new();
        assert!(catalog.is_known("spreadsheet.parse"));
        assert!(!catalog.is_known("nonexistent.skill"));
    }

    #[test]
    fn test_validate_ok() {
        let catalog = SkillCatalog::new();
        let skills = vec!["spreadsheet.parse".to_string(), "llm.analysis".to_string()];
        assert!(catalog.validate(&skills).is_ok());
    }

    #[test]
    fn test_validate_unknown() {
        let catalog = SkillCatalog::new();
        let skills = vec!["spreadsheet.pars".to_string()];
        assert!(catalog.validate(&skills).is_err());
    }

    #[test]
    fn test_validate_empty() {
        let catalog = SkillCatalog::new();
        assert!(catalog.validate(&[]).is_ok());
    }
}
