use crate::chunker::Chunk;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub document_id: String,
    pub document_name: String,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub excerpt: String,
}

impl Citation {
    pub fn from_chunk(chunk: &Chunk, document_name: &str) -> Self {
        Self {
            id: format!("cite-{}-{}", chunk.document_id, chunk.chunk_id),
            document_id: chunk.document_id.clone(),
            document_name: document_name.to_string(),
            section: chunk.metadata.section.clone(),
            page: chunk.metadata.page,
            start_offset: chunk.metadata.start_offset,
            end_offset: chunk.metadata.end_offset,
            excerpt: chunk.content.chars().take(200).collect(),
        }
    }
}

pub struct CitationManager {
    citations: Vec<Citation>,
}

impl CitationManager {
    pub fn new() -> Self {
        Self {
            citations: Vec::new(),
        }
    }

    pub fn add_citation(&mut self, citation: Citation) {
        self.citations.push(citation);
    }

    pub fn get_citations(&self) -> &[Citation] {
        &self.citations
    }

    pub fn format_citations(&self) -> String {
        self.citations
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let section = c
                    .section
                    .as_deref()
                    .map(|s| format!(" - {}", s))
                    .unwrap_or_default();
                format!(
                    "[{}] {}{} ({} p.{})",
                    i + 1,
                    c.document_name,
                    section,
                    c.document_id,
                    c.page.unwrap_or(0) + 1
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for CitationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    #[test]
    fn test_citation_from_chunk() {
        let chunk = Chunk {
            chunk_id: "c1".to_string(),
            document_id: "doc1".to_string(),
            content: "Test content here".to_string(),
            index: 0,
            metadata: ChunkMetadata {
                page: Some(1),
                section: Some("Intro".to_string()),
                start_offset: 0,
                end_offset: 17,
                heading: Some("Intro".to_string()),
                heading_level: Some(1),
            },
            token_estimate: 3,
        };

        let citation = Citation::from_chunk(&chunk, "Test Doc");
        assert_eq!(citation.document_name, "Test Doc");
        assert_eq!(citation.section, Some("Intro".to_string()));
    }

    #[test]
    fn test_format_citations() {
        let mut manager = CitationManager::new();
        manager.add_citation(Citation {
            id: "c1".to_string(),
            document_id: "doc1".to_string(),
            document_name: "Report.pdf".to_string(),
            section: Some("Summary".to_string()),
            page: Some(0),
            start_offset: 0,
            end_offset: 100,
            excerpt: "...".to_string(),
        });

        let formatted = manager.format_citations();
        assert!(formatted.contains("[1]"));
        assert!(formatted.contains("Report.pdf"));
    }
}
