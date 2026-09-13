# ADR-025: Release Metadata / Desktop Update Trust

## Status

Proposed

## Context

Desktop app update mechanism.

## Decision

Signed update artifacts with Tauri updater.

## Alternatives

- Auto-update without signing: Less secure
- Manual updates: Poor UX

## Consequences

- Cryptographic verification
- Staged rollout
- Rollback support
- Platform-specific updaters