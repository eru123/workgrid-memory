# WorkGrid Memory

**WorkGrid Memory** is a local-first desktop application for building persistent, searchable memory across developer workspaces and reusable personal/profile context.

It indexes project files, symbols, functions, classes, routes, configs, documentation, and relationships into a local memory layer. It also supports global profiles for people, pets, places, products, workflows, ideas, processes, and personal preferences. These memories can be exposed through an MCP server so AI coding agents can retrieve grounded context instead of guessing like a very expensive autocomplete machine with confidence issues.

## Vision

AI coding agents often fail because they lack reliable project context. They read a few files, make assumptions, and then confidently edit code based on partial understanding.

WorkGrid Memory aims to solve that by becoming a local context engine for both:

1. **Workspace Memory**
   Project-specific memory for codebases, files, symbols, dependencies, routes, configs, docs, and architecture.

2. **Global Profile Memory**
   Reusable context that applies across workspaces, such as personal workflows, commit preferences, coding style, people, pets, places, jobs, products, business processes, and other definable profiles.

The goal is not to replace AI agents. The goal is to make them less useless by giving them grounded, searchable, cited context.

## Core Principles

* **Local-first by default**
* **Workspace data stays workspace-scoped**
* **Profiles are global but permission-controlled**
* **Sensitive profile data must not leak automatically**
* **AI agents should receive evidence, not vague vibes**
* **Every retrieved code context should include file paths and line ranges**
* **MCP tools should support verification, not just search**
* **No cloud dependency required for the core system**

## What WorkGrid Memory Does

WorkGrid Memory lets you add one or more project folders as workspaces.

For each workspace, it can:

* Scan project files
* Respect ignore rules
* Hash files for change detection
* Extract symbols such as functions, classes, methods, interfaces, routes, configs, and database references
* Build semantic chunks from meaningful code sections
* Generate embeddings
* Store local vector search data
* Store metadata, symbol graphs, and keyword indexes
* Watch files for incremental updates
* Expose workspace context through MCP tools

It also lets you create global profiles for anything with definable characteristics.

Examples:

* Personal profile
* Wife/family profile
* Friend profile
* Pet profile
* Client profile
* Company profile
* Product profile
* Place profile
* Object profile
* Idea profile
* Process profile
* Coding workflow profile
* Commit style profile
* Work habits profile
* Business rules profile

These profiles can extend AI context across projects without mixing them into workspace-only data.

## Workspace Memory vs Global Profile Memory

WorkGrid Memory separates context into two major scopes.

### Workspace Memory

Workspace memory belongs to a specific project.

Examples:

* `/home/user/projects/accounting-system`
* `/home/user/projects/coco-lang`
* `/home/user/www/client-portal`

Workspace memory includes:

* Files
* Code chunks
* Symbols
* Routes
* Classes
* Functions
* Configs
* Database schemas
* Migrations
* Documentation
* Dependency maps
* Architecture notes
* Local project decisions

Workspace memory should not be automatically reused in another workspace unless explicitly requested.

### Global Profile Memory

Global profiles are reusable across all workspaces.

Examples:

* How the user prefers commit messages
* How the user structures projects
* How the user usually works
* Personal coding rules
* Business preferences
* Client information
* People and relationship context
* Pet information
* Product facts
* Process documentation
* Ideas and concepts

Profiles can be used as additional context when relevant.

For example:

* A commit agent can retrieve the user’s preferred commit format.
* A project planning agent can retrieve the user’s usual development workflow.
* A client project agent can retrieve client profile context.
* A writing agent can retrieve tone and business preferences.
* A coding agent can retrieve reusable architectural preferences.

Profiles should have field-level privacy controls because some profile data may be sensitive.

## Example Use Cases

### Codebase Context for AI Agents

An agent asks:

> Where is invoice total calculated?

WorkGrid Memory can return:

* Relevant files
* Matching symbols
* Function snippets
* Line ranges
* Related files
* Confidence scores
* Evidence summaries

Instead of dumping the entire project into a model and sacrificing tokens to the void.

### Commit Style Memory

You can teach WorkGrid Memory your preferred commit style.

Example profile:

```json
{
  "type": "workflow",
  "name": "Commit Style",
  "description": "How I want commits to be written",
  "rules": [
    "Use concise commit messages",
    "Prefer conventional commits",
    "Use feat, fix, refactor, chore, docs, test",
    "Mention the affected module when useful",
    "Do not include overly dramatic explanations"
  ]
}
```

Then an MCP client can call WorkGrid Memory before generating commits.

### Personal Workflow Context

You can store how you usually work.

Example:

```json
{
  "type": "workflow",
  "name": "Development Workflow",
  "rules": [
    "Plan before implementing",
    "Prefer small incremental changes",
    "Avoid large rewrites unless necessary",
    "Document risky decisions",
    "Add disaster fallback plans for AI agent rate limits"
  ]
}
```

### Client or Business Context

A client profile can define:

* Company name
* Contact person
* Project preferences
* Pricing assumptions
* Technical stack
* Communication style
* Important constraints
* Prior agreements

This gives agents more accurate context when working on client-specific documents or code.

### Workspace-Aware Project Planning

A planning agent can combine:

* Current workspace architecture
* Project files
* Related symbols
* Existing documentation
* User development preferences
* Relevant global profiles

Then produce plans that fit the actual project instead of pretending every codebase is a clean greenfield fantasy. Adorable, but false.

## Architecture

WorkGrid Memory is designed as a desktop application built with:

* **React** for the frontend
* **Tauri** for the desktop shell
* **Rust** for native backend capabilities
* **SQLite** for metadata, profiles, graph data, and keyword search
* **SQLite FTS5** for fast full-text search
* **LanceDB or similar local vector storage** for embeddings
* **Tree-sitter** for code parsing and symbol extraction
* **MCP server layer** for AI agent access
* **Local embedding providers** such as Ollama
* Optional hosted embedding providers later

## High-Level System Design

```text
WorkGrid Memory
│
├─ Desktop App
│  ├─ React UI
│  ├─ Workspace dashboard
│  ├─ Profile manager
│  ├─ Search interface
│  ├─ MCP server controls
│  └─ Settings
│
├─ Tauri Backend
│  ├─ File system access
│  ├─ Process management
│  ├─ Secure storage paths
│  ├─ Workspace permissions
│  └─ Native commands
│
├─ Index Engine
│  ├─ File scanner
│  ├─ File watcher
│  ├─ Ignore matcher
│  ├─ Hashing
│  ├─ Code parser
│  ├─ Symbol extractor
│  ├─ Chunk generator
│  └─ Embedding queue
│
├─ Memory Storage
│  ├─ Workspace metadata
│  ├─ File chunks
│  ├─ Symbol graph
│  ├─ Profile memory
│  ├─ Keyword index
│  └─ Vector index
│
└─ MCP Layer
   ├─ Streamable HTTP server
   ├─ stdio bridge CLI
   ├─ Workspace tools
   ├─ Profile tools
   └─ Context-pack builder
```

## Recommended Runtime Components

WorkGrid Memory should eventually be split into three runtime parts:

```text
1. workgrid-memory
   The main Tauri desktop application.

2. workgrid-memory-engine
   The background indexing and search engine.

3. workgrid-memory-mcp
   The MCP server or stdio bridge used by AI agents.
```

This separation keeps the UI responsive and makes the MCP layer easier to integrate with different AI clients.

## MCP Support

WorkGrid Memory should expose context through MCP.

Recommended transports:

1. **Streamable HTTP**

   * Best for the desktop app’s local server mode
   * Runs at a local address such as `http://127.0.0.1:3847/mcp`
   * Controlled by an on/off toggle in the app

2. **stdio bridge**

   * Used for agents that expect to launch an MCP server as a subprocess
   * The bridge connects back to the local WorkGrid Memory service or reads the local index directly

## Example MCP Configuration

```json
{
  "mcpServers": {
    "workgrid-memory": {
      "command": "workgrid-memory-mcp",
      "args": ["stdio"]
    }
  }
}
```

For HTTP-compatible clients:

```json
{
  "mcpServers": {
    "workgrid-memory": {
      "url": "http://127.0.0.1:3847/mcp",
      "headers": {
        "Authorization": "Bearer YOUR_LOCAL_TOKEN"
      }
    }
  }
}
```

## Workspace MCP Tools

### `search_workspace`

Searches workspace memory using hybrid retrieval.

Combines:

* Vector search
* Keyword search
* Symbol search
* File graph relationships
* Recency signals

### `get_file_context`

Returns context for a specific file.

May include:

* Summary
* Symbols
* Imports
* Exports
* Related files
* Relevant chunks
* Line ranges

### `explain_symbol`

Explains a class, function, method, route, model, config key, or database reference.

### `find_references`

Finds references to a symbol using indexed graph data and keyword fallback.

### `get_related_files`

Returns files connected through imports, routes, database usage, tests, configs, or symbol relationships.

### `get_workspace_map`

Returns a compact architecture map of the workspace.

### `verify_claim`

Checks whether a statement is supported by indexed project evidence.

Example:

```json
{
  "claim": "The app uses JWT for authentication",
  "verdict": "contradicted",
  "evidence": [
    {
      "file": "app/Auth/AuthService.php",
      "lines": "20-44",
      "summary": "Authentication uses session cookies, not JWT."
    }
  ]
}
```

## Profile MCP Tools

### `search_profiles`

Searches global profile memory.

Example input:

```json
{
  "query": "How does the user prefer commit messages?"
}
```

### `get_profile_context`

Returns a specific profile with allowed fields only.

### `get_relevant_profiles`

Finds profiles relevant to a task, workspace, or agent request.

Example:

```json
{
  "task": "Generate a commit message for the current code changes",
  "workspace": "accounting-system"
}
```

### `build_context_pack`

Combines workspace memory and relevant global profiles into one structured context package.

Example result:

```json
{
  "workspace": {
    "name": "accounting-system",
    "relevantFiles": [
      {
        "path": "app/Services/InvoiceService.php",
        "lines": "40-88",
        "summary": "Calculates invoice totals."
      }
    ]
  },
  "profiles": [
    {
      "name": "Commit Style",
      "type": "workflow",
      "summary": "User prefers concise conventional commits."
    }
  ],
  "warnings": [
    "Sensitive profile fields were excluded."
  ]
}
```

## Profile Types

Possible profile types include:

```text
person
pet
place
object
product
idea
process
workflow
client
company
project
skill
preference
rule_set
asset
document
custom
```

Profiles should be flexible and user-definable.

## Profile Privacy

Profiles may contain sensitive or personal data, so WorkGrid Memory should support:

* Field-level privacy
* Sensitive profile flags
* MCP exposure controls
* Manual approval for private fields
* Redaction rules
* Asset permissions
* Audit logs for profile access
* Explicit profile linking to workspaces when needed

Example:

```json
{
  "name": "Personal Profile",
  "type": "person",
  "visibility": "private",
  "mcpExposure": "manual_approval",
  "fields": {
    "fullName": {
      "value": "Example User",
      "sensitive": false
    },
    "phoneNumber": {
      "value": "+63...",
      "sensitive": true
    }
  }
}
```

## Security Model

WorkGrid Memory must treat workspace files and profile memory as private by default.

Security rules:

* Do not expose profile data to MCP unless allowed.
* Do not expose sensitive fields automatically.
* Do not mix workspace memory across projects.
* Do not send local data to cloud providers unless explicitly configured.
* Do not index secret values from `.env` files.
* Only index environment variable keys, not values.
* Require user-approved workspace folders.
* Restrict file access to approved paths.
* Bind local HTTP server to `127.0.0.1`.
* Use local authentication tokens for HTTP MCP access.
* Keep MCP tools read-only in the first versions.
* Log MCP access for review.

## Suggested Local Storage Layout

```text
WorkGrid Memory/
  workspaces/
    workspace-id/
      metadata.sqlite
      vectors/
      graph.sqlite
      logs/
      cache/
      config.json

  profiles/
    profiles.sqlite
    assets/
      profile-images/
      documents/
      attachments/

  mcp/
    tokens.sqlite
    access-log.sqlite

  settings.json
```

## Workspace Data Model

Core tables:

```sql
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  git_remote TEXT,
  created_at TEXT,
  updated_at TEXT
);

CREATE TABLE files (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  path TEXT NOT NULL,
  language TEXT,
  hash TEXT NOT NULL,
  size INTEGER,
  mtime TEXT,
  indexed_at TEXT,
  ignored INTEGER DEFAULT 0
);

CREATE TABLE chunks (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  file_id TEXT NOT NULL,
  symbol_id TEXT,
  chunk_type TEXT NOT NULL,
  content TEXT NOT NULL,
  start_line INTEGER,
  end_line INTEGER,
  token_count INTEGER,
  hash TEXT NOT NULL
);

CREATE TABLE symbols (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  file_id TEXT NOT NULL,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  signature TEXT,
  doc TEXT,
  start_line INTEGER,
  end_line INTEGER,
  parent_symbol_id TEXT
);

CREATE TABLE edges (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  edge_type TEXT NOT NULL,
  confidence REAL DEFAULT 1.0
);
```

## Profile Data Model

Core tables:

```sql
CREATE TABLE profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  description TEXT,
  visibility TEXT DEFAULT 'private',
  mcp_exposure TEXT DEFAULT 'disabled',
  created_at TEXT,
  updated_at TEXT
);

CREATE TABLE profile_fields (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  field_value TEXT,
  field_type TEXT DEFAULT 'text',
  sensitive INTEGER DEFAULT 0,
  mcp_allowed INTEGER DEFAULT 0,
  created_at TEXT,
  updated_at TEXT
);

CREATE TABLE profile_assets (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  path TEXT NOT NULL,
  description TEXT,
  sensitive INTEGER DEFAULT 1,
  mcp_allowed INTEGER DEFAULT 0,
  created_at TEXT
);

CREATE TABLE profile_links (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  linked_type TEXT NOT NULL,
  linked_id TEXT NOT NULL,
  relationship TEXT,
  created_at TEXT
);
```

## Retrieval Strategy

WorkGrid Memory should use hybrid retrieval.

For workspace memory:

```text
final_score =
  vector_score * 0.40 +
  keyword_score * 0.25 +
  symbol_score * 0.20 +
  graph_score * 0.10 +
  recency_score * 0.05
```

For profile memory:

```text
final_score =
  profile_match_score * 0.35 +
  field_match_score * 0.25 +
  task_relevance_score * 0.25 +
  explicit_link_score * 0.15
```

For combined context:

```text
1. Determine the user/task intent
2. Search the active workspace
3. Search relevant global profiles
4. Apply privacy filters
5. Remove sensitive fields unless explicitly allowed
6. Build a compact context pack
7. Return evidence, summaries, and warnings
```

## Example Combined Context Flow

Task:

> Generate a commit message for this workspace.

WorkGrid Memory should retrieve:

* Git diff summary
* Recently changed files
* Relevant workspace symbols
* User commit style profile
* Project naming conventions
* Any workspace-specific contribution rules

Then return:

```json
{
  "task": "generate_commit_message",
  "workspaceContext": {
    "changedFiles": [
      "src/indexer/FileScanner.ts",
      "src/mcp/tools/searchWorkspace.ts"
    ],
    "summary": "Updated file scanning and MCP search behavior."
  },
  "profileContext": {
    "commitStyle": "User prefers concise conventional commits."
  },
  "suggestedOutput": "feat(indexer): improve workspace scanning and MCP search"
}
```

## Development Roadmap

### Phase 1: Desktop Shell

* Create Tauri + React app
* Add workspace picker
* Store workspace list
* Add basic settings page
* Add local app data directory
* Add logs

### Phase 2: Workspace Indexing

* Add file scanner
* Add ignore matcher
* Add file hashing
* Add SQLite metadata
* Add file list UI
* Add manual reindex command

### Phase 3: Search Memory

* Add chunking
* Add SQLite FTS search
* Add vector storage
* Add embedding provider
* Add search UI
* Add result previews with file paths and line ranges

### Phase 4: Code Intelligence

* Add Tree-sitter parsing
* Extract functions, classes, methods, interfaces, routes, configs, and database references
* Add symbol graph
* Add related file detection
* Add workspace architecture map

### Phase 5: Global Profile Memory

* Add profile database
* Add profile manager UI
* Add flexible profile types
* Add custom fields
* Add profile assets
* Add privacy controls
* Add profile search
* Add profile-to-workspace linking

### Phase 6: MCP Server

* Add local MCP server
* Add MCP on/off toggle
* Add stdio bridge CLI
* Add workspace tools
* Add profile tools
* Add context-pack builder
* Add access logs

### Phase 7: Hardening

* Add secret redaction
* Add profile exposure controls
* Add stale index warnings
* Add no-evidence responses
* Add backup/restore
* Add import/export
* Add test fixtures
* Add crash recovery

## Disaster Development Plan

Development tools fail. Rate limits happen. Agents break. APIs change. This is technology, so naturally everything is held together by invoices and hope.

WorkGrid Memory development should be resilient to tooling failures.

### If Codex Is Rate-Limited

Fallback plan:

1. Switch high-level planning tasks to Claude.
2. Switch repetitive coding tasks to CodeWhale or another local/cheaper coding agent.
3. Use smaller task prompts.
4. Split implementation into small isolated files.
5. Commit after each working milestone.
6. Avoid asking one agent to rewrite large modules.
7. Use manual implementation for critical architecture decisions.
8. Keep `IMPLEMENTATION.md` updated so any agent can resume work.

### If Claude Is Unavailable

Fallback plan:

1. Use Codex for code edits.
2. Use CodeWhale for local code navigation.
3. Use DeepSeek or Qwen through an OpenAI-compatible provider for implementation help.
4. Avoid architecture rewrites.
5. Continue using the project task checklist manually.

### If CodeWhale Fails or Lacks Plugin Support

Fallback plan:

1. Use it only as a code-generation helper.
2. Provide it with copied context from `DESIGN.md` and `IMPLEMENTATION.md`.
3. Use WorkGrid Memory itself later as the context provider.
4. Avoid relying on custom plugin behavior.
5. Keep prompts small and file-specific.

### If Embedding Provider Fails

Fallback plan:

1. Disable semantic search temporarily.
2. Continue using SQLite FTS keyword search.
3. Queue embedding jobs for retry.
4. Mark vector index as stale.
5. Keep workspace metadata and symbol search working.

### If Tree-sitter Parser Fails

Fallback plan:

1. Fall back to line-based chunking.
2. Fall back to regex-based symbol extraction.
3. Mark symbol confidence as low.
4. Keep indexing the file as searchable text.
5. Log parser errors for later language support fixes.

### If MCP Server Fails

Fallback plan:

1. Keep desktop search working.
2. Restart MCP server process.
3. Regenerate MCP config.
4. Show logs in the UI.
5. Allow exporting context manually as JSON or Markdown.

### If Profile Retrieval Becomes Noisy

Fallback plan:

1. Require explicit profile linking.
2. Reduce automatic profile retrieval.
3. Add relevance thresholds.
4. Hide sensitive profiles by default.
5. Allow users to disable global profile context per workspace.

### If Profile Data Is Accidentally Too Sensitive

Fallback plan:

1. Default all sensitive fields to MCP-disabled.
2. Add emergency “disable all profile MCP access” switch.
3. Add audit log review.
4. Require manual approval before exposing private profile fields.
5. Add redaction previews before context is sent to agents.

## Current Status

WorkGrid Memory is currently in design/planning stage.

Planned MVP:

* Tauri desktop app
* Workspace manager
* Local workspace indexing
* SQLite metadata
* Keyword search
* Vector search
* Basic symbol extraction
* Global profile manager
* MCP server toggle
* Workspace search MCP tool
* Profile search MCP tool
* Context-pack builder

## License

[MIT License](./LICENSE)

## Author

Created by [Jericho Aquino](https://github.com/eru123).

## Project Goal

WorkGrid Memory exists to give AI agents durable, grounded, local context across both codebases and reusable real-world knowledge.

It is a memory layer for development, workflows, profiles, and project intelligence.

The ambition is simple:

> Make AI agents work with actual context instead of guessing from scraps.
