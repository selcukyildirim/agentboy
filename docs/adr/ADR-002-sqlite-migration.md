# ADR-002: SQLite & Migration Library

## Status

Accepted

## Context

Local storage for Free Desktop Client needs:
- Embedded database (no server)
- ACID compliance
- Migration support
- Async support for Tauri
- Compile-time query verification

## Decision

Use **sqlx** with SQLite backend and built-in migration support.

## Alternatives

- **rusqlite**: Mature but synchronous only
- **diesel**: Compile-time verified but heavier setup
- **sled**: Key-value store, not relational

## Consequences

- Async/await support via tokio
- Compile-time SQL verification
- Built-in migration framework
- SQLite for simplicity and portability
- Single file database