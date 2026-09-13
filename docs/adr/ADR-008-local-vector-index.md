# ADR-008: Local Vector Index

## Status

Accepted

## Context

Local vector storage for RAG embeddings.

## Decision

Use **lancedb** or **qdrant-local** (to be decided during PHASE-06).

## Alternatives

- **hnswlib-rust**: HNSW index, lower level
- Custom HNSW: More control but more work

## Consequences

- Embedded, no server needed
- SQLite-compatible
- Good performance for local use