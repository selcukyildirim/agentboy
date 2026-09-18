use crate::tool::Tool;
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let id = tool.manifest().id.to_string();
        self.tools.insert(id, tool);
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn Tool> {
        self.tools.get(id).map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(std::string::String::as_str).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
