# ADR-015: Secrets Backend

## Status

Proposed

## Context

Paid server needs centralized secrets management.

## Decision

HashiCorp Vault or AWS Secrets Manager (cloud-dependent).

## Alternatives

- Environment variables: Simple but less secure
- Kubernetes secrets: Platform-specific

## Consequences

- Centralized secret management
- Audit trail
- Rotation support
- Encryption at rest