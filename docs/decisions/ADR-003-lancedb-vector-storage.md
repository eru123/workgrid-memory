# ADR-003: LanceDB for Local Vector Storage

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory needs local vector storage for semantic search over code chunks and profile embeddings. The store must work without a server process, support metadata filtering alongside vector search, and handle incremental updates.

## Decision

Use LanceDB (local file-based mode) for vector storage.

## Rationale

- **Serverless**: LanceDB runs embedded, no separate database process
- **Lance columnar format**: Efficient for vectors + metadata in the same store
- **Incremental updates**: Supports append, delete, and update operations
- **Metadata filtering**: Can combine vector similarity with SQL-like filters
- **Rust + TypeScript SDKs**: Works in both the Rust engine and potential Node sidecars
- **Open-source**: Apache 2.0 license

## Alternatives Considered

- **ChromaDB**: Requires a Python/HTTP server process; adds deployment complexity
- **Qdrant**: Server process required; overkill for local-only
- **FAISS**: No persistent storage or metadata; requires separate database
- **pgvector**: Requires PostgreSQL server

## Consequences

- LanceDB is younger than SQLite; API may evolve
- Separate storage per workspace (one LanceDB directory per workspace index)
- Profile vectors stored in a separate LanceDB collection
