# ADR-007: Monorepo Structure with pnpm + Cargo Workspaces

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory consists of multiple components: a Tauri desktop app (React + Rust), a Rust engine crate, an MCP server crate, shared types, and UI packages. These components must be developed together but can be built and tested independently.

## Decision

Use a monorepo with pnpm workspaces for JavaScript/TypeScript packages and a Cargo workspace for Rust crates.

**Directory structure:**
```
apps/desktop/         # Tauri + React app
crates/engine/        # Core indexing/retrieval engine
crates/mcp-server/    # MCP server
crates/shared/        # Shared Rust types
packages/ui/          # Shared React components
packages/schemas/     # Shared TypeScript types/schemas
packages/profile-schemas/ # Profile type definitions
docs/                 # Documentation
fixtures/             # Test fixtures
scripts/              # Dev/build scripts
```

## Rationale

- **Single versioning**: One source of truth for all packages
- **Shared types**: Schema packages can be consumed by both frontend and sidecars
- **Simplified CI**: One repository to clone, build, and test
- **pnpm workspaces**: Efficient disk usage, strict dependency boundaries
- **Cargo workspace**: Shared compilation cache, consistent Rust edition

## Consequences

- Repository will grow large; `.gitignore` must exclude build artifacts
- Need both pnpm and Cargo toolchains for development
- Cross-language type sharing requires code generation or manual synchronization
