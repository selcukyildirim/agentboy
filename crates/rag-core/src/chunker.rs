use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub index: usize,
    pub metadata: ChunkMetadata,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub page: Option<u32>,
    pub section: Option<String>,
    pub start_offset: usize,
    pub end_offset: usize,
    pub heading: Option<String>,
    pub heading_level: Option<u32>,
}

pub struct StructureAwareChunker {
    max_chunk_size: usize,
    overlap: usize,
    min_chunk_size: usize,
}

impl StructureAwareChunker {
    pub fn new(max_chunk_size: usize, overlap: usize) -> Self {
        Self {
            max_chunk_size,
            overlap,
            min_chunk_size: 50,
        }
    }

    pub fn with_min_size(mut self, min: usize) -> Self {
        self.min_chunk_size = min;
        self
    }

    pub fn chunk(&self, document_id: &str, content: &str) -> Vec<Chunk> {
        let sections = self.split_by_headings(content);
        let mut chunks = Vec::new();

        for section in sections {
            let section_chunks = self.chunk_section(document_id, &section);
            chunks.extend(section_chunks);
        }

        chunks
    }

    fn split_by_headings(&self, content: &str) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_level: u32 = 0;
        let mut current_content = String::new();
        let mut start_offset = 0;

        for line in content.lines() {
            if let Some((level, heading)) = self.parse_heading(line) {
                if !current_content.trim().is_empty() {
                    sections.push(Section {
                        heading: current_heading.clone(),
                        level: current_level,
                        content: current_content.trim().to_string(),
                        start_offset,
                        end_offset: start_offset + current_content.len(),
                    });
                }
                current_heading = Some(heading);
                current_level = level;
                current_content.clear();
                start_offset = content.find(line).unwrap_or(0);
            } else {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if !current_content.trim().is_empty() {
            sections.push(Section {
                heading: current_heading,
                level: current_level,
                content: current_content.trim().to_string(),
                start_offset,
                end_offset: start_offset + current_content.len(),
            });
        }

        if sections.is_empty() {
            sections.push(Section {
                heading: None,
                level: 0,
                content: content.to_string(),
                start_offset: 0,
                end_offset: content.len(),
            });
        }

        sections
    }

    fn parse_heading(&self, line: &str) -> Option<(u32, String)> {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count() as u32;
            let heading = trimmed.trim_start_matches('#').trim().to_string();
            if !heading.is_empty() {
                return Some((level, heading));
            }
        }
        None
    }

    fn chunk_section(&self, document_id: &str, section: &Section) -> Vec<Chunk> {
        let words: Vec<&str> = section.content.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.len() <= self.max_chunk_size {
            chunks.push(Chunk {
                chunk_id: format!("{}-{}", document_id, chunks.len()),
                document_id: document_id.to_string(),
                content: section.content.clone(),
                index: chunks.len(),
                metadata: ChunkMetadata {
                    page: None,
                    section: section.heading.clone(),
                    start_offset: section.start_offset,
                    end_offset: section.end_offset,
                    heading: section.heading.clone(),
                    heading_level: Some(section.level),
                },
                token_estimate: self.estimate_tokens(&section.content),
            });
        } else {
            let mut start = 0;
            while start < words.len() {
                let end = (start + self.max_chunk_size).min(words.len());
                let chunk_text = words[start..end].join(" ");
                let token_estimate = self.estimate_tokens(&chunk_text);

                chunks.push(Chunk {
                    chunk_id: format!("{}-{}", document_id, chunks.len()),
                    document_id: document_id.to_string(),
                    content: chunk_text,
                    index: chunks.len(),
                    metadata: ChunkMetadata {
                        page: None,
                        section: section.heading.clone(),
                        start_offset: section.start_offset + start,
                        end_offset: section.start_offset + end,
                        heading: section.heading.clone(),
                        heading_level: Some(section.level),
                    },
                    token_estimate,
                });

                start = end - self.overlap.min(end - start);
                if start + self.max_chunk_size >= words.len() {
                    break;
                }
            }
        }

        chunks
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}

struct Section {
    heading: Option<String>,
    level: u32,
    content: String,
    start_offset: usize,
    end_offset: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_with_headings() {
        let chunker = StructureAwareChunker::new(10, 2);
        let content = "# Heading 1\nThis is some content.\n## Heading 2\nMore content here.";
        let chunks = chunker.chunk("doc1", content);
        assert!(!chunks.is_empty());
        assert!(chunks[0].metadata.heading.is_some());
    }

    #[test]
    fn test_chunk_plain_text() {
        let chunker = StructureAwareChunker::new(5, 1);
        let content = "one two three four five six seven eight nine ten";
        let chunks = chunker.chunk("doc1", content);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_estimate_tokens() {
        let chunker = StructureAwareChunker::new(100, 10);
        assert_eq!(chunker.estimate_tokens("hello world"), 2);
    }
}
