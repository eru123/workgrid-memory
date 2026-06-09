# ADR-002: SQLite for Metadata and FTS Search

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory needs reliable local storage for workspace metadata (files, chunks, symbols, graph edges), job tracking, global profiles, and full-text search capability. The database must be embedded (no server process), fast, and durable.

## Decision

Use SQLite with FTS5 extension for all metadata and full-text keyword search.

## Rationale

- **Embedded, zero-config**: No database server to install or manage
- **FTS5**: Built-in full-text search with ranking, prefix queries, and phrase matching — sufficient for exact symbol/term lookup
- **Single-file per database**: Workspace isolation is natural (one SQLite file per workspace)
- **Well-tested**: SQLite is one of the most reliable software components in existence
- **Rust bindings**: `rusqlite` provides safe, ergonomic access
- **WAL mode**: Supports concurrent reads during writes

## Consequences

- Vector search requires a separate store (LanceDB)
- No built-in graph traversal — graph edges stored relationally
- Database file locking may affect concurrent access patterns
