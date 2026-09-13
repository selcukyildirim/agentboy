use crate::model::{DocumentMetadata, DocumentType, ParsedDocument, Section, Table};
use agent_common::error::{AppError, AppResult};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DocumentParser: Send + Sync {
    fn supported_extensions(&self) -> Vec<&str>;
    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument>;
}

fn word_count(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

pub struct PlainTextParser;

#[async_trait]
impl DocumentParser for PlainTextParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["txt", "md", "markdown"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let text = String::from_utf8_lossy(content).to_string();
        let doc_type = if filename.ends_with(".md") || filename.ends_with(".markdown") {
            DocumentType::Markdown
        } else {
            DocumentType::Txt
        };

        let sections = parse_markdown_sections(&text);

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type,
            sections,
            tables: Vec::new(),
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

fn parse_markdown_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_level: u32 = 0;
    let mut current_content = String::new();

    for line in text.lines() {
        if line.starts_with('#') {
            if !current_content.trim().is_empty() || current_heading.is_some() {
                sections.push(Section {
                    heading: current_heading.clone(),
                    content: current_content.trim().to_string(),
                    level: current_level,
                    page: None,
                });
            }

            let level = line.chars().take_while(|&c| c == '#').count() as u32;
            let heading = line.trim_start_matches('#').trim().to_string();
            current_heading = Some(heading);
            current_level = level;
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_content.trim().is_empty() || current_heading.is_some() {
        sections.push(Section {
            heading: current_heading,
            content: current_content.trim().to_string(),
            level: current_level,
            page: None,
        });
    }

    sections
}

pub struct CsvParser;

#[async_trait]
impl DocumentParser for CsvParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["csv", "tsv"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let delimiter = if filename.ends_with(".tsv") { b'\t' } else { b',' };
        let spreadsheet =
            spreadsheet_engine::read_csv_with_delimiter(content, delimiter)?;
        let text = String::from_utf8_lossy(content).to_string();

        let tables: Vec<Table> = spreadsheet
            .sheets
            .iter()
            .map(|sheet| Table {
                headers: sheet.headers.clone(),
                rows: sheet.rows.clone(),
                page: None,
            })
            .collect();

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Csv,
            sections: Vec::new(),
            tables,
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct JsonParser;

#[async_trait]
impl DocumentParser for JsonParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["json"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let text = String::from_utf8_lossy(content).to_string();
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AppError::Validation(format!("JSON parse error: {}", e)))?;

        let formatted = serde_json::to_string_pretty(&value).unwrap_or_default();

        let sections = vec![Section {
            heading: Some("JSON Content".to_string()),
            content: formatted,
            level: 1,
            page: None,
        }];

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Json,
            sections,
            tables: Vec::new(),
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct XmlParser;

#[async_trait]
impl DocumentParser for XmlParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["xml"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let text = String::from_utf8_lossy(content).to_string();

        let sections = vec![Section {
            heading: Some("XML Content".to_string()),
            content: text.clone(),
            level: 1,
            page: None,
        }];

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Xml,
            sections,
            tables: Vec::new(),
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct XlsxParser;

#[async_trait]
impl DocumentParser for XlsxParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["xlsx", "xls"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let spreadsheet = spreadsheet_engine::read_xlsx(content)?;
        let text = String::from_utf8_lossy(content).to_string();

        let tables: Vec<Table> = spreadsheet
            .sheets
            .iter()
            .map(|sheet| Table {
                headers: sheet.headers.clone(),
                rows: sheet.rows.clone(),
                page: None,
            })
            .collect();

        let sheet_count = tables.len();
        let sections = vec![Section {
            heading: Some(format!("{} sheets", sheet_count)),
            content: format!("Spreadsheet with {} sheet(s)", sheet_count),
            level: 1,
            page: None,
        }];

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Xlsx,
            sections,
            tables,
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct PdfParser;

#[async_trait]
impl DocumentParser for PdfParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["pdf"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let text = String::from_utf8_lossy(content).to_string();

        let sections = vec![Section {
            heading: None,
            content: text.clone(),
            level: 1,
            page: None,
        }];

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Pdf,
            sections,
            tables: Vec::new(),
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct DocxParser;

#[async_trait]
impl DocumentParser for DocxParser {
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["docx"]
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let text = String::from_utf8_lossy(content).to_string();

        let sections = vec![Section {
            heading: None,
            content: text.clone(),
            level: 1,
            page: None,
        }];

        Ok(ParsedDocument {
            document_id: Uuid::new_v4().to_string(),
            filename: filename.to_string(),
            doc_type: DocumentType::Docx,
            sections,
            tables: Vec::new(),
            metadata: DocumentMetadata {
                page_count: None,
                word_count: word_count(&text),
                created_at: None,
                author: None,
            },
        })
    }
}

pub struct AutoParser {
    parsers: Vec<Box<dyn DocumentParser>>,
}

impl AutoParser {
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(PlainTextParser),
                Box::new(CsvParser),
                Box::new(JsonParser),
                Box::new(XmlParser),
                Box::new(XlsxParser),
                Box::new(PdfParser),
                Box::new(DocxParser),
            ],
        }
    }

    pub fn detect_type(filename: &str) -> DocumentType {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "pdf" => DocumentType::Pdf,
            "docx" => DocumentType::Docx,
            "xlsx" | "xls" => DocumentType::Xlsx,
            "csv" => DocumentType::Csv,
            "tsv" => DocumentType::Csv,
            "json" => DocumentType::Json,
            "xml" => DocumentType::Xml,
            "md" | "markdown" => DocumentType::Markdown,
            "txt" => DocumentType::Txt,
            _ => DocumentType::Unknown,
        }
    }
}

#[async_trait]
impl DocumentParser for AutoParser {
    fn supported_extensions(&self) -> Vec<&str> {
        let mut exts = Vec::new();
        for parser in &self.parsers {
            exts.extend(parser.supported_extensions());
        }
        exts
    }

    async fn parse(&self, content: &[u8], filename: &str) -> AppResult<ParsedDocument> {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        for parser in &self.parsers {
            if parser.supported_extensions().contains(&ext.as_str()) {
                return parser.parse(content, filename).await;
            }
        }
        Err(AppError::Validation(format!(
            "Unsupported file type: {}",
            ext
        )))
    }
}

pub fn extract_text(doc: &ParsedDocument) -> String {
    let mut text = String::new();
    for section in &doc.sections {
        if let Some(heading) = &section.heading {
            text.push_str(heading);
            text.push('\n');
        }
        text.push_str(&section.content);
        text.push('\n');
    }
    text.trim().to_string()
}

pub fn extract_tables_as_text(doc: &ParsedDocument) -> String {
    let mut text = String::new();
    for (i, table) in doc.tables.iter().enumerate() {
        text.push_str(&format!("Table {}:\n", i + 1));
        text.push_str(&table.headers.join(" | "));
        text.push('\n');
        text.push_str(&"-".repeat(table.headers.len() * 10));
        text.push('\n');
        for row in &table.rows {
            text.push_str(&row.join(" | "));
            text.push('\n');
        }
        text.push('\n');
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DocumentParser;

    #[tokio::test]
    async fn test_csv_parser() {
        let parser = CsvParser;
        let csv = b"name,amount\nAcme,100\nBeta,250\n";
        let doc = parser.parse(csv, "data.csv").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Csv);
        assert_eq!(doc.tables.len(), 1);
        assert_eq!(doc.tables[0].headers, vec!["name", "amount"]);
        assert_eq!(doc.tables[0].rows.len(), 2);
    }

    #[tokio::test]
    async fn test_tsv_parser() {
        let parser = CsvParser;
        let tsv = b"name\tamount\nAcme\t100\n";
        let doc = parser.parse(tsv, "data.tsv").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Csv);
        assert_eq!(doc.tables[0].rows[0][0], "Acme");
    }

    #[tokio::test]
    async fn test_json_parser() {
        let parser = JsonParser;
        let json = br#"{"name": "Acme", "amount": 100}"#;
        let doc = parser.parse(json, "data.json").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Json);
        assert_eq!(doc.sections.len(), 1);
    }

    #[tokio::test]
    async fn test_json_parser_invalid() {
        let parser = JsonParser;
        let json = b"not json";
        let result = parser.parse(json, "data.json").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_markdown_parser() {
        let parser = PlainTextParser;
        let md = b"# Title\nSome content\n## Section 1\nMore content\n";
        let doc = parser.parse(md, "doc.md").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Markdown);
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.sections[0].heading.as_deref(), Some("Title"));
        assert_eq!(doc.sections[1].heading.as_deref(), Some("Section 1"));
    }

    #[tokio::test]
    async fn test_txt_parser() {
        let parser = PlainTextParser;
        let txt = b"Hello world\nThis is a test\n";
        let doc = parser.parse(txt, "doc.txt").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Txt);
        assert!(doc.metadata.word_count > 0);
    }

    #[tokio::test]
    async fn test_auto_parser_detect() {
        assert_eq!(AutoParser::detect_type("doc.pdf"), DocumentType::Pdf);
        assert_eq!(AutoParser::detect_type("doc.xlsx"), DocumentType::Xlsx);
        assert_eq!(AutoParser::detect_type("doc.csv"), DocumentType::Csv);
        assert_eq!(AutoParser::detect_type("doc.json"), DocumentType::Json);
        assert_eq!(AutoParser::detect_type("doc.md"), DocumentType::Markdown);
        assert_eq!(AutoParser::detect_type("doc.txt"), DocumentType::Txt);
        assert_eq!(AutoParser::detect_type("doc.unknown"), DocumentType::Unknown);
    }

    #[tokio::test]
    async fn test_auto_parser_routing() {
        let parser = AutoParser::new();
        let csv = b"name,amount\nAcme,100\n";
        let doc = parser.parse(csv, "data.csv").await.unwrap();
        assert_eq!(doc.doc_type, DocumentType::Csv);
    }

    #[tokio::test]
    async fn test_auto_parser_unsupported() {
        let parser = AutoParser::new();
        let data = b"binary data";
        let result = parser.parse(data, "file.xyz").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_text() {
        let doc = ParsedDocument {
            document_id: "1".to_string(),
            filename: "test.txt".to_string(),
            doc_type: DocumentType::Txt,
            sections: vec![Section {
                heading: Some("Title".to_string()),
                content: "Body text".to_string(),
                level: 1,
                page: None,
            }],
            tables: vec![],
            metadata: DocumentMetadata {
                page_count: None,
                word_count: 3,
                created_at: None,
                author: None,
            },
        };
        let text = extract_text(&doc);
        assert!(text.contains("Title"));
        assert!(text.contains("Body text"));
    }

    #[tokio::test]
    async fn test_extract_tables_as_text() {
        let doc = ParsedDocument {
            document_id: "1".to_string(),
            filename: "test.csv".to_string(),
            doc_type: DocumentType::Csv,
            sections: vec![],
            tables: vec![Table {
                headers: vec!["name".to_string(), "amount".to_string()],
                rows: vec![vec!["Acme".to_string(), "100".to_string()]],
                page: None,
            }],
            metadata: DocumentMetadata {
                page_count: None,
                word_count: 2,
                created_at: None,
                author: None,
            },
        };
        let text = extract_tables_as_text(&doc);
        assert!(text.contains("name"));
        assert!(text.contains("Acme"));
    }
}