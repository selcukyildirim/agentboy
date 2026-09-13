use crate::skill::Skill;
use std::collections::HashMap;

pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        let id = skill.manifest().id.clone();
        self.skills.insert(id, skill);
    }

    pub fn get(&self, id: &str) -> Option<&dyn Skill> {
        self.skills.get(id).map(|s| s.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
