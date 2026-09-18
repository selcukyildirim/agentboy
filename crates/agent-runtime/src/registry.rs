use crate::agent::Agent;
use std::collections::HashMap;

pub struct AgentRegistry {
    agents: HashMap<String, Box<dyn Agent>>,
}

impl AgentRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent: Box<dyn Agent>) {
        let id = agent.manifest().id;
        self.agents.insert(id, agent);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn Agent> {
        self.agents.get(id).map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
