# ADR-006: Rust-First Engine with TypeScript Sidecar Option

**Status:** Accepted
**Date:** 2026-06-09

## Context

The WorkGrid Memory engine handles file scanning, hashing, chunking, symbol extraction, embedding queuing, and retrieval. It must be fast, handle large workspaces, and integrate with Tauri's Rust backend.

## Decision

Build the core engine in Rust as a crate within the Cargo workspace, with a TypeScript sidecar kept as a fallback option for LanceDB/MCP integration if the Rust-native approach proves difficult.

## Rationale

- **Performance**: Rust handles large file trees and compute-intensive operations efficiently
- **Tauri integration**: Direct `tauri::command` exposure from Rust engine functions
- **Tree-sitter**: Rust bindings for tree-sitter are mature
- **Safety**: Memory safety for file operations
- **Sidecar bundling**: Tauri's `externalBin` can bundle the engine as a standalone binary if needed

## Consequences

- Steeper learning curve for contributors unfamiliar with Rust
- LanceDB TypeScript SDK is more mature than Rust bindings; may need a Node sidecar temporarily
- Must maintain clear Rust API boundaries for Tauri commands
