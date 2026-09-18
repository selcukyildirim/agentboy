use crate::bm25::BM25Retriever;
use crate::chunker::{Chunk, StructureAwareChunker};
use crate::citation::{Citation, CitationManager};
use crate::context_budget::ContextBudget;
use crate::embedding::EmbeddingProvider;
use crate::merger::{Reranker, RetrievalMerger};
use crate::metadata::MetadataFilter;
use crate::vector_index::VectorIndex;
use agent_common::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub document_id: String,
    pub chunk_count: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub chunks: Vec<Chunk>,
    pub citations: Vec<Citation>,
    pub context_text: String,
    pub total_tokens: usize,
}

pub struct RagPipeline {
    chunker: StructureAwareChunker,
    vector_index: VectorIndex,
    bm25: BM25Retriever,
    merger: RetrievalMerger,
    reranker: Reranker,
    embedding_provider: Box<dyn EmbeddingProvider>,
    document_names: HashMap<String, String>,
}

impl RagPipeline {
    pub fn new(embedding_provider: Box<dyn EmbeddingProvider>) -> Self {
        Self {
            chunker: StructureAwareChunker::new(512, 50),
            vector_index: VectorIndex::new(),
            bm25: BM25Retriever::new(),
            merger: RetrievalMerger::default(),
            reranker: Reranker::default(),
            embedding_provider,
            document_names: HashMap::new(),
        }
    }

    pub fn with_chunk_size(mut self, max_chunk_size: usize, overlap: usize) -> Self {
        self.chunker = StructureAwareChunker::new(max_chunk_size, overlap);
        self
    }

    pub fn with_weights(mut self, vector_weight: f32, bm25_weight: f32) -> Self {
        self.merger = RetrievalMerger::new(vector_weight, bm25_weight);
        self
    }

    pub async fn ingest(
        &mut self,
        document_id: &str,
        content: &str,
        document_name: &str,
    ) -> AppResult<IngestResult> {
        self.document_names
            .insert(document_id.to_string(), document_name.to_string());

        let chunks = self.chunker.chunk(document_id, content);
        if chunks.is_empty() {
            return Err(AppError::Validation(
                "No chunks produced from document".to_string(),
            ));
        }

        let total_tokens: usize = chunks.iter().map(|c| c.token_estimate).sum();

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed(&texts).await?;

        for (chunk, embedding) in chunks.into_iter().zip(embeddings) {
            self.bm25.add_document(&chunk);
            self.vector_index.insert(chunk, embedding);
        }

        Ok(IngestResult {
            document_id: document_id.to_string(),
            chunk_count: self.vector_index.count(),
            total_tokens,
        })
    }

    pub async fn query(
        &self,
        query: &str,
        top_k: usize,
        filter: Option<&MetadataFilter>,
        budget: Option<&mut ContextBudget>,
    ) -> AppResult<QueryResult> {
        let query_embedding = self.embedding_provider.embed(&[query.to_string()]).await?;

        let vector_results = self.vector_index.search(&query_embedding[0], top_k * 2);

        let bm25_results = self.bm25.search(query, top_k * 2);

        let mut merged = self.merger.merge(vector_results, bm25_results, top_k * 2);

        merged = self.reranker.rerank(query, merged);

        if let Some(f) = filter {
            merged.retain(|(chunk, _)| f.matches(chunk));
        }

        merged.truncate(top_k);

        let mut citation_manager = CitationManager::new();
        for (chunk, _) in &merged {
            let doc_name = self
                .document_names
                .get(&chunk.document_id)
                .cloned()
                .unwrap_or_else(|| chunk.document_id.clone());
            citation_manager.add_citation(Citation::from_chunk(chunk, &doc_name));
        }

        let chunks: Vec<Chunk> = merged.into_iter().map(|(c, _)| c).collect();
        let total_tokens: usize = chunks.iter().map(|c| c.token_estimate).sum();

        let selected_chunks = if let Some(b) = budget {
            b.select_chunks(chunks)
        } else {
            chunks
        };

        let context_text = selected_chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        Ok(QueryResult {
            chunks: selected_chunks,
            citations: citation_manager.get_citations().to_vec(),
            context_text,
            total_tokens,
        })
    }

    pub fn delete_document(&mut self, document_id: &str) -> usize {
        self.document_names.remove(document_id);
        self.vector_index.delete_by_document(document_id)
    }

    pub fn document_count(&self) -> usize {
        self.document_names.len()
    }

    pub fn chunk_count(&self) -> usize {
        self.vector_index.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::LocalEmbeddingProvider;

    fn make_pipeline() -> RagPipeline {
        RagPipeline::new(Box::new(LocalEmbeddingProvider::new(16)))
    }

    #[tokio::test]
    async fn test_ingest_and_query() {
        let mut pipeline = make_pipeline();
        let result = pipeline
            .ingest(
                "doc1",
                "# Revenue\nOur revenue grew 20% this quarter.\n\n# Expenses\nExpenses decreased by 5%.",
                "Financial Report",
            )
            .await
            .unwrap();

        assert_eq!(result.document_id, "doc1");
        assert!(result.chunk_count > 0);
        assert!(result.total_tokens > 0);

        let query_result = pipeline
            .query("revenue growth", 3, None, None)
            .await
            .unwrap();
        assert!(!query_result.chunks.is_empty());
        assert!(!query_result.citations.is_empty());
        assert!(!query_result.context_text.is_empty());
    }

    #[tokio::test]
    async fn test_ingest_multiple_documents() {
        let mut pipeline = make_pipeline();
        pipeline
            .ingest("doc1", "Revenue analysis for Q1", "Report Q1")
            .await
            .unwrap();
        pipeline
            .ingest("doc2", "Expense analysis for Q2", "Report Q2")
            .await
            .unwrap();

        assert_eq!(pipeline.document_count(), 2);
    }

    #[tokio::test]
    async fn test_delete_document() {
        let mut pipeline = make_pipeline();
        pipeline
            .ingest("doc1", "Some content here", "Doc 1")
            .await
            .unwrap();
        assert_eq!(pipeline.chunk_count(), 1);

        pipeline.delete_document("doc1");
        assert_eq!(pipeline.chunk_count(), 0);
        assert_eq!(pipeline.document_count(), 0);
    }

    #[tokio::test]
    async fn test_query_with_filter() {
        let mut pipeline = make_pipeline();
        pipeline
            .ingest("doc1", "# Revenue\nHigh revenue", "Report")
            .await
            .unwrap();

        let filter = MetadataFilter::new().with_document_id("doc1");
        let result = pipeline.query("revenue", 3, Some(&filter), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_with_budget() {
        let mut pipeline = make_pipeline();
        pipeline
            .ingest("doc1", "# Revenue\nHigh revenue this quarter", "Report")
            .await
            .unwrap();

        let mut budget = ContextBudget::new(100, 10);
        let result = pipeline.query("revenue", 3, None, Some(&mut budget)).await;
        assert!(result.is_ok());
        assert!(budget.used() > 0);
    }

    #[tokio::test]
    async fn test_empty_ingest() {
        let mut pipeline = make_pipeline();
        let result = pipeline.ingest("doc1", "", "Empty Doc").await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.chunk_count, 1);
    }
}
