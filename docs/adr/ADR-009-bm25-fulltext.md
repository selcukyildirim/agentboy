# ADR-009: BM25 / Full-Text Search

## Status

Accepted

## Context

Keyword search for RAG retrieval.

## Decision

Use **tantivy** for full-text search.

## Alternatives

- **sonic**: Fast but external dependency
- SQLite FTS5: Simpler but less powerful

## Consequences

- Pure Rust
- BM25 ranking
- Tokenization support
- Index stored locally