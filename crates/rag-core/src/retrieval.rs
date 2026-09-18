use crate::chunker::Chunk;
use agent_common::error::AppResult;

pub struct HybridRetriever;

impl HybridRetriever {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub async fn retrieve(
        &self,
        _query: &str,
        _vector_results: Vec<Chunk>,
        _keyword_results: Vec<Chunk>,
    ) -> AppResult<Vec<Chunk>> {
        Ok(vec![])
    }
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self::new()
    }
}
