use crate::workflow::{Workflow, WorkflowStep};
use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Executes a single workflow step. Implemented by the host (e.g. the desktop
/// runs agents, the skill registry runs skills).
#[async_trait]
pub trait StepExecutor: Send + Sync {
    async fn execute_step(
        &self,
        skill_id: &str,
        input: serde_json::Value,
    ) -> AppResult<serde_json::Value>;
}

/// Default executor used by `run`; returns a placeholder result.
struct StubExecutor;

#[async_trait]
impl StepExecutor for StubExecutor {
    async fn execute_step(
        &self,
        skill_id: &str,
        _input: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        Ok(serde_json::json!({ "skill": skill_id, "status": "stub" }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub workflow_version: u32,
    pub status: ExecutionStatus,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub step_results: Vec<StepResult>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub skill_id: String,
    pub status: StepStatus,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
    pub rollback_performed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    RolledBack,
}

pub struct WorkflowRunner;

impl WorkflowRunner {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(
        &self,
        workflow: &Workflow,
        input: serde_json::Value,
    ) -> AppResult<WorkflowExecution> {
        self.run_with(&StubExecutor, workflow, input).await
    }

    /// Run a workflow with a real step executor.
    pub async fn run_with(
        &self,
        executor: &dyn StepExecutor,
        workflow: &Workflow,
        input: serde_json::Value,
    ) -> AppResult<WorkflowExecution> {
        let params = self.extract_params(workflow, &input)?;
        let mut execution = self.create_execution(workflow, &input);

        let mut prev_outputs: HashMap<String, serde_json::Value> = HashMap::new();

        for step in &workflow.steps {
            let step_result = self
                .execute_step(executor, step, &params, &mut prev_outputs)
                .await;
            execution.step_results.push(step_result.clone());

            match step_result.status {
                StepStatus::Failed => {
                    execution.status = ExecutionStatus::Failed;
                    execution.error = step_result.error.clone();
                    self.perform_rollback(
                        executor,
                        workflow,
                        &execution.step_results,
                        &mut prev_outputs,
                    )
                    .await;
                    break;
                }
                StepStatus::Completed => {
                    if let Some(output) = &step_result.output {
                        prev_outputs.insert(step.id.clone(), output.clone());
                    }
                }
                _ => {}
            }
        }

        if execution.status == ExecutionStatus::Running {
            execution.status = ExecutionStatus::Completed;
            execution.output = prev_outputs.values().last().cloned();
        }

        execution.completed_at = Some(chrono::Utc::now().to_rfc3339());
        execution.duration_ms = self.calculate_duration(&execution);

        Ok(execution)
    }

    fn extract_params(
        &self,
        workflow: &Workflow,
        input: &serde_json::Value,
    ) -> AppResult<HashMap<String, serde_json::Value>> {
        let mut params = HashMap::new();

        for param in &workflow.parameters {
            if let Some(value) = input.get(&param.name) {
                params.insert(param.name.clone(), value.clone());
            } else if let Some(default) = &param.default {
                params.insert(param.name.clone(), default.clone());
            } else if param.required {
                return Err(AppError::Validation(format!(
                    "Missing required parameter: {}",
                    param.name
                )));
            }
        }

        Ok(params)
    }

    fn create_execution(
        &self,
        workflow: &Workflow,
        input: &serde_json::Value,
    ) -> WorkflowExecution {
        WorkflowExecution {
            id: uuid::Uuid::new_v4().to_string(),
            workflow_id: workflow.id.clone(),
            workflow_version: workflow.version,
            status: ExecutionStatus::Running,
            input: input.clone(),
            output: None,
            step_results: Vec::new(),
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            duration_ms: None,
            error: None,
        }
    }

    async fn execute_step(
        &self,
        executor: &dyn StepExecutor,
        step: &WorkflowStep,
        params: &HashMap<String, serde_json::Value>,
        prev_outputs: &HashMap<String, serde_json::Value>,
    ) -> StepResult {
        let mut result = StepResult {
            step_id: step.id.clone(),
            skill_id: step.skill_id.clone(),
            status: StepStatus::Running,
            input: serde_json::json!({}),
            output: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            error: None,
            rollback_performed: false,
        };

        let input = self.resolve_input(step, params, prev_outputs);
        result.input = input.clone();

        match executor.execute_step(&step.skill_id, input).await {
            Ok(output) => {
                result.status = StepStatus::Completed;
                result.output = Some(output);
            }
            Err(e) => {
                result.status = StepStatus::Failed;
                result.error = Some(e.to_string());
            }
        }

        result.completed_at = Some(chrono::Utc::now().to_rfc3339());
        result
    }

    fn resolve_input(
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

    async fn perform_rollback(
        &self,
        executor: &dyn StepExecutor,
        workflow: &Workflow,
        step_results: &[StepResult],
        prev_outputs: &mut HashMap<String, serde_json::Value>,
    ) {
        for step_result in step_results.iter().rev() {
            if step_result.status == StepStatus::Completed {
                if let Some(step) = workflow.steps.iter().find(|s| s.id == step_result.step_id) {
                    if let Some(ref rollback) = step.rollback {
                        tracing::info!(step_id = %step.id, "Performing rollback");
                        let input = self.resolve_input(
                            &WorkflowStep {
                                id: step.id.clone(),
                                skill_id: rollback.skill_id.clone(),
                                name: step.name.clone(),
                                input_mapping: rollback.input_mapping.clone(),
                                output_mapping: None,
                                depends_on: vec![],
                                timeout_seconds: None,
                                retry_count: None,
                                rollback: None,
                            },
                            &HashMap::new(),
                            prev_outputs,
                        );
                        let _ = executor.execute_step(&rollback.skill_id, input).await;
                    }
                }
            }
        }
    }

    fn calculate_duration(&self, execution: &WorkflowExecution) -> Option<u64> {
        let start = chrono::DateTime::parse_from_rfc3339(&execution.started_at).ok()?;
        let end = execution.completed_at.as_ref()?;
        let end_dt = chrono::DateTime::parse_from_rfc3339(end).ok()?;
        Some((end_dt - start).num_milliseconds() as u64)
    }
}

impl Default for WorkflowRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{ParameterType, Workflow, WorkflowParameter, WorkflowStep};

    #[tokio::test]
    async fn test_workflow_runner() {
        let mut wf = Workflow::new("wf1", "Test");
        wf.add_parameter(WorkflowParameter {
            name: "data".to_string(),
            param_type: ParameterType::String,
            default: None,
            required: true,
            description: "Input".to_string(),
        });
        wf.add_step(WorkflowStep {
            id: "step1".to_string(),
            skill_id: "test.skill".to_string(),
            name: "Step 1".to_string(),
            input_mapping: Some(serde_json::json!({"input": "$param.data"})),
            output_mapping: None,
            depends_on: vec![],
            timeout_seconds: None,
            retry_count: None,
            rollback: None,
        });

        let runner = WorkflowRunner::new();
        let input = serde_json::json!({"data": "hello"});
        let execution = runner.run(&wf, input).await.unwrap();

        assert_eq!(execution.status, ExecutionStatus::Completed);
        assert_eq!(execution.step_results.len(), 1);
    }

    #[tokio::test]
    async fn test_workflow_missing_param() {
        let mut wf = Workflow::new("wf1", "Test");
        wf.add_parameter(WorkflowParameter {
            name: "required_param".to_string(),
            param_type: ParameterType::String,
            default: None,
            required: true,
            description: "Required".to_string(),
        });

        let runner = WorkflowRunner::new();
        let input = serde_json::json!({});
        let result = runner.run(&wf, input).await;

        assert!(result.is_err());
    }

    struct RecordingExecutor(std::sync::Mutex<Vec<String>>);

    #[async_trait]
    impl StepExecutor for RecordingExecutor {
        async fn execute_step(
            &self,
            skill_id: &str,
            _input: serde_json::Value,
        ) -> AppResult<serde_json::Value> {
            agent_common::sync::lock(&self.0).push(skill_id.to_string());
            Ok(serde_json::json!({ "ok": skill_id }))
        }
    }

    #[tokio::test]
    async fn test_run_with_custom_executor() {
        let mut wf = Workflow::new("wf2", "Custom");
        wf.add_step(WorkflowStep {
            id: "s1".into(),
            skill_id: "paid.margin-guardian".into(),
            name: "Margin".into(),
            input_mapping: None,
            output_mapping: None,
            depends_on: vec![],
            timeout_seconds: None,
            retry_count: None,
            rollback: None,
        });
        let executor = RecordingExecutor(std::sync::Mutex::new(Vec::new()));
        let runner = WorkflowRunner::new();
        let execution = runner
            .run_with(&executor, &wf, serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(execution.status, ExecutionStatus::Completed);
        assert_eq!(
            agent_common::sync::lock(&executor.0)[0],
            "paid.margin-guardian"
        );
    }
}
