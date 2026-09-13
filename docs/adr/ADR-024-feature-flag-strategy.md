# ADR-024: Feature Flag Strategy

## Status

Proposed

## Context

Feature toggling for gradual rollout.

## Decision

**LaunchDarkly** or custom PostgreSQL-based (to be confirmed).

## Alternatives

- **Unleash**: Open source
- Environment variables: Simple but limited

## Consequences

- Gradual rollout
- User targeting
- A/B testing
- Kill switches