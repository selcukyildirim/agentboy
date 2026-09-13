# ADR-005: HTTP Client & Retry

## Status

Accepted

## Context

HTTP client for LLM provider communication.

## Decision

Use **reqwest** with tower middleware for retry/timeout.

## Alternatives

- **hyper**: Lower level, more control but more boilerplate
- **ureq**: Synchronous, not suitable for async

## Consequences

- Async/await native
- Built-in connection pooling
- Tower middleware for retry/timeout
- TLS support via rustls