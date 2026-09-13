# ADR-017: Go HTTP Router

## Status

Proposed

## Context

Go Control API needs HTTP routing.

## Decision

**chi** or **gin** (to be confirmed during PHASE-01A).

## Alternatives

- **echo**: Good but less standard
- **gorilla/mux**: Mature but less actively maintained

## Consequences

- Middleware support
- URL parameter handling
- Group routing
- Fast performance