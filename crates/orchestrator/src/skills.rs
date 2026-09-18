use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use agent_common::error::AppResult;
use async_trait::async_trait;
use skill_sdk::manifest::{SkillManifest, SkillTier};
use skill_sdk::registry::SkillRegistry;
use skill_sdk::skill::Skill;
use tool_runtime::registry::ToolRegistry;
use tool_runtime::tools::csv_engine::CsvEngine;
use tool_runtime::tools::filesystem::{FilesystemReadTool, FilesystemWriteTool};
use tool_runtime::tools::json_transform::JsonTransform;
use tool_runtime::tools::pdf_docx::{DocxParser, PdfParser};
use tool_runtime::tools::xlsx_engine::XlsxEngine;

type SkillFn = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = AppResult<serde_json::Value>> + Send>>
        + Send
        + Sync,
>;

/// A skill implemented by an async function.
pub struct FnSkill {
    manifest: SkillManifest,
    f: SkillFn,
}

impl FnSkill {
    pub fn new<F>(manifest: SkillManifest, f: F) -> Self
    where
        F: Fn(
                serde_json::Value,
            ) -> Pin<Box<dyn Future<Output = AppResult<serde_json::Value>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            manifest,
            f: Arc::new(f),
        }
    }
}

#[async_trait]
impl Skill for FnSkill {
    fn manifest(&self) -> &SkillManifest {
        &self.manifest
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        (self.f)(input).await
    }
}

/// A skill that delegates to a tool-runtime tool with a fixed operation.
pub struct ToolSkill {
    manifest: SkillManifest,
    registry: Arc<ToolRegistry>,
    tool_id: &'static str,
    operation: &'static str,
}

impl ToolSkill {
    #[must_use]
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        registry: Arc<ToolRegistry>,
        tool_id: &'static str,
        operation: &'static str,
    ) -> Self {
        Self {
            manifest: manifest(id, name, description),
            registry,
            tool_id,
            operation,
        }
    }
}

#[async_trait]
impl Skill for ToolSkill {
    fn manifest(&self) -> &SkillManifest {
        &self.manifest
    }

    async fn execute(&self, mut input: serde_json::Value) -> AppResult<serde_json::Value> {
        if let Some(obj) = input.as_object_mut() {
            obj.insert(
                "operation".to_string(),
                serde_json::Value::String(self.operation.to_string()),
            );
        }
        let tool = self.registry.get(self.tool_id).ok_or_else(|| {
            agent_common::error::AppError::NotFound(format!("Tool {}", self.tool_id))
        })?;
        tool.execute(input).await
    }
}

fn manifest(id: &str, name: &str, description: &str) -> SkillManifest {
    SkillManifest {
        id: id.to_string(),
        version: "2.0.0".to_string(),
        name: name.to_string(),
        description: description.to_string(),
        tier: SkillTier::Free,
        tools_required: vec![],
        input_schema: None,
        output_schema: None,
    }
}

/// Build the tool registry with all available tools.
#[must_use]
pub fn build_tool_registry(allowed_dirs: Vec<std::path::PathBuf>) -> ToolRegistry {
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(CsvEngine::new()));
    tools.register(Box::new(XlsxEngine::new()));
    tools.register(Box::new(JsonTransform::new()));
    tools.register(Box::new(PdfParser::new()));
    tools.register(Box::new(DocxParser::new()));
    tools.register(Box::new(FilesystemReadTool::new(allowed_dirs.clone())));
    tools.register(Box::new(FilesystemWriteTool::new(allowed_dirs)));
    tools
}

/// Build the registry of concrete skills available to agents.
#[must_use]
pub fn build_registry() -> SkillRegistry {
    build_registry_with_tools(Arc::new(build_tool_registry(vec![])))
}

/// Build the skill registry backed by a specific tool registry.
#[must_use]
pub fn build_registry_with_tools(tools: Arc<ToolRegistry>) -> SkillRegistry {
    let mut registry = SkillRegistry::new();

    // spreadsheet.* → tool-runtime
    registry.register(Box::new(ToolSkill::new(
        "spreadsheet.parse",
        "Spreadsheet Parse",
        "Parse CSV/XLSX content into structured rows",
        tools.clone(),
        "csv.parse",
        "parse",
    )));
    registry.register(Box::new(ToolSkill::new(
        "spreadsheet.analyze",
        "Spreadsheet Analyze",
        "Summarize spreadsheet columns",
        tools,
        "spreadsheet.process",
        "summarize",
    )));
    registry.register(Box::new(FnSkill::new(
        manifest(
            "spreadsheet.compare",
            "Spreadsheet Compare",
            "Merge two spreadsheets for comparison",
        ),
        |input| {
            Box::pin(async move {
                let a = input["a"].as_str().unwrap_or("");
                let b = input["b"].as_str().unwrap_or("");
                let engine = XlsxEngine::new();
                let da = engine.parse_csv(a)?;
                let db = engine.parse_csv(b)?;
                let merged = engine.merge(&da, &db)?;
                serde_json::to_value(merged)
                    .map_err(|e| agent_common::error::AppError::Validation(e.to_string()))
            })
        },
    )));

    // document.* → document-parser
    registry.register(Box::new(FnSkill::new(
        manifest(
            "document.parse",
            "Document Parse",
            "Parse a document into sections/tables",
        ),
        |input| {
            Box::pin(async move {
                use document_parser::parser::{extract_text, AutoParser, DocumentParser};
                let content = input["content"].as_str().ok_or_else(|| {
                    agent_common::error::AppError::Validation("Missing 'content'".into())
                })?;
                let filename = input["filename"].as_str().unwrap_or("document.txt");
                let doc = AutoParser::new()
                    .parse(content.as_bytes(), filename)
                    .await?;
                Ok(serde_json::json!({
                    "doc_type": format!("{:?}", doc.doc_type),
                    "word_count": doc.metadata.word_count,
                    "section_count": doc.sections.len(),
                    "table_count": doc.tables.len(),
                    "text": extract_text(&doc),
                }))
            })
        },
    )));
    registry.register(Box::new(FnSkill::new(
        manifest(
            "document.extract",
            "Document Extract",
            "Extract plain text from a document",
        ),
        |input| {
            Box::pin(async move {
                use document_parser::parser::{extract_text, AutoParser, DocumentParser};
                let content = input["content"].as_str().ok_or_else(|| {
                    agent_common::error::AppError::Validation("Missing 'content'".into())
                })?;
                let filename = input["filename"].as_str().unwrap_or("document.txt");
                let doc = AutoParser::new()
                    .parse(content.as_bytes(), filename)
                    .await?;
                Ok(serde_json::json!({ "text": extract_text(&doc) }))
            })
        },
    )));
    registry.register(Box::new(FnSkill::new(
        manifest("document.analyze", "Document Analyze", "Produce deterministic document stats"),
        |input| {
            Box::pin(async move {
                let text = input["text"].as_str().unwrap_or("");
                let words = text.split_whitespace().count();
                let lines = text.lines().count();
                Ok(serde_json::json!({ "words": words, "lines": lines, "chars": text.chars().count() }))
            })
        },
    )));
    registry.register(Box::new(FnSkill::new(
        manifest("document.validate", "Document Validate", "Validate required document fields"),
        |input| {
            Box::pin(async move {
                let text = input["text"].as_str().unwrap_or("");
                let min_words = input["min_words"].as_u64().unwrap_or(1);
                let words = text.split_whitespace().count() as u64;
                let valid = words >= min_words;
                Ok(serde_json::json!({
                    "valid": valid,
                    "word_count": words,
                    "issues": if valid { Vec::<String>::new() } else { vec![format!("document has {words} words, minimum is {min_words}")] },
                }))
            })
        },
    )));

    // decision.analyze → deterministic scaffold (upgraded in L4)
    registry.register(Box::new(FnSkill::new(
        manifest(
            "decision.analyze",
            "Decision Analyze",
            "Score options against weighted criteria",
        ),
        |input| {
            Box::pin(async move {
                let options = input["options"].as_array().cloned().unwrap_or_default();
                let scored: Vec<serde_json::Value> = options
                    .iter()
                    .map(|o| {
                        let score = o["score"].as_f64().unwrap_or(0.0);
                        serde_json::json!({ "name": o["name"], "score": score })
                    })
                    .collect();
                Ok(serde_json::json!({ "scored": scored }))
            })
        },
    )));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use skill_sdk::registry::KNOWN_SKILLS;

    #[test]
    fn test_all_known_non_llm_skills_registered() {
        let registry = build_registry();
        for id in KNOWN_SKILLS {
            if *id == "llm.analysis" {
                continue;
            }
            assert!(registry.contains(id), "skill {id} not registered");
        }
    }

    #[test]
    fn test_validate_all_agent_skills() {
        let registry = build_registry();
        let ids: Vec<String> = KNOWN_SKILLS
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        assert!(registry.validate(&ids).is_ok());
    }

    #[tokio::test]
    async fn test_spreadsheet_parse_skill() {
        let registry = build_registry();
        let skill = registry.get("spreadsheet.parse").unwrap();
        let out = skill
            .execute(serde_json::json!({ "content": "a,b\n1,2\n3,4" }))
            .await
            .unwrap();
        assert_eq!(out["count"], 3);
    }

    #[tokio::test]
    async fn test_document_parse_skill() {
        let registry = build_registry();
        let skill = registry.get("document.parse").unwrap();
        let out = skill
            .execute(serde_json::json!({ "content": "# Title\n\nHello world content", "filename": "doc.md" }))
            .await
            .unwrap();
        assert!(out["text"].as_str().unwrap().contains("Hello"));
    }

    #[tokio::test]
    async fn test_document_validate_skill() {
        let registry = build_registry();
        let skill = registry.get("document.validate").unwrap();
        let out = skill
            .execute(serde_json::json!({ "text": "one two three", "min_words": 5 }))
            .await
            .unwrap();
        assert_eq!(out["valid"], false);
    }
}
