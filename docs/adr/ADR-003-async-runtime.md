# ADR-003: Async Runtime Strategy

## Status

Accepted

## Context

Rust async runtime selection for the entire workspace.

## Decision

Use **tokio** as the async runtime.

## Alternatives

- **async-std**: Alternative async runtime, smaller ecosystem

## Consequences

- Tauri v2 uses tokio internally
- Largest Rust async ecosystem
- Battle-tested in production
- Rich middleware ecosystem (tower, reqwest, sqlx)