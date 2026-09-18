use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

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
pub struct RagResult {
    pub chunk_id: String,
    pub document_id: String,
    pub content: String,
    pub score: f64,
    pub metadata: serde_json::Value,
}

fn get_documents() -> &'static Mutex<Vec<DocumentInfo>> {
    static DOCUMENTS: OnceLock<Mutex<Vec<DocumentInfo>>> = OnceLock::new();
    DOCUMENTS.get_or_init(|| Mutex::new(Vec::new()))
}

fn get_chunks() -> &'static Mutex<Vec<ChunkEntry>> {
    static CHUNKS: OnceLock<Mutex<Vec<ChunkEntry>>> = OnceLock::new();
    CHUNKS.get_or_init(|| Mutex::new(Vec::new()))
}

#[derive(Debug, Clone)]
struct ChunkEntry {
    chunk_id: String,
    document_id: String,
    content: String,
    offset: usize,
}

#[tauri::command]
pub fn list_documents() -> Vec<DocumentInfo> {
    get_documents().lock().unwrap().clone()
}

#[tauri::command]
pub fn upload_document(name: String, content: Vec<u8>, content_type: String) -> Result<DocumentInfo, String> {
    let text = String::from_utf8(content.clone())
        .map_err(|e| format!("Invalid UTF-8 content: {}", e))?;

    let chunk_size = 1000;
    let chunks: Vec<String> = text
        .chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect())
        .collect();

    let chunk_count = chunks.len().max(1);
    let doc_id = uuid::Uuid::new_v4().to_string();

    let mut chunk_store = get_chunks().lock().unwrap();
    for (i, chunk) in chunks.iter().enumerate() {
        chunk_store.push(ChunkEntry {
            chunk_id: format!("{}-{}", doc_id, i),
            document_id: doc_id.clone(),
            content: chunk.clone(),
            offset: i * chunk_size,
        });
    }

    let doc = DocumentInfo {
        id: doc_id.clone(),
        name: name.clone(),
        content_type,
        size_bytes: content.len() as u64,
        chunk_count,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };

    get_documents().lock().unwrap().push(doc.clone());
    crate::commands::health::record_document_indexed();

    tracing::info!(
        doc_id = %doc_id,
        name = %name,
        chunk_count = chunk_count,
        "Document uploaded and chunked"
    );

    Ok(doc)
}

#[tauri::command]
pub async fn query_rag(query: String, top_k: Option<usize>) -> Result<serde_json::Value, String> {
    let k = top_k.unwrap_or(5);

    tracing::info!(query = %query, top_k = k, "RAG query from UI");

    let chunks = get_chunks().lock().unwrap();
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<(f64, &ChunkEntry)> = chunks
        .iter()
        .map(|chunk| {
            let content_lower = chunk.content.to_lowercase();
            let score: f64 = query_words
                .iter()
                .map(|word| {
                    let count = content_lower.matches(word).count();
                    count as f64
                })
                .sum();
            (score, chunk)
        })
        .filter(|(score, _)| *score > 0.0)
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);

    let results: Vec<serde_json::Value> = scored
        .iter()
        .map(|(score, chunk)| {
            serde_json::json!({
                "chunk_id": chunk.chunk_id,
                "document_id": chunk.document_id,
                "content": chunk.content,
                "score": score,
            })
        })
        .collect();

    Ok(serde_json::json!({
        "query": query,
        "results": results,
        "count": results.len(),
    }))
}
