# ADR-005: MCP Server with HTTP + stdio Transport

**Status:** Accepted
**Date:** 2026-06-09

## Context

WorkGrid Memory must expose workspace and profile context to AI coding agents through MCP (Model Context Protocol). Different MCP clients support different transports — some expect HTTP/SSE, others expect stdio.

## Decision

Support two MCP transports: Streamable HTTP as the primary transport (served by the desktop app) and a stdio bridge CLI as a secondary transport.

## Rationale

- **Streamable HTTP**: Fits the desktop app's always-running model; bind to `127.0.0.1` for security
- **stdio bridge CLI** (`workgrid-memory-mcp`): Compatible with clients that launch MCP servers as subprocesses; bridges to the running app or reads the index directly
- **Read-only tools only**: MVP restricts MCP to search, context retrieval, and claim verification
- **Local token auth**: HTTP transport requires a local auth token
- **Toggleable**: MCP server is off by default; user enables via UI

## Consequences

- Two code paths for tool execution (HTTP server + stdio bridge)
- Must maintain auth token lifecycle (generate, rotate, display)
- Profile tools require separate permission checks from workspace tools
