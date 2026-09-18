use agent_runtime::registry::AgentRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub registry: Arc<RwLock<AgentRegistry>>,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        agents::register_all(&mut registry);

        tracing::info!(count = registry.list().len(), "Registered agents");

        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
