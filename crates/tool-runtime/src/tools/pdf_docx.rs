use async_trait::async_trait;
use agent_common::error::{AppError, AppResult};
use agent_common::types::ToolRisk;
use crate::tool::Tool;
use crate::manifest::ToolManifest;

pub struct PdfParser;

impl PdfParser {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_text(&self, _content: &[u8]) -> AppResult<String> {
        Err(AppError::ToolExecutionFailed {
            tool: "pdf.extract".to_string(),
            reason: "PDF parsing not yet implemented - requires pdf-extract crate".to_string(),
        })
    }

    pub fn extract_metadata(&self, _content: &[u8]) -> AppResult<serde_json::Value> {
        Err(AppError::ToolExecutionFailed {
            tool: "pdf.metadata".to_string(),
            reason: "PDF metadata extraction not yet implemented".to_string(),
        })
    }
}

impl Default for PdfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for PdfParser {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "pdf.parse",
            version: "1.0.0",
            name: "PDF Parser",
            description: "Extract text and metadata from PDF files",
            risk: ToolRisk::Read,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let operation = input["operation"].as_str().unwrap_or("extract_text");

        match operation {
            "extract_text" => {
                let _content = input["content"]
                    .as_str()
                    .ok_or_else(|| AppError::Validation("Missing 'content' field (base64)".to_string()))?;
                Err(AppError::ToolExecutionFailed {
                    tool: "pdf.extract".to_string(),
                    reason: "PDF parsing not yet implemented".to_string(),
                })
            }
            "extract_metadata" => {
                Err(AppError::ToolExecutionFailed {
                    tool: "pdf.metadata".to_string(),
                    reason: "PDF metadata not yet implemented".to_string(),
                })
            }
            _ => Err(AppError::Validation(format!("Unknown operation: {}", operation))),
        }
    }
}

pub struct DocxParser;

impl DocxParser {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_text(&self, _content: &[u8]) -> AppResult<String> {
        Err(AppError::ToolExecutionFailed {
            tool: "docx.extract".to_string(),
            reason: "DOCX parsing not yet implemented - requires docx crate".to_string(),
        })
    }
}

impl Default for DocxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DocxParser {
    fn manifest(&self) -> &ToolManifest {
        static MANIFEST: ToolManifest = ToolManifest {
            id: "docx.parse",
            version: "1.0.0",
            name: "DOCX Parser",
            description: "Extract text from Word documents",
            risk: ToolRisk::Read,
            input_schema: None,
            output_schema: None,
        };
        &MANIFEST
    }

    async fn execute(&self, input: serde_json::Value) -> AppResult<serde_json::Value> {
        let _content = input["content"]
            .as_str()
            .ok_or_else(|| AppError::Validation("Missing 'content' field (base64)".to_string()))?;

        Err(AppError::ToolExecutionFailed {
            tool: "docx.extract".to_string(),
            reason: "DOCX parsing not yet implemented".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_parser_manifest() {
        let parser = PdfParser::new();
        assert_eq!(parser.manifest().id, "pdf.parse");
        assert!(matches!(parser.manifest().risk, ToolRisk::Read));
    }

    #[test]
    fn test_docx_parser_manifest() {
        let parser = DocxParser::new();
        assert_eq!(parser.manifest().id, "docx.parse");
        assert!(matches!(parser.manifest().risk, ToolRisk::Read));
    }
}