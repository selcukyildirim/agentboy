-- Subsystem tables used by the runtime libraries and the desktop app.
-- Single source of truth for these schemas (crates rely on them at runtime).

-- Align audit_events with audit-core (column `created_at`).
DROP TABLE IF EXISTS audit_events;
CREATE TABLE audit_events (
    event_id TEXT PRIMARY KEY,
    execution_id TEXT,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    result TEXT NOT NULL,
    details JSON,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_audit_events_agent ON audit_events(agent_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);

-- Desktop agent executions (with steps and token/cost accounting).
CREATE TABLE IF NOT EXISTS agent_executions (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    status TEXT NOT NULL,
    input TEXT,
    output TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    duration_ms INTEGER,
    model TEXT,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cost_usd REAL,
    error TEXT,
    steps TEXT
);
CREATE INDEX IF NOT EXISTS idx_agent_executions_started ON agent_executions(started_at);
CREATE INDEX IF NOT EXISTS idx_agent_executions_agent ON agent_executions(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_executions_status ON agent_executions(status);

-- Local decision memory (decision-core).
CREATE TABLE IF NOT EXISTS decision_memory (
    id TEXT PRIMARY KEY,
    decision_type TEXT NOT NULL,
    context_json TEXT NOT NULL,
    recommendation_json TEXT NOT NULL,
    outcome_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_decision_memory_type ON decision_memory(decision_type);
CREATE INDEX IF NOT EXISTS idx_decision_memory_created ON decision_memory(created_at);

-- Workflow execution history (workflow-engine).
CREATE TABLE IF NOT EXISTS workflow_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    workflow_version INTEGER NOT NULL,
    status TEXT NOT NULL,
    data_json TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow ON workflow_executions(workflow_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_started ON workflow_executions(started_at);

CREATE TABLE IF NOT EXISTS workflow_versions (
    workflow_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    data_json TEXT NOT NULL,
    changelog TEXT,
    created_at TEXT NOT NULL,
    PRIMARY KEY (workflow_id, version)
);

-- Cache entries (cache-core): indexes for invalidation/cleanup.
CREATE INDEX IF NOT EXISTS idx_cache_entries_type ON cache_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_cache_entries_expires ON cache_entries(expires_at);
