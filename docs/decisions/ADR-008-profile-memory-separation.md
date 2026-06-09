# ADR-008: Global Profile Memory Separation

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory supports two scopes of memory: workspace memory (project-specific) and global profile memory (user-wide, reusable across workspaces). These must remain separate to prevent workspace data contamination and profile privacy leaks.

## Decision

Store global profiles in a separate `profiles.sqlite` database and a separate LanceDB vector collection (`profile-vectors.lance/`), outside of any workspace directory. Workspace indexes remain fully self-contained within their workspace storage directories.

## Rationale

- **Privacy boundary**: Deleting a workspace must not delete user profiles
- **Permission clarity**: Profile MCP exposure is controlled independently of workspace exposure
- **Data integrity**: Workspace indexes never absorb unrelated personal data
- **Auditability**: Profile access can be logged separately from workspace access

## Consequences

- Two database systems to maintain (profiles vs workspace metadata)
- Profile-to-workspace links require explicit join logic
- Context packs must merge workspace and profile results with clear labeling
- Must handle workspaces with no linked profiles gracefully
