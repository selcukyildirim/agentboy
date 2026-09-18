use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub steps: Vec<WorkflowStep>,
    pub trigger: Trigger,
    pub parameters: Vec<WorkflowParameter>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub input_mapping: Option<serde_json::Value>,
    pub output_mapping: Option<serde_json::Value>,
    pub depends_on: Vec<String>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: Option<u32>,
    pub rollback: Option<RollbackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    pub skill_id: String,
    pub input_mapping: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub default: Option<serde_json::Value>,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    File,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub workflow_id: String,
    pub version: u32,
    pub workflow: Workflow,
    pub created_at: String,
    pub changelog: String,
}

impl Workflow {
    #[must_use]
    pub fn new(id: &str, name: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            version: 1,
            steps: Vec::new(),
            trigger: Trigger::Manual,
            parameters: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
            tags: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn add_parameter(&mut self, param: WorkflowParameter) {
        self.parameters.push(param);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn validate_parameters(
        &self,
        provided: &HashMap<String, serde_json::Value>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for param in &self.parameters {
            if param.required && !provided.contains_key(&param.name) {
                errors.push(format!("Missing required parameter: {}", param.name));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    #[must_use]
    pub fn resolve_input(
        &self,
        step: &WorkflowStep,
        params: &HashMap<String, serde_json::Value>,
        prev_outputs: &HashMap<String, serde_json::Value>,
    ) -> serde_json::Value {
        if let Some(ref mapping) = step.input_mapping {
            let mut resolved = mapping.clone();
            if let Some(obj) = resolved.as_object_mut() {
                for (_key, value) in obj.iter_mut() {
                    if let Some(param_name) = value.as_str().and_then(|s| s.strip_prefix("$param."))
                    {
                        if let Some(param_val) = params.get(param_name) {
                            *value = param_val.clone();
                        }
                    } else if let Some(ref_name) =
                        value.as_str().and_then(|s| s.strip_prefix("$prev."))
                    {
                        if let Some(prev_val) = prev_outputs.get(ref_name) {
                            *value = prev_val.clone();
                        }
                    }
                }
            }
            resolved
        } else {
            serde_json::json!({})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let mut wf = Workflow::new("wf1", "Test Workflow");
        wf.add_step(WorkflowStep {
            id: "step1".to_string(),
            skill_id: "skill.test".to_string(),
            name: "Test Step".to_string(),
            input_mapping: Some(serde_json::json!({"input": "$param.data"})),
            output_mapping: None,
            depends_on: vec![],
            timeout_seconds: None,
            retry_count: None,
            rollback: None,
        });

        assert_eq!(wf.steps.len(), 1);
        assert_eq!(wf.version, 1);
    }

    #[test]
    fn test_parameter_validation() {
        let mut wf = Workflow::new("wf1", "Test");
        wf.add_parameter(WorkflowParameter {
            name: "data".to_string(),
            param_type: ParameterType::String,
            default: None,
            required: true,
            description: "Input data".to_string(),
        });

        let mut params = std::collections::HashMap::new();
        assert!(wf.validate_parameters(&params).is_err());

        params.insert("data".to_string(), serde_json::json!("test"));
        assert!(wf.validate_parameters(&params).is_ok());
    }
}
