use crate::chunker::Chunk;

pub struct ContextBudget {
    max_tokens: usize,
    reserved_tokens: usize,
    used_tokens: usize,
}

impl ContextBudget {
    pub fn new(max_tokens: usize, reserved_tokens: usize) -> Self {
        Self {
            max_tokens,
            reserved_tokens,
            used_tokens: 0,
        }
    }

    pub fn available(&self) -> usize {
        self.max_tokens
            .saturating_sub(self.reserved_tokens)
            .saturating_sub(self.used_tokens)
    }

    pub fn can_fit(&self, chunk: &Chunk) -> bool {
        chunk.token_estimate <= self.available()
    }

    pub fn consume(&mut self, tokens: usize) -> bool {
        if tokens <= self.available() {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn select_chunks(&mut self, chunks: Vec<Chunk>) -> Vec<Chunk> {
        let mut selected = Vec::new();
        for chunk in chunks {
            if self.can_fit(&chunk) {
                self.consume(chunk.token_estimate);
                selected.push(chunk);
            }
        }
        selected
    }

    pub fn used(&self) -> usize {
        self.used_tokens
    }

    pub fn total(&self) -> usize {
        self.max_tokens
    }

    pub fn reset(&mut self) {
        self.used_tokens = 0;
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::new(8000, 2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::ChunkMetadata;

    fn make_chunk(tokens: usize) -> Chunk {
        Chunk {
            chunk_id: "c1".to_string(),
            document_id: "doc1".to_string(),
            content: "test".to_string(),
            index: 0,
            metadata: ChunkMetadata {
                page: None,
                section: None,
                start_offset: 0,
                end_offset: 4,
                heading: None,
                heading_level: None,
            },
            token_estimate: tokens,
        }
    }

    #[test]
    fn test_budget_select_chunks() {
        let mut budget = ContextBudget::new(100, 20);
        let chunks = vec![make_chunk(30), make_chunk(30), make_chunk(30), make_chunk(30)];
        let selected = budget.select_chunks(chunks);
        assert_eq!(selected.len(), 2);
        assert_eq!(budget.used(), 60);
    }

    #[test]
    fn test_budget_available() {
        let mut budget = ContextBudget::new(100, 20);
        assert_eq!(budget.available(), 80);
        budget.consume(30);
        assert_eq!(budget.available(), 50);
    }
}