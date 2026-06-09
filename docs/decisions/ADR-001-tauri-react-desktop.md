# ADR-001: Tauri + React for Desktop Application

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory needs a local-first desktop application that can manage multiple workspaces, index files, run background processes, and serve an MCP server. The application must work on Linux, Windows, and macOS.

## Decision

Use Tauri v2 with React + TypeScript for the desktop application shell.

## Rationale

- **Tauri** provides native filesystem access, process management, sidecar bundling, and secure storage paths through Rust — all necessary for workspace indexing and MCP server lifecycle management.
- **React + TypeScript** gives a fast, maintainable UI with strong typing for the settings, dashboard, profile manager, search interface, and MCP controls.
- Tauri v2 supports `externalBin` for bundling sidecar binaries (engine, MCP bridge), matching the recommended three-executable design.
- Smaller bundle size than Electron; Rust backend avoids a second Node.js process.

## Consequences

- Requires Rust toolchain for development
- Tauri v2 is still maturing; some APIs may change
- Packaging for all three OS platforms requires CI setup
