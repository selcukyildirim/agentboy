use crate::chunker::Chunk;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MetadataFilter {
    pub document_id: Option<String>,
    pub content_type: Option<String>,
    pub page: Option<u32>,
    pub section: Option<String>,
    pub custom: HashMap<String, String>,
}

impl MetadataFilter {
    pub fn new() -> Self {
        Self {
            document_id: None,
            content_type: None,
            page: None,
            section: None,
            custom: HashMap::new(),
        }
    }

    pub fn with_document_id(mut self, id: &str) -> Self {
        self.document_id = Some(id.to_string());
        self
    }

    pub fn with_section(mut self, section: &str) -> Self {
        self.section = Some(section.to_string());
        self
    }

    pub fn matches(&self, chunk: &Chunk) -> bool {
        if let Some(ref doc_id) = self.document_id {
            if &chunk.document_id != doc_id {
                return false;
            }
        }

        if let Some(page) = self.page {
            if chunk.metadata.page != Some(page) {
                return false;
            }
        }

        if let Some(ref section) = self.section {
            if chunk.metadata.section.as_ref() != Some(section) {
                return false;
            }
        }

        true
    }

    pub fn filter(&self, chunks: Vec<Chunk>) -> Vec<Chunk> {
        chunks.into_iter().filter(|c| self.matches(c)).collect()
    }
}

impl Default for MetadataFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    fn make_chunk(doc_id: &str, section: &str) -> Chunk {
        Chunk {
            chunk_id: format!("{}-0", doc_id),
            document_id: doc_id.to_string(),
            content: "test".to_string(),
            index: 0,
            metadata: ChunkMetadata {
                page: None,
                section: Some(section.to_string()),
                start_offset: 0,
                end_offset: 4,
                heading: Some(section.to_string()),
                heading_level: Some(1),
            },
            token_estimate: 1,
        }
    }

    #[test]
    fn test_filter_by_document() {
        let filter = MetadataFilter::new().with_document_id("doc1");
        let chunks = vec![make_chunk("doc1", "intro"), make_chunk("doc2", "intro")];
        let filtered = filter.filter(chunks);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].document_id, "doc1");
    }

    #[test]
    fn test_filter_by_section() {
        let filter = MetadataFilter::new().with_section("intro");
        let chunks = vec![
            make_chunk("doc1", "intro"),
            make_chunk("doc1", "conclusion"),
        ];
        let filtered = filter.filter(chunks);
        assert_eq!(filtered.len(), 1);
    }
}
