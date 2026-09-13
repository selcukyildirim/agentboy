use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionState {
    Pending,
    Planning,
    ExecutingTool { tool_id: String },
    CallingLLM { provider: String },
    Validating,
    Completed,
    Failed { reason: String },
    Cancelled { reason: String },
}

impl ExecutionState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExecutionState::Completed
                | ExecutionState::Failed { .. }
                | ExecutionState::Cancelled { .. }
        )
    }

    pub fn can_transition_to(&self, next: &ExecutionState) -> bool {
        use ExecutionState::*;
        matches!(
            (self, next),
            (Pending, Planning)
                | (Planning, ExecutingTool { .. })
                | (Planning, CallingLLM { .. })
                | (Planning, Validating)
                | (ExecutingTool { .. }, Planning)
                | (ExecutingTool { .. }, CallingLLM { .. })
                | (ExecutingTool { .. }, Validating)
                | (CallingLLM { .. }, Planning)
                | (CallingLLM { .. }, ExecutingTool { .. })
                | (CallingLLM { .. }, Validating)
                | (Validating, Completed)
                | (Validating, Failed { .. })
                | (Pending, Failed { .. })
                | (Planning, Failed { .. })
                | (Pending, Cancelled { .. })
                | (Planning, Cancelled { .. })
                | (ExecutingTool { .. }, Cancelled { .. })
                | (CallingLLM { .. }, Cancelled { .. })
                | (Validating, Cancelled { .. })
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: u32,
    pub state: ExecutionState,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
}

pub struct StepGuard {
    max_steps: u32,
    current_step: u32,
}

impl StepGuard {
    pub fn new(max_steps: u32) -> Self {
        Self {
            max_steps,
            current_step: 0,
        }
    }

    pub fn can_proceed(&self) -> bool {
        self.current_step < self.max_steps
    }

    pub fn increment(&mut self) -> AppResult<u32> {
        if !self.can_proceed() {
            return Err(AppError::ExecutionLimitExceeded {
                limit: self.max_steps,
            });
        }
        self.current_step += 1;
        Ok(self.current_step)
    }

    pub fn current(&self) -> u32 {
        self.current_step
    }

    pub fn remaining(&self) -> u32 {
        self.max_steps.saturating_sub(self.current_step)
    }
}

use agent_common::error::{AppError, AppResult};

pub struct OutputValidator;

impl OutputValidator {
    pub fn validate(
        output: &serde_json::Value,
        schema: Option<&serde_json::Value>,
    ) -> AppResult<()> {
        match schema {
            Some(schema) => Self::validate_against_schema(output, schema),
            None => Ok(()),
        }
    }

    fn validate_against_schema(
        output: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> AppResult<()> {
        if let Some(expected_type) = schema.get("type").and_then(|v| v.as_str()) {
            let actual_type = match output {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };
            if actual_type != expected_type {
                return Err(AppError::Validation(format!(
                    "Expected type '{}', got '{}'",
                    expected_type, actual_type
                )));
            }
        }

        if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
            if let Some(obj) = output.as_object() {
                for field in required {
                    if let Some(field_name) = field.as_str() {
                        if !obj.contains_key(field_name) {
                            return Err(AppError::Validation(format!(
                                "Missing required field: {}",
                                field_name
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        assert!(ExecutionState::Pending.can_transition_to(&ExecutionState::Planning));
        assert!(ExecutionState::Planning.can_transition_to(&ExecutionState::ExecutingTool { tool_id: "test".to_string() }));
        assert!(!ExecutionState::Pending.can_transition_to(&ExecutionState::Completed));
    }

    #[test]
    fn test_step_guard() {
        let mut guard = StepGuard::new(3);
        assert!(guard.can_proceed());
        assert_eq!(guard.increment().unwrap(), 1);
        assert_eq!(guard.increment().unwrap(), 2);
        assert_eq!(guard.increment().unwrap(), 3);
        assert!(!guard.can_proceed());
        assert!(guard.increment().is_err());
    }

    #[test]
    fn test_output_validator() {
        let output = serde_json::json!({"name": "test", "value": 42});
        let schema = serde_json::json!({"type": "object", "required": ["name", "value"]});
        assert!(OutputValidator::validate(&output, Some(&schema)).is_ok());

        let bad_output = serde_json::json!({"name": "test"});
        assert!(OutputValidator::validate(&bad_output, Some(&schema)).is_err());
    }
}