# ADR-006: Secure Credential Storage

## Status

Accepted

## Context

API keys need secure storage without plaintext in database.

## Decision

Use **keyring** crate for OS-backed credential storage.

## Alternatives

- Manual Keychain/libsecret: Platform-specific code
- Encrypted file: Less secure, key management complex

## Consequences

- macOS: Keychain
- Windows: Credential Manager
- Linux: Secret Service/libsecret
- Cross-platform abstraction
- Never store plaintext in DB