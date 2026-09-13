use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub content_type: ContentType,
    pub metadata: DocumentMetadata,
    pub sections: Vec<Section>,
    pub raw_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Pdf,
    Docx,
    Xlsx,
    Csv,
    Txt,
    Markdown,
    Json,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_path: Option<String>,
    pub created_at: String,
    pub modified_at: Option<String>,
    pub size_bytes: u64,
    pub page_count: Option<u32>,
    pub author: Option<String>,
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: Option<String>,
    pub level: u32,
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub page: Option<u32>,
}

impl Document {
    pub fn new(id: &str, name: &str, content_type: ContentType, raw_content: String) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            content_type,
            metadata: DocumentMetadata {
                source_path: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: None,
                size_bytes: raw_content.len() as u64,
                page_count: None,
                author: None,
                custom: HashMap::new(),
            },
            sections: Vec::new(),
            raw_content,
        }
    }

    pub fn plain_text(id: &str, name: &str, content: &str) -> Self {
        let mut doc = Self::new(id, name, ContentType::Txt, content.to_string());
        doc.sections.push(Section {
            id: format!("{}-s0", id),
            title: None,
            level: 0,
            content: content.to_string(),
            start_offset: 0,
            end_offset: content.len(),
            page: None,
        });
        doc
    }
}

pub fn detect_content_type(filename: &str) -> ContentType {
    match filename.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "pdf" => ContentType::Pdf,
        "docx" | "doc" => ContentType::Docx,
        "xlsx" | "xls" => ContentType::Xlsx,
        "csv" => ContentType::Csv,
        "txt" => ContentType::Txt,
        "md" | "markdown" => ContentType::Markdown,
        "json" => ContentType::Json,
        _ => ContentType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_content_type() {
        assert!(matches!(detect_content_type("file.pdf"), ContentType::Pdf));
        assert!(matches!(detect_content_type("file.docx"), ContentType::Docx));
        assert!(matches!(detect_content_type("file.xlsx"), ContentType::Xlsx));
        assert!(matches!(detect_content_type("file.csv"), ContentType::Csv));
        assert!(matches!(detect_content_type("file.txt"), ContentType::Txt));
        assert!(matches!(detect_content_type("file.unknown"), ContentType::Unknown));
    }

    #[test]
    fn test_document_plain_text() {
        let doc = Document::plain_text("doc1", "test.txt", "Hello world");
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.raw_content, "Hello world");
    }
}