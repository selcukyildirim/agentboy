use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashState {
    pub session_id: String,
    pub started_at: String,
    pub last_checkpoint: String,
    pub pending_executions: Vec<PendingExecution>,
    pub cached_results: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingExecution {
    pub execution_id: String,
    pub agent_id: String,
    pub input: serde_json::Value,
    pub current_step: u32,
    pub checkpoint_data: Option<serde_json::Value>,
}

pub struct CrashRecovery {
    state_dir: PathBuf,
}

impl CrashRecovery {
    pub fn new(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    pub fn save_state(&self, state: &CrashState) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.state_dir.join("crash_state.json");
        let json = serde_json::to_string_pretty(state)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_state(&self) -> Result<Option<CrashState>, Box<dyn std::error::Error>> {
        let path = self.state_dir.join("crash_state.json");
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(path)?;
        let state: CrashState = serde_json::from_str(&json)?;
        Ok(Some(state))
    }

    pub fn clear_state(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.state_dir.join("crash_state.json");
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn create_checkpoint(&self, execution_id: &str, step: u32, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.load_state()?.unwrap_or_else(|| CrashState {
            session_id: uuid::Uuid::new_v4().to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            pending_executions: Vec::new(),
            cached_results: HashMap::new(),
        });

        state.last_checkpoint = chrono::Utc::now().to_rfc3339();

        if let Some(exec) = state.pending_executions.iter_mut().find(|e| e.execution_id == execution_id) {
            exec.current_step = step;
            exec.checkpoint_data = Some(data);
        } else {
            state.pending_executions.push(PendingExecution {
                execution_id: execution_id.to_string(),
                agent_id: String::new(),
                input: serde_json::json!({}),
                current_step: step,
                checkpoint_data: Some(data),
            });
        }

        self.save_state(&state)?;
        Ok(())
    }

    pub fn get_resumable_executions(&self) -> Vec<PendingExecution> {
        self.load_state()
            .ok()
            .flatten()
            .map(|s| s.pending_executions)
            .unwrap_or_default()
    }
}

impl Default for CrashRecovery {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("agentboy_crash"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("test_crash_{}", name))
    }

    #[test]
    fn test_crash_recovery_save_load() {
        let dir = test_dir("save_load");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        let state = CrashState {
            session_id: "test".to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            pending_executions: vec![],
            cached_results: HashMap::new(),
        };

        recovery.save_state(&state).unwrap();
        let loaded = recovery.load_state().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().session_id, "test");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let dir = test_dir("nonexistent");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        let loaded = recovery.load_state().unwrap();
        assert!(loaded.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clear_state() {
        let dir = test_dir("clear");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        let state = CrashState {
            session_id: "clear_test".to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            pending_executions: vec![],
            cached_results: HashMap::new(),
        };

        recovery.save_state(&state).unwrap();
        assert!(recovery.load_state().unwrap().is_some());

        recovery.clear_state().unwrap();
        assert!(recovery.load_state().unwrap().is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_create_checkpoint() {
        let dir = test_dir("checkpoint");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        recovery.create_checkpoint("exec-1", 5, serde_json::json!({"step": 5})).unwrap();

        let loaded = recovery.load_state().unwrap().unwrap();
        assert_eq!(loaded.pending_executions.len(), 1);
        assert_eq!(loaded.pending_executions[0].execution_id, "exec-1");
        assert_eq!(loaded.pending_executions[0].current_step, 5);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_checkpoint_updates_existing_execution() {
        let dir = test_dir("checkpoint_update");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        recovery.create_checkpoint("exec-1", 3, serde_json::json!({})).unwrap();
        recovery.create_checkpoint("exec-1", 7, serde_json::json!({"updated": true})).unwrap();

        let loaded = recovery.load_state().unwrap().unwrap();
        assert_eq!(loaded.pending_executions.len(), 1);
        assert_eq!(loaded.pending_executions[0].current_step, 7);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_get_resumable_executions() {
        let dir = test_dir("resumable");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        let empty = recovery.get_resumable_executions();
        assert!(empty.is_empty());

        recovery.create_checkpoint("exec-1", 2, serde_json::json!({})).unwrap();
        recovery.create_checkpoint("exec-2", 5, serde_json::json!({})).unwrap();

        let resumable = recovery.get_resumable_executions();
        assert_eq!(resumable.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cached_results_preserved() {
        let dir = test_dir("cached");
        let _ = std::fs::create_dir_all(&dir);

        let recovery = CrashRecovery::new(dir.clone());
        let mut cached = HashMap::new();
        cached.insert("result-1".to_string(), serde_json::json!({"data": 42}));

        let state = CrashState {
            session_id: "cache_test".to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_checkpoint: chrono::Utc::now().to_rfc3339(),
            pending_executions: vec![],
            cached_results: cached,
        };

        recovery.save_state(&state).unwrap();
        let loaded = recovery.load_state().unwrap().unwrap();
        assert_eq!(loaded.cached_results.len(), 1);
        assert_eq!(loaded.cached_results["result-1"]["data"], 42);

        let _ = std::fs::remove_dir_all(&dir);
    }
}