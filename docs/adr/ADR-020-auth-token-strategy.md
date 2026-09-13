# ADR-020: Auth Token Strategy

## Status

Proposed

## Context

Authentication token management.

## Decision

JWT access tokens + refresh token rotation.

## Alternatives

- Session-based: Server-side state
- Opaque tokens: Simpler but less scalable

## Consequences

- Stateless access tokens
- Refresh token rotation for security
- Short-lived access tokens (15 min)
- Long-lived refresh tokens (7 days)