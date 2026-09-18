use agent_runtime::registry::AgentRegistry;
use orchestrator::Orchestrator;
use skill_sdk::SkillRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub registry: Arc<RwLock<AgentRegistry>>,
    pub orchestrator: Arc<Orchestrator>,
    pub skills: Arc<SkillRegistry>,
}

impl AppState {
    pub fn new() -> Self {
        let mut registry = AgentRegistry::new();
        agents::register_all(&mut registry);

        tracing::info!(count = registry.list().len(), "Registered agents");

        Self {
            registry: Arc::new(RwLock::new(registry)),
            orchestrator: Arc::new(Orchestrator::new()),
            skills: Arc::new(orchestrator::build_registry()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
