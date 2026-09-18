use crate::chunker::Chunk;

pub struct RetrievalMerger {
    vector_weight: f32,
    bm25_weight: f32,
}

impl RetrievalMerger {
    pub fn new(vector_weight: f32, bm25_weight: f32) -> Self {
        Self {
            vector_weight,
            bm25_weight,
        }
    }

    pub fn merge(
        &self,
        vector_results: Vec<(Chunk, f32)>,
        bm25_results: Vec<(Chunk, f32)>,
        top_k: usize,
    ) -> Vec<(Chunk, f32)> {
        let mut scores: std::collections::HashMap<String, (Chunk, f32)> =
            std::collections::HashMap::new();

        for (chunk, score) in vector_results {
            let entry = scores
                .entry(chunk.chunk_id.clone())
                .or_insert_with(|| (chunk, 0.0));
            entry.1 += score * self.vector_weight;
        }

        for (chunk, score) in bm25_results {
            let entry = scores
                .entry(chunk.chunk_id.clone())
                .or_insert_with(|| (chunk, 0.0));
            entry.1 += score * self.bm25_weight;
        }

        let mut results: Vec<(Chunk, f32)> = scores.into_values().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }
}

impl Default for RetrievalMerger {
    fn default() -> Self {
        Self::new(0.7, 0.3)
    }
}

pub struct Reranker;

impl Reranker {
    pub fn new() -> Self {
        Self
    }

    pub fn rerank(&self, query: &str, results: Vec<(Chunk, f32)>) -> Vec<(Chunk, f32)> {
        let mut reranked: Vec<(Chunk, f32)> = results
            .into_iter()
            .map(|(chunk, score)| {
                let boosted = self.compute_boost(query, &chunk);
                (chunk, score + boosted)
            })
            .collect();

        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        reranked
    }

    fn compute_boost(&self, query: &str, chunk: &Chunk) -> f32 {
        let mut boost = 0.0;

        if let Some(ref heading) = chunk.metadata.heading {
            let query_lower = query.to_lowercase();
            let heading_lower = heading.to_lowercase();
            if query_lower.contains(&heading_lower) || heading_lower.contains(&query_lower) {
                boost += 0.2;
            }
        }

        if chunk.metadata.heading_level == Some(1) {
            boost += 0.05;
        }

        boost
    }
}

impl Default for Reranker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    fn make_chunk(id: &str, heading: Option<&str>) -> Chunk {
        Chunk {
            chunk_id: id.to_string(),
            document_id: "doc1".to_string(),
            content: "test content".to_string(),
            index: 0,
            metadata: ChunkMetadata {
                page: None,
                section: heading.map(String::from),
                start_offset: 0,
                end_offset: 11,
                heading: heading.map(String::from),
                heading_level: Some(1),
            },
            token_estimate: 2,
        }
    }

    #[test]
    fn test_merge_results() {
        let merger = RetrievalMerger::new(0.7, 0.3);
        let v_results = vec![(make_chunk("c1", None), 0.9), (make_chunk("c2", None), 0.5)];
        let b_results = vec![(make_chunk("c1", None), 0.8), (make_chunk("c3", None), 0.7)];

        let merged = merger.merge(v_results, b_results, 10);
        assert!(merged.len() <= 3);
        assert!(merged[0].1 >= merged.last().unwrap().1);
    }

    #[test]
    fn test_rerank_heading_match() {
        let reranker = Reranker::new();
        let results = vec![
            (make_chunk("c1", Some("Introduction")), 0.5),
            (make_chunk("c2", Some("Unrelated")), 0.6),
        ];

        let reranked = reranker.rerank("introduction", results);
        assert_eq!(reranked[0].0.chunk_id, "c1");
    }
}
