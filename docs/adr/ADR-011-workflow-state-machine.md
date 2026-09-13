# ADR-011: Workflow State Machine

## Status

Accepted

## Context

Local workflow execution needs state management.

## Decision

Custom enum-based FSM with explicit state transitions.

## Alternatives

- **state_machine crate**: Generic but less domain-specific

## Consequences

- Domain-specific states
- Simple, explicit transitions
- Easy to serialize/deserialize
- Audit-friendly