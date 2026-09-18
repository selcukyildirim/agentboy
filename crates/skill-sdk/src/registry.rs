use crate::skill::Skill;
use std::collections::HashMap;

/// Skill ids that are always available without an explicit implementation.
/// `llm.analysis` is provided by the agent context itself.
pub const BUILTIN_SKILLS: &[&str] = &["llm.analysis"];

/// Known skill ids across the platform (implemented or builtin).
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
];

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        let id = skill.manifest().id.clone();
        self.skills.insert(id, skill);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn Skill> {
        self.skills.get(id).map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.skills
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.skills.contains_key(id)
    }

    #[must_use]
    pub fn is_builtin(id: &str) -> bool {
        BUILTIN_SKILLS.contains(&id)
    }

    /// A skill is resolvable if it is registered or is a builtin.
    #[must_use]
    pub fn is_resolvable(&self, id: &str) -> bool {
        self.contains(id) || Self::is_builtin(id)
    }

    /// Validate that every requested skill is known and resolvable.
    pub fn validate(&self, ids: &[String]) -> Result<(), Vec<String>> {
        let mut problems = Vec::new();
        for id in ids {
            if !KNOWN_SKILLS.contains(&id.as_str()) {
                problems.push(format!("unknown skill: {id}"));
            } else if !self.is_resolvable(id) {
                problems.push(format!("unimplemented skill: {id}"));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{SkillManifest, SkillTier};
    use agent_common::error::AppResult;
    use async_trait::async_trait;

    fn manifest_stub(id: &str) -> SkillManifest {
        SkillManifest {
            id: id.to_string(),
            version: "1.0.0".into(),
            name: id.into(),
            description: String::new(),
            tier: SkillTier::Free,
            tools_required: vec![],
            input_schema: None,
            output_schema: None,
        }
    }

    struct Named(&'static str);
    #[async_trait]
    impl Skill for Named {
        fn manifest(&self) -> &SkillManifest {
            Box::leak(Box::new(manifest_stub(self.0)))
        }
        async fn execute(&self, _i: serde_json::Value) -> AppResult<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
    }

    #[test]
    fn test_builtin_allowed() {
        let reg = SkillRegistry::new();
        assert!(SkillRegistry::is_builtin("llm.analysis"));
        assert!(reg.validate(&["llm.analysis".to_string()]).is_ok());
    }

    #[test]
    fn test_unknown_rejected() {
        let reg = SkillRegistry::new();
        let err = reg.validate(&["nope".to_string()]).unwrap_err();
        assert!(err[0].contains("unknown"));
    }

    #[test]
    fn test_unimplemented_rejected_until_registered() {
        let mut reg = SkillRegistry::new();
        assert!(reg.validate(&["spreadsheet.parse".to_string()]).is_err());
        reg.register(Box::new(Named("spreadsheet.parse")));
        assert!(reg.validate(&["spreadsheet.parse".to_string()]).is_ok());
    }

    #[test]
    fn test_contains_and_list() {
        let mut reg = SkillRegistry::new();
        reg.register(Box::new(Named("document.parse")));
        assert!(reg.contains("document.parse"));
        assert!(reg.list().contains(&"document.parse"));
    }
}
