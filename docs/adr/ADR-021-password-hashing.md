# ADR-021: Password Hashing

## Status

Proposed

## Context

Secure password storage.

## Decision

**argon2id** (preferred) or bcrypt.

## Alternatives

- **scrypt**: Good but less widely used
- **bcrypt**: Mature but slower

## Consequences

- Memory-hard function
- Resistant to GPU attacks
- Configurable parameters
- Industry standard