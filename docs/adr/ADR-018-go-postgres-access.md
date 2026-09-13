# ADR-018: Go PostgreSQL Access Layer

## Status

Proposed

## Context

Go API needs PostgreSQL access.

## Decision

**pgx** or **sqlx** (to be confirmed).

## Alternatives

- **gorm**: ORM, more abstract
- **database/sql**: Standard library

## Consequences

- Type-safe queries
- Connection pooling
- Transaction support
- Migration support