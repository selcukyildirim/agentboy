use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub department: String,
    pub description: String,
    pub tier: AgentTier,
    pub skills: Vec<String>,
    pub permissions: AgentPermissions,
    pub execution: ExecutionLimits,
    #[serde(default)]
    pub rag_enabled: bool,
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    #[serde(default)]
    pub max_cost_usd: Option<f64>,
    /// Declarative description of the inputs this agent expects. Used by the
    /// desktop UI to build an input form.
    #[serde(default)]
    pub input_schema: Vec<InputField>,
}

/// A single input the agent accepts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputField {
    /// JSON key the agent reads (e.g. "`bank_statement`").
    pub key: String,
    /// Human-readable label for the UI.
    pub label: String,
    /// Kind of input control to render.
    pub kind: InputKind,
    pub required: bool,
    /// Optional example value shown as placeholder / "load example".
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl InputField {
    #[must_use]
    pub fn new(key: &str, label: &str, kind: InputKind, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            kind,
            required,
            example: None,
            description: None,
        }
    }

    #[must_use]
    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }

    #[must_use]
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    /// CSV/Excel/text file whose contents are passed as a string.
    File,
    /// Free CSV text pasted by the user.
    Csv,
    /// Numeric input.
    Number,
    /// Plain text / natural-language input.
    Text,
    /// Structured JSON (array or object) input.
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentTier {
    Free,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub filesystem_read: bool,
    pub filesystem_write: bool,
    pub network_llm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    pub max_steps: u32,
    pub timeout_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_field_builder() {
        let field = InputField::new("bank_statement", "Bank Statement", InputKind::File, true)
            .with_example("date,amount\n2024-01-01,100")
            .with_description("CSV export from the bank");

        assert_eq!(field.key, "bank_statement");
        assert_eq!(field.kind, InputKind::File);
        assert!(field.required);
        assert!(field.example.is_some());
        assert!(field.description.is_some());
    }

    #[test]
    fn test_input_kind_serializes_snake_case() {
        let json = serde_json::to_value(InputKind::File).unwrap();
        assert_eq!(json, "file");
        let json = serde_json::to_value(InputKind::Number).unwrap();
        assert_eq!(json, "number");
    }

    #[test]
    fn test_manifest_defaults_input_schema_empty() {
        let manifest = AgentManifest {
            id: "x".into(),
            version: "1.0.0".into(),
            name: "X".into(),
            department: "Test".into(),
            description: "d".into(),
            tier: AgentTier::Free,
            skills: vec![],
            permissions: AgentPermissions {
                filesystem_read: false,
                filesystem_write: false,
                network_llm: false,
            },
            execution: ExecutionLimits {
                max_steps: 1,
                timeout_seconds: 1,
            },
            rag_enabled: false,
            output_schema: None,
            max_cost_usd: None,
            input_schema: vec![],
        };
        let json = serde_json::to_value(&manifest).unwrap();
        assert_eq!(json["input_schema"], serde_json::json!([]));
    }
}
