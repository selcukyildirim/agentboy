use crate::chunker::Chunk;
use std::collections::HashMap;

pub struct BM25Retriever {
    k1: f32,
    b: f32,
    index: HashMap<String, Vec<(String, usize)>>,
    doc_lengths: HashMap<String, usize>,
    avg_doc_length: f32,
    doc_count: usize,
    chunk_map: HashMap<String, Chunk>,
}

impl BM25Retriever {
    pub fn new() -> Self {
        Self {
            k1: 1.5,
            b: 0.75,
            index: HashMap::new(),
            doc_lengths: HashMap::new(),
            avg_doc_length: 0.0,
            doc_count: 0,
            chunk_map: HashMap::new(),
        }
    }

    pub fn add_document(&mut self, chunk: &Chunk) {
        let terms = self.tokenize(&chunk.content);
        let doc_id = chunk.chunk_id.clone();

        self.chunk_map.insert(doc_id.clone(), chunk.clone());
        self.doc_lengths.insert(doc_id.clone(), terms.len());
        self.doc_count += 1;

        let mut term_freq: HashMap<String, usize> = HashMap::new();
        for term in &terms {
            *term_freq.entry(term.clone()).or_insert(0) += 1;
        }

        for (term, freq) in term_freq {
            self.index
                .entry(term)
                .or_insert_with(Vec::new)
                .push((doc_id.clone(), freq));
        }

        let total: usize = self.doc_lengths.values().sum();
        self.avg_doc_length = total as f32 / self.doc_count as f32;
    }

    pub fn search(&self, query: &str, top_k: usize) -> Vec<(Chunk, f32)> {
        let query_terms = self.tokenize(query);
        let mut scores: HashMap<String, f32> = HashMap::new();

        for term in &query_terms {
            if let Some(postings) = self.index.get(term) {
                let df = postings.len() as f32;
                let idf = ((self.doc_count as f32 - df + 0.5) / (df + 0.5) + 1.0).ln();

                for (doc_id, tf) in postings {
                    let doc_len = self.doc_lengths.get(doc_id).copied().unwrap_or(0) as f32;
                    let tf_val = *tf as f32;
                    let tf_norm = (tf_val * (self.k1 + 1.0))
                        / (tf_val
                            + self.k1 * (1.0 - self.b + self.b * doc_len / self.avg_doc_length));
                    let score = idf * tf_norm;
                    *scores.entry(doc_id.clone()).or_insert(0.0) += score;
                }
            }
        }

        let mut results: Vec<(Chunk, f32)> = scores
            .into_iter()
            .filter_map(|(id, score)| {
                self.chunk_map.get(&id).map(|c| (c.clone(), score))
            })
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty() && s.len() > 2)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.doc_count
    }
}

impl Default for BM25Retriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    fn make_chunk(id: &str, content: &str) -> Chunk {
        Chunk {
            chunk_id: id.to_string(),
            document_id: "doc1".to_string(),
            content: content.to_string(),
            index: 0,
            metadata: ChunkMetadata {
                page: None,
                section: None,
                start_offset: 0,
                end_offset: content.len(),
                heading: None,
                heading_level: None,
            },
            token_estimate: content.split_whitespace().count(),
        }
    }

    #[test]
    fn test_bm25_search() {
        let mut retriever = BM25Retriever::new();
        retriever.add_document(&make_chunk("c1", "the cat sat on the mat"));
        retriever.add_document(&make_chunk("c2", "the dog played in the park"));
        retriever.add_document(&make_chunk("c3", "the cat and dog are friends"));

        let results = retriever.search("cat", 2);
        assert_eq!(results.len(), 2);
        assert!(results[0].1 >= results[1].1);
    }

    #[test]
    fn test_tokenize() {
        let retriever = BM25Retriever::new();
        let tokens = retriever.tokenize("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
    }
}