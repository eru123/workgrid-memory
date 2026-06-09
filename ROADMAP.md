# Roadmap

This roadmap is a planning document, not a contract with the universe. The universe has lawyers.

## Stage 0: Planning

- [ ] Finalize `DESIGN.md`
- [ ] Finalize `IMPLEMENTATION.md`
- [ ] Create GitHub community files
- [ ] Select license
- [ ] Define MVP scope

## Stage 1: Desktop Shell

- [ ] Create Tauri + React application
- [ ] Add workspace picker
- [ ] Add settings page
- [ ] Add local app data directory
- [ ] Add logging
- [ ] Add basic dashboard

## Stage 2: Workspace Indexing

- [ ] Add file scanner
- [ ] Add ignore rules
- [ ] Add hashing
- [ ] Add SQLite metadata store
- [ ] Add manual reindex command
- [ ] Add file list UI

## Stage 3: Search Memory

- [ ] Add chunking
- [ ] Add SQLite FTS search
- [ ] Add embedding provider abstraction
- [ ] Add local embedding provider
- [ ] Add vector store
- [ ] Add search UI

## Stage 4: Code Intelligence

- [ ] Add Tree-sitter parser integration
- [ ] Extract functions/classes/methods
- [ ] Extract routes/configs/database references
- [ ] Add symbol graph
- [ ] Add related-file detection
- [ ] Add workspace architecture map

## Stage 5: Global Profile Memory

- [ ] Add profile database
- [ ] Add profile manager UI
- [ ] Add profile field privacy
- [ ] Add profile assets
- [ ] Add profile search
- [ ] Add profile-to-workspace linking

## Stage 6: MCP Integration

- [ ] Add local MCP server
- [ ] Add MCP on/off toggle
- [ ] Add stdio bridge CLI
- [ ] Add workspace tools
- [ ] Add profile tools
- [ ] Add context-pack builder
- [ ] Add access logs

## Stage 7: Hardening

- [ ] Add secret redaction tests
- [ ] Add profile exposure tests
- [ ] Add path traversal tests
- [ ] Add stale index warnings
- [ ] Add backup/restore
- [ ] Add import/export
- [ ] Add crash recovery

## Stage 8: Release

- [ ] Package desktop app
- [ ] Add install instructions
- [ ] Add signed builds if possible
- [ ] Publish first pre-release
