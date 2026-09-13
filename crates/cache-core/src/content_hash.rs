use sha2::{Digest, Sha256};

pub fn content_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn operation_cache_key(
    operation_version: &str,
    input_hash: &str,
    normalized_params: &str,
    parser_version: &str,
) -> String {
    let combined = format!(
        "{}:{}:{}:{}",
        operation_version, input_hash, normalized_params, parser_version
    );
    content_hash(combined.as_bytes())
}
