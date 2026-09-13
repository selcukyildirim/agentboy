# ADR-016: Connector Protocol

## Status

Proposed

## Context

ERP/DB/API connectors need a standard protocol.

## Decision

gRPC or REST-based connector protocol (to be defined).

## Alternatives

- Direct SDK: Tight coupling
- GraphQL: Flexible but complex

## Consequences

- Language-agnostic
- Streaming support
- Versioned contracts
- Health checks