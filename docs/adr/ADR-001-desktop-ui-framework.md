# ADR-001: Desktop UI Framework

## Status

Accepted

## Context

AgentBoy needs a desktop UI framework for the Free Desktop Client. Requirements:
- Rust-based backend for native performance
- Web-based UI for rapid development
- Cross-platform support (macOS, Windows, Linux)
- Small binary size
- Access to native OS APIs

## Decision

Use **Tauri v2** as the desktop application framework.

## Alternatives

- **egui**: Pure Rust immediate-mode GUI. Good performance but limited web ecosystem.
- **Iced**: Elm-inspired Rust GUI. Clean architecture but smaller ecosystem.
- **Electron**: Widely used but large binary size and JavaScript-only backend.

## Consequences

- Binary size ~5-10MB (vs ~150MB for Electron)
- Rust backend for all agent logic
- Web frontend (React/Vue/Svelte) for UI
- Native API access via Tauri commands
- Automatic updates via Tauri updater
- Small team can develop rapidly