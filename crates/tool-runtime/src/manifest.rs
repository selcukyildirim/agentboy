use serde::{Deserialize, Serialize};
use agent_common::types::ToolRisk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub id: &'static str,
    pub version: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub risk: ToolRisk,
    pub input_schema: Option<&'static str>,
    pub output_schema: Option<&'static str>,
}