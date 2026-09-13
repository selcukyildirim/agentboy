# ADR-007: Cache Implementation

## Status

Accepted

## Context

Multi-level caching for performance.

## Decision

L1: **moka** (in-memory) + L2: SQLite

## Alternatives

- **dashmap**: Concurrent HashMap, no TTL
- **sled**: Embedded key-value store

## Consequences

- moka: LRU/TTL support, async
- SQLite: Persistent, queryable
- Content-addressable keys via SHA-256