# ADR-004: Ollama as Default Embedding Provider

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory needs to generate embeddings for code chunks and safe profile fields to enable semantic search. The embedding provider must be local-first, free, and produce consistent vectors.

## Decision

Use Ollama with `nomic-embed-text` as the default embedding provider, with an abstraction layer supporting pluggable providers.

## Rationale

- **Local-first**: Matches the core principle; no data leaves the machine
- **Free**: No API costs for embedding generation
- **Consistent dimensions**: `nomic-embed-text` produces 768-dimensional vectors
- **Provider abstraction**: Supports future swap to OpenAI-compatible, Voyage, or custom providers
- **Health-checkable**: Can detect when Ollama is offline and degrade gracefully

## Consequences

- Requires user to install Ollama separately
- Embedding queue must handle provider unavailability gracefully
- Must track embedding model + dimensions per workspace index version
- Do not mix embeddings from different models in the same collection
