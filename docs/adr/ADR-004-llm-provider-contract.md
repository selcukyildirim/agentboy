# ADR-004: LLM Provider Contract

## Status

Accepted

## Context

Multiple LLM providers need a unified interface.

## Decision

Custom `async_trait` based gateway with provider-agnostic types.

## Alternatives

- **litellm**: Python-focused, not suitable for Rust
- **langchain-rust**: Too heavyweight for our needs

## Consequences

- Full control over provider abstraction
- Minimal dependencies
- Provider-agnostic capability model
- Easy to add new providers