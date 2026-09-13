use crate::chunker::Chunk;
use crate::embedding::cosine_similarity;
use std::collections::HashMap;

pub struct VectorIndex {
    vectors: Vec<(String, Vec<f32>)>,
    chunk_map: HashMap<String, Chunk>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
            chunk_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, chunk: Chunk, embedding: Vec<f32>) {
        let id = chunk.chunk_id.clone();
        self.vectors.push((id.clone(), embedding));
        self.chunk_map.insert(id, chunk);
    }

    pub fn search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(Chunk, f32)> {
        let mut scores: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, emb)| {
                let score = cosine_similarity(query_embedding, emb);
                (id.clone(), score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);

        scores
            .into_iter()
            .filter_map(|(id, score)| {
                self.chunk_map.get(&id).map(|chunk| (chunk.clone(), score))
            })
            .collect()
    }

    pub fn delete_by_document(&mut self, document_id: &str) -> usize {
        let initial = self.vectors.len();
        self.vectors
            .retain(|(id, _)| !id.starts_with(document_id));
        self.chunk_map
            .retain(|id, _| !id.starts_with(document_id));
        initial - self.vectors.len()
    }

    pub fn count(&self) -> usize {
        self.vectors.len()
    }
}

impl Default for VectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    fn make_chunk(id: &str) -> Chunk {
        Chunk {
            chunk_id: id.to_string(),
            document_id: "doc1".to_string(),
            content: format!("Content for {}", id),
            index: 0,
            metadata: ChunkMetadata {
                page: None,
                section: None,
                start_offset: 0,
                end_offset: 10,
                heading: None,
                heading_level: None,
            },
            token_estimate: 3,
        }
    }

    #[test]
    fn test_vector_index_insert_and_search() {
        let mut index = VectorIndex::new();
        let chunk1 = make_chunk("c1");
        let chunk2 = make_chunk("c2");

        index.insert(chunk1, vec![1.0, 0.0, 0.0]);
        index.insert(chunk2, vec![0.0, 1.0, 0.0]);

        let results = index.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.chunk_id, "c1");
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn test_delete_by_document() {
        let mut index = VectorIndex::new();
        index.insert(make_chunk("doc1-c1"), vec![1.0, 0.0]);
        index.insert(make_chunk("doc2-c1"), vec![0.0, 1.0]);

        let removed = index.delete_by_document("doc1");
        assert_eq!(removed, 1);
        assert_eq!(index.count(), 1);
    }
}