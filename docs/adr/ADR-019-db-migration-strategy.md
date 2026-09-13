# ADR-019: Database Migration Strategy

## Status

Proposed

## Context

Schema management for PostgreSQL.

## Decision

**golang-migrate** or **atlas** (to be confirmed).

## Alternatives

- Raw SQL files
- ORM migrations

## Consequences

- Version-controlled schemas
- Rollback support
- CI/CD integration
- Down migrations