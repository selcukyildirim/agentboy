use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentInfo {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub chunk_count: usize,
    pub uploaded_at: String,
}

static mut DOCUMENTS: Option<Vec<DocumentInfo>> = None;

fn get_documents() -> &'static mut Vec<DocumentInfo> {
    unsafe {
        if DOCUMENTS.is_none() {
            DOCUMENTS = Some(Vec::new());
        }
        DOCUMENTS.as_mut().unwrap()
    }
}

#[tauri::command]
pub fn list_documents() -> Vec<DocumentInfo> {
    get_documents().clone()
}

#[tauri::command]
pub fn upload_document(name: String, content: Vec<u8>, content_type: String) -> Result<DocumentInfo, String> {
    let doc = DocumentInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.clone(),
        content_type: content_type.clone(),
        size_bytes: content.len() as u64,
        chunk_count: (content.len() / 1000).max(1),
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };

    get_documents().push(doc.clone());

    tracing::info!(doc_id = %doc.id, name = %name, "Document uploaded");

    Ok(doc)
}

#[tauri::command]
pub async fn query_rag(query: String, top_k: Option<usize>) -> Result<serde_json::Value, String> {
    let k = top_k.unwrap_or(5);

    tracing::info!(query = %query, top_k = k, "RAG query from UI");

    Ok(serde_json::json!({
        "query": query,
        "results": [],
        "count": 0,
        "message": "RAG query executed (placeholder - connect to rag-core)"
    }))
}