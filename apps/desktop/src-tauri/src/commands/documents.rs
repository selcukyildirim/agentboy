use document_parser::parser::{extract_text, AutoParser, DocumentParser};
use rag_core::context_budget::ContextBudget;
use rag_core::embedding::LocalEmbeddingProvider;
use rag_core::injection::InjectionDefense;
use rag_core::pipeline::RagPipeline;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentInfo {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub chunk_count: usize,
    pub uploaded_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RagCitation {
    pub id: String,
    pub document_id: String,
    pub document_name: String,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub excerpt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RagHit {
    pub chunk_id: String,
    pub document_id: String,
    pub document_name: String,
    pub section: Option<String>,
    pub page: Option<u32>,
    pub content: String,
}

struct KnowledgeState {
    pipeline: RagPipeline,
    documents: Vec<DocumentInfo>,
}

impl KnowledgeState {
    fn new() -> Self {
        Self {
            pipeline: RagPipeline::new(Box::new(LocalEmbeddingProvider::new(256)))
                .with_chunk_size(512, 50)
                .with_weights(0.6, 0.4),
            documents: Vec::new(),
        }
    }
}

fn store() -> &'static Mutex<KnowledgeState> {
    static S: OnceLock<Mutex<KnowledgeState>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(KnowledgeState::new()))
}

fn doc_name(state: &KnowledgeState, document_id: &str) -> String {
    state
        .documents
        .iter()
        .find(|d| d.id == document_id)
        .map(|d| d.name.clone())
        .unwrap_or_else(|| document_id.to_string())
}

#[tauri::command]
pub async fn list_documents() -> Result<Vec<DocumentInfo>, String> {
    Ok(store().lock().await.documents.clone())
}

#[tauri::command]
pub async fn upload_document(
    name: String,
    content: Vec<u8>,
    content_type: String,
) -> Result<DocumentInfo, String> {
    // Real parsing via document-parser (PDF/DOCX/XLSX/CSV/JSON/XML/TXT/MD).
    let parsed = AutoParser::new()
        .parse(&content, &name)
        .await
        .map_err(|e| e.to_string())?;
    let text = extract_text(&parsed);
    let doc_id = uuid::Uuid::new_v4().to_string();

    let mut state = store().lock().await;
    let ingest = state
        .pipeline
        .ingest(&doc_id, &text, &name)
        .await
        .map_err(|e| e.to_string())?;

    let doc = DocumentInfo {
        id: doc_id.clone(),
        name: name.clone(),
        content_type,
        size_bytes: content.len() as u64,
        chunk_count: ingest.chunk_count,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };
    state.documents.push(doc.clone());

    crate::commands::health::record_document_indexed();
    tracing::info!(doc_id = %doc_id, name = %name, chunks = ingest.chunk_count, "Document ingested");

    Ok(doc)
}

#[tauri::command]
pub async fn query_rag(query: String, top_k: Option<usize>) -> Result<serde_json::Value, String> {
    let k = top_k.unwrap_or(5);
    tracing::info!(query = %query, top_k = k, "RAG query from UI");

    let state = store().lock().await;

    let mut budget = ContextBudget::new(4000, 500);
    let result = state
        .pipeline
        .query(&query, k, None, Some(&mut budget))
        .await
        .map_err(|e| e.to_string())?;

    // Prompt-injection defense on retrieved context.
    let defense = InjectionDefense::new();
    let suspicious = defense.is_suspicious(&result.context_text);
    let safe_context = if suspicious {
        tracing::warn!("Suspicious content detected in retrieved context; sanitizing");
        defense.sanitize(&result.context_text)
    } else {
        result.context_text.clone()
    };

    let hits: Vec<RagHit> = result
        .chunks
        .iter()
        .map(|c| RagHit {
            chunk_id: c.chunk_id.clone(),
            document_id: c.document_id.clone(),
            document_name: doc_name(&state, &c.document_id),
            section: c.metadata.heading.clone().or_else(|| c.metadata.section.clone()),
            page: c.metadata.page,
            content: c.content.clone(),
        })
        .collect();

    let citations: Vec<RagCitation> = result
        .citations
        .iter()
        .map(|c| RagCitation {
            id: c.id.clone(),
            document_id: c.document_id.clone(),
            document_name: c.document_name.clone(),
            section: c.section.clone(),
            page: c.page,
            excerpt: c.excerpt.clone(),
        })
        .collect();

    Ok(serde_json::json!({
        "query": query,
        "count": hits.len(),
        "results": hits,
        "citations": citations,
        "context_text": safe_context,
        "injection_flagged": suspicious,
    }))
}
