use audit_core::event::AuditEvent;
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub event_id: String,
    pub execution_id: Option<String>,
    pub agent_id: String,
    pub action: String,
    pub resource: String,
    pub result: String,
    pub timestamp: String,
    pub details: Option<serde_json::Value>,
}

fn to_entry(e: AuditEvent) -> AuditEntry {
    AuditEntry {
        event_id: e.event_id.to_string(),
        execution_id: e.execution_id.map(|u| u.to_string()),
        agent_id: e.agent_id,
        action: e.action,
        resource: e.resource,
        result: format!("{:?}", e.result),
        timestamp: e.timestamp.to_rfc3339(),
        details: e.details,
    }
}

#[tauri::command]
pub async fn list_audit(
    state: tauri::State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AuditEntry>, String> {
    state
        .audit
        .query_all(limit.unwrap_or(100))
        .await
        .map(|events| events.into_iter().map(to_entry).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_audit_for_agent(
    state: tauri::State<'_, AppState>,
    agent_id: String,
    limit: Option<usize>,
) -> Result<Vec<AuditEntry>, String> {
    state
        .audit
        .query(&agent_id, limit.unwrap_or(50))
        .await
        .map(|events| events.into_iter().map(to_entry).collect())
        .map_err(|e| e.to_string())
}
