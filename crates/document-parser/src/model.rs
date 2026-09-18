use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub document_id: String,
    pub filename: String,
    pub doc_type: DocumentType,
    pub sections: Vec<Section>,
    pub tables: Vec<Table>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentType {
    Pdf,
    Docx,
    Xlsx,
    Csv,
    Txt,
    Markdown,
    Json,
    Xml,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub heading: Option<String>,
    pub content: String,
    pub level: u32,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub page_count: Option<u32>,
    pub word_count: u32,
    pub created_at: Option<String>,
    pub author: Option<String>,
}
