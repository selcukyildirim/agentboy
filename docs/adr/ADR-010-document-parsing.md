# ADR-010: Document Parsing Stack

## Status

Accepted

## Context

Parsing various document formats for RAG.

## Decision

- **calamine**: XLSX reading
- **csv**: CSV parsing
- **pdf-extract**: PDF text extraction
- **docx-rs**: DOCX parsing

## Alternatives

- **poppler**: System dependency, not pure Rust
- **libreoffice headless**: Heavy, external process

## Consequences

- Pure Rust where possible
- Minimal system dependencies
- Async-friendly interfaces