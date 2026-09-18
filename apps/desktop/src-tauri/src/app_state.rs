use agent_runtime::registry::AgentRegistry;
use audit_core::store::SqliteAuditStore;
use orchestrator::Orchestrator;
use skill_sdk::SkillRegistry;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub registry: Arc<RwLock<AgentRegistry>>,
    pub orchestrator: Arc<Orchestrator>,
    pub skills: Arc<SkillRegistry>,
    pub pool: SqlitePool,
    pub audit: Arc<SqliteAuditStore>,
}

impl AppState {
    pub async fn new() -> Self {
        let mut registry = AgentRegistry::new();
        agents::register_all(&mut registry);
        tracing::info!(count = registry.list().len(), "Registered agents");

        let pool = crate::db::init()
            .await
            .expect("failed to initialize local database");

        crate::resilience::init(pool.clone());

        let audit = SqliteAuditStore::new(&crate::db::db_url())
            .await
            .expect("failed to initialize audit store");

        Self {
            registry: Arc::new(RwLock::new(registry)),
            orchestrator: Arc::new(Orchestrator::new()),
            skills: Arc::new(orchestrator::build_registry()),
            pool,
            audit: Arc::new(audit),
        }
    }
}
