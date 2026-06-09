# WorkGrid Memory — DESIGN.md

Status: Draft  
Product type: Local-first desktop application  
Primary stack: React + Tauri + Rust + SQLite + LanceDB + MCP  
Primary goal: Give AI coding agents grounded, cited, workspace-aware, and profile-aware context so they hallucinate less and waste fewer tokens wandering through files like a lost tourist with sudo access.

---

## 1. Vision

WorkGrid Memory is a desktop application that builds a local, persistent, searchable memory for developer workspaces and reusable real-world profiles.

It scans project folders, indexes file contents, maps code structure, extracts symbols such as functions, classes, methods, routes, models, configs, and database references, then exposes that knowledge to external AI agents through a local MCP server.

It also maintains a global profile memory layer for anything the user wants to define and reuse as context: people, pets, places, objects, products, ideas, processes, jobs, personal work habits, commit preferences, writing style, client preferences, and other definable entities. This profile layer is intentionally separate from workspace indexes. Workspace data is workspace-only. Profiles are global context and may be selectively attached to workspace context when useful and permitted.

The dream version is not just a vector database. A vector database alone can find “similar-looking text,” which is useful until it confidently retrieves a README from 2021 and calls it architecture. WorkGrid Memory should combine:

- Vector search for semantic retrieval
- Full-text search for exact terms
- Symbol extraction for code structure
- Graph mapping for file relationships
- MCP tools for agent access
- Global profile memory for people, pets, places, objects, products, ideas, processes, preferences, and skills
- Evidence-first responses with file paths, line ranges, profile sources, and confidence

The product promise:

> WorkGrid Memory gives coding agents reliable, local, cited context from your actual workspace, plus permissioned global profiles that explain the user, their preferences, their processes, and the real-world entities that matter to their work.

---

## 2. Product Definition

WorkGrid Memory is a local desktop memory manager for software projects.

A user adds one or more workspaces. For each workspace, the app creates and maintains a local index. Agents can then call the local MCP server to retrieve relevant code context without needing to load the entire project into the model context window.

The same app also lets the user create and maintain global profiles. A profile is a structured memory object that can describe almost anything: a person, pet, place, product, idea, process, project convention, commit style, work routine, client, job, asset, or domain concept. Profiles can be searched, related to each other, and exposed through MCP as general context when the user allows it.

The app is designed for developers who use tools such as Codex, Claude, CodeWhale, Cline, Continue, local agents, or other MCP-compatible clients.

---

## 3. Core Design Principles

### 3.1. Local-first

All workspace indexes are stored locally by default. Source code, embeddings, symbols, and metadata should not leave the machine unless the user explicitly configures a remote embedding provider or cloud sync later.

### 3.2. Evidence-first

Every retrieval result must include:

- Workspace ID
- File path
- Line range
- Symbol name when available
- Score/confidence
- Reason for retrieval
- Snippet or summary

No anonymous “the code probably does X” nonsense. If WorkGrid Memory cannot find evidence, it should say so.

### 3.3. Hybrid retrieval

Code search cannot rely only on vector similarity. The retrieval layer must combine:

- Semantic vector search
- SQLite FTS keyword search
- Symbol table lookup
- Graph proximity
- Git recency metadata
- Optional language intelligence later

### 3.4. Read-only first

The MCP server should begin with read-only tools. Editing, deleting, refactoring, and command execution are not part of the MVP. Agents are reckless enough with text. Do not give them a chainsaw on day one.

### 3.5. Multi-workspace by design

Unlike a VS Code extension trapped inside one editor window, WorkGrid Memory should manage many workspaces from one desktop app.

Example:

```text
~/www/sapi
~/projects/coco-lang
~/www/accountinglabs
~/clients/school-ims
```

Each workspace has its own index, settings, and MCP exposure policy.

### 3.6. Global profiles, separate from workspace memory

Workspace indexes must remain scoped to their workspace. A workspace index should not silently absorb unrelated personal or global information.

Profiles live in a global profile memory layer. They may be used across workspaces only when:

- The profile is enabled for MCP exposure
- The requesting tool has permission
- The profile is relevant to the request
- Sensitive fields are allowed for that context

This keeps project code and personal knowledge from melting into one privacy soup, which is exactly the kind of soup nobody ordered.

### 3.7. User-teachable behavior

WorkGrid Memory should let the user teach reusable behaviors, not just facts.

Examples:

- How the user wants commits written
- How the user structures branches
- How the user reviews pull requests
- How the user prefers documentation
- How the user usually works
- What tone to use for clients
- What conventions are used across personal projects

These should be stored as profile instructions or memory skills. Agents can request them through MCP when they need behavioral guidance.

---

## 4. Non-goals for MVP

The first version should not attempt:

- Cloud sync
- Team sharing
- Remote indexing
- Multi-user permissions
- Editing tools over MCP
- Full call graph support for every language
- Complex visual architecture graph UI
- Custom LLM agent chat UI
- Training or fine-tuning models
- Building a proprietary embedding model
- Automatic surveillance of people or relationships
- Biometric identification of unknown people
- Exposing sensitive personal profiles to MCP by default
- Turning profiles into a social network, because apparently one layer of chaos is enough

These can happen later, after the boring useful parts work. Software products die when the homepage gets finished before the indexer.

---

## 5. High-Level Architecture

```text
WorkGrid Memory Desktop App
│
├─ React Frontend
│  ├─ Workspace dashboard
│  ├─ Profile manager
│  ├─ Search interface
│  ├─ Context pack preview
│  ├─ MCP server controls
│  ├─ Index job monitor
│  ├─ Logs and diagnostics
│  └─ Settings
│
├─ Tauri / Rust Core
│  ├─ Native filesystem access
│  ├─ Process management
│  ├─ Secure app storage paths
│  ├─ Sidecar lifecycle management
│  ├─ Profile asset access controls
│  └─ Commands exposed to React
│
├─ WorkGrid Memory Engine
│  ├─ Workspace scanner
│  ├─ File watcher
│  ├─ Ignore matcher
│  ├─ Hashing and change detection
│  ├─ Chunker
│  ├─ Parser and symbol extractor
│  ├─ Profile manager
│  ├─ Profile instruction resolver
│  ├─ Embedding queue
│  ├─ Graph builder
│  ├─ Context pack builder
│  └─ Retrieval engine
│
├─ Storage Layer
│  ├─ SQLite app database
│  ├─ SQLite workspace metadata databases
│  ├─ SQLite FTS keyword indexes
│  ├─ LanceDB workspace vector stores
│  ├─ LanceDB profile vector store
│  ├─ Profile asset store
│  └─ Graph tables
│
└─ MCP Layer
   ├─ Local Streamable HTTP MCP server
   ├─ stdio bridge CLI
   ├─ Workspace tool registry
   ├─ Profile tool registry
   ├─ Resource registry
   └─ Security and permission checks
```

---

## 6. Recommended Process Layout

The cleanest design is to split the product into three executables or logical modules:

```text
1. workgrid-memory
   The Tauri desktop GUI.

2. workgrid-memory-engine
   The indexing, retrieval, and storage engine.

3. workgrid-memory-mcp
   The MCP server or stdio bridge used by external agents.
```

In development, these may live in one monorepo. In production, they may be bundled as sidecars.

Tauri supports bundling external binaries through `externalBin`, which makes this layout practical without forcing users to install Node.js, Python, or other dependencies manually.

---

## 7. Technology Stack

### 7.1. Desktop App

- React
- TypeScript
- Tauri v2
- Rust backend commands
- Tailwind CSS or equivalent utility styling
- Zustand or TanStack Store for frontend state

### 7.2. Engine

Preferred starting option:

- Rust for file scanning, hashing, watching, storage access, and process control
- TypeScript sidecar only if MCP or LanceDB integration is easier initially

Acceptable practical compromise:

- Tauri/Rust app controls a Node/TypeScript sidecar engine
- The sidecar handles MCP, LanceDB TypeScript SDK, and embeddings
- Rust handles app lifecycle, file permissions, and native shell/process management

### 7.3. Storage

- SQLite for metadata, jobs, chunks, symbols, graph edges, profiles, profile attributes, profile relationships, and FTS
- LanceDB for local vector storage
- Separate LanceDB collections for workspace chunks and global profile memory
- App data directory for all indexes, profile assets, and logs

### 7.4. Parsing

Start with:

- Plain text scanner
- Language-specific chunking
- Tree-sitter-based symbol extraction

Initial languages:

- TypeScript
- JavaScript
- TSX
- JSX
- PHP
- JSON
- Markdown
- YAML
- SQL

Later:

- Python
- Go
- Rust
- Java
- C#
- Vue
- Svelte

### 7.5. Embeddings

Default local provider:

- Ollama with `nomic-embed-text` or equivalent local embedding model

Optional providers:

- OpenAI-compatible embedding endpoint
- Voyage
- Cloud embedding services
- Custom HTTP embedding provider

Important rule:

Do not mix embeddings from different models in the same vector collection. Store embedding provider, model, and dimensions per workspace index version.

---

## 8. Workspace Model

A workspace is a user-approved project folder.

Each workspace has:

- Stable workspace ID
- Display name
- Root path
- Git metadata when available
- Indexing settings
- Ignore settings
- MCP exposure settings
- Storage location
- Last indexed timestamp
- Health state

Workspace ID generation:

```text
hash(root_path + git_remote_url + created_at_seed)
```

Do not use only the folder name. Humans reuse folder names with the imagination of a broken photocopier.

---

## 9. Local Storage Layout

Suggested app data structure:

```text
WorkGrid Memory/
  app.sqlite
  profiles.sqlite
  profile-vectors.lance/
  profile-assets/
    images/
    documents/
    notes/
  logs/
    app.log
    mcp.log
    engine.log
    profiles.log

  workspaces/
    sapi-abcd1234/
      metadata.sqlite
      vectors.lance/
      cache/
      snapshots/
      logs/
      config.json

    coco-lang-efgh5678/
      metadata.sqlite
      vectors.lance/
      cache/
      snapshots/
      logs/
      config.json
```

`app.sqlite` stores global app configuration and the workspace registry.

`profiles.sqlite` stores global profiles, profile attributes, profile relationships, profile permissions, and profile audit metadata.

Each workspace owns its own metadata and vector database. Profiles do not live inside workspace databases.
---

## 10. Global Profile Memory Model

A profile is a global structured memory object. It represents anything with definable characteristics.

Supported profile categories should be user-extensible, but the first set should include:

- Person
- Pet or animal
- Place
- Object
- Product
- Organization
- Job or role
- Project idea
- Process
- Habit or routine
- Preference
- Skill or instruction
- Client
- Asset
- Domain concept

Examples:

```text
Person profile:
  Name: Aibie
  Relationship: Wife
  Notes: User-provided personal context
  Assets: reference photos, documents, notes
  Sensitivity: private

Pet profile:
  Name: Mochi
  Species: Cat
  Traits: shy, indoor, likes dry food
  Sensitivity: private

Commit style profile:
  Name: Jericho Commit Style
  Type: Skill / Instruction
  Rules:
    - Use concise imperative commits
    - Mention affected module when useful
    - Avoid vague messages like "fix stuff"
    - Prefer format: type(scope): summary
  Trigger:
    - commit
    - git
    - changelog
    - release notes

Process profile:
  Name: Jericho Development Workflow
  Type: Process
  Steps:
    - inspect existing pattern
    - make small patch
    - run typecheck/tests
    - update docs if behavior changes
```

Profiles are not workspace data. They live globally and may support any workspace when relevant.

### 10.1. Profile scope rules

```text
Workspace memory:
  Belongs to one workspace only.
  Contains source files, symbols, chunks, project metadata, and workspace graph.

Global profiles:
  Belong to the user/app globally.
  Contain reusable context, preferences, facts, assets, processes, and instructions.
  Can be attached to MCP context packs only when relevant and permitted.
```

A profile may be manually linked to one or more workspaces, but the profile remains global. Linking means "this profile is relevant here," not "copy this profile into the workspace index."

### 10.2. Profile anatomy

Each profile should support:

- Stable profile ID
- Display name
- Profile type
- Description
- Attributes
- Notes
- Instructions or rules
- Tags
- Sensitivity level
- Source metadata
- Assets such as images or documents
- Relationships to other profiles
- Workspace relevance links
- Embedding records
- Last reviewed timestamp
- MCP exposure policy

### 10.3. Profile attributes

Attributes should be flexible, not hardcoded into one giant table designed by someone allergic to future requirements.

Examples:

```json
{
  "profileType": "person",
  "attributes": {
    "relationship": "wife",
    "preferredName": "Aibie",
    "notes": "User-provided context only",
    "importantDates": [],
    "communicationPreference": "private"
  }
}
```

```json
{
  "profileType": "skill",
  "attributes": {
    "domain": "git",
    "triggerTerms": ["commit", "git commit", "release notes"],
    "rules": [
      "Use imperative mood",
      "Keep summary under 72 characters when possible",
      "Mention module scope when useful"
    ]
  }
}
```

### 10.4. Profile assets

Profiles may contain assets:

- Images
- Documents
- Notes
- Audio references later
- External links
- Local file references

For personal photos, the app should store them as private user-provided assets. The system should not market itself as identifying unknown people. It stores known, user-approved context. That distinction matters unless one enjoys building a privacy lawsuit generator.

### 10.5. Profile relationships

Profile graph relationships may include:

```text
related_to
owned_by
works_at
friend_of
spouse_of
pet_of
located_at
prefers
uses
belongs_to
teaches
applies_to
relevant_to_workspace
conflicts_with
supersedes
```

Examples:

```text
Jericho Development Workflow teaches WorkGrid Memory agents how the user works.
Jericho Commit Style applies_to all coding workspaces unless overridden.
Client X Billing Preference relevant_to_workspace /clients/client-x-billing.
Pet profile belongs_to personal profiles, not software workspaces by default.
```

### 10.6. Profile instruction and skill system

Some profiles behave like reusable skills. These are procedural memories, not just facts.

Skill-like profiles should have:

- Trigger terms
- Scope
- Priority
- Rules
- Examples
- Anti-patterns
- When not to apply
- Last reviewed timestamp

Examples:

```text
Commit Style Skill
Code Review Skill
Client Message Style Skill
Debugging Workflow Skill
Documentation Style Skill
Deployment Checklist Skill
```

The context pack builder can include these skills when a request matches their trigger terms.

### 10.7. Profile privacy and sensitivity

Profiles should support sensitivity levels:

```text
public
internal
private
sensitive
secret
```

Default recommendations:

- People profiles: private by default
- Photos: sensitive by default
- Personal contact information: sensitive by default
- Credentials or tokens: secret and never exposed through MCP
- Work preferences: internal or private
- Generic coding style profiles: internal
- Client information: private or sensitive

MCP exposure must be disabled by default for sensitive profiles. The user must opt in.

### 10.8. Profile retrieval behavior

Profiles should be retrieved using hybrid search:

- Exact name/tag search
- Attribute search
- Vector search over descriptions, notes, and instructions
- Relationship graph expansion
- Workspace relevance links
- Sensitivity and permission filtering

Context packs should separate workspace evidence from profile context:

```json
{
  "workspaceContext": [
    {
      "file": "src/auth/login.ts",
      "lines": "20-70",
      "reason": "Login validation implementation"
    }
  ],
  "profileContext": [
    {
      "profile": "Jericho Commit Style",
      "type": "skill",
      "reason": "Request is about commit message generation",
      "sensitivity": "internal"
    }
  ]
}
```

Agents must be able to see which context came from code and which came from profiles. Mixing them without labels would be a fine way to manufacture nonsense with confidence.

---

## 11. Metadata Schema

Minimum SQLite schema:

```sql
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  git_remote TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_indexed_at TEXT,
  status TEXT NOT NULL DEFAULT 'new'
);

CREATE TABLE files (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  path TEXT NOT NULL,
  language TEXT,
  hash TEXT NOT NULL,
  size INTEGER NOT NULL,
  mtime TEXT,
  indexed_at TEXT,
  ignored INTEGER DEFAULT 0,
  deleted INTEGER DEFAULT 0
);

CREATE TABLE chunks (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  file_id TEXT NOT NULL,
  symbol_id TEXT,
  chunk_type TEXT NOT NULL,
  content TEXT NOT NULL,
  start_line INTEGER NOT NULL,
  end_line INTEGER NOT NULL,
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
  start_line INTEGER NOT NULL,
  end_line INTEGER NOT NULL,
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

CREATE TABLE index_jobs (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  job_type TEXT NOT NULL,
  status TEXT NOT NULL,
  total_items INTEGER DEFAULT 0,
  processed_items INTEGER DEFAULT 0,
  error TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT
);

CREATE VIRTUAL TABLE chunks_fts USING fts5(
  content,
  file_path,
  symbol_name
);
```

Global profile schema, stored in `profiles.sqlite` or `app.sqlite` depending on final storage split:

```sql
CREATE TABLE profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  profile_type TEXT NOT NULL,
  description TEXT,
  sensitivity TEXT NOT NULL DEFAULT 'private',
  mcp_exposure TEXT NOT NULL DEFAULT 'disabled',
  source TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_reviewed_at TEXT,
  archived INTEGER DEFAULT 0
);

CREATE TABLE profile_attributes (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  sensitivity TEXT,
  source TEXT,
  confidence REAL DEFAULT 1.0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE profile_instructions (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  name TEXT NOT NULL,
  trigger_terms_json TEXT,
  rules_json TEXT NOT NULL,
  examples_json TEXT,
  anti_patterns_json TEXT,
  priority INTEGER DEFAULT 100,
  enabled INTEGER DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE profile_assets (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  local_path TEXT NOT NULL,
  hash TEXT,
  description TEXT,
  sensitivity TEXT NOT NULL DEFAULT 'sensitive',
  created_at TEXT NOT NULL
);

CREATE TABLE profile_relationships (
  id TEXT PRIMARY KEY,
  from_profile_id TEXT NOT NULL,
  to_profile_id TEXT NOT NULL,
  relationship_type TEXT NOT NULL,
  confidence REAL DEFAULT 1.0,
  source TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE profile_workspace_links (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  relevance TEXT,
  enabled INTEGER DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE TABLE profile_audit_log (
  id TEXT PRIMARY KEY,
  profile_id TEXT,
  action TEXT NOT NULL,
  actor TEXT NOT NULL,
  detail_json TEXT,
  created_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE profiles_fts USING fts5(
  name,
  profile_type,
  description,
  tags,
  searchable_text
);
```

---

## 12. Indexing Pipeline

Initial indexing:

```text
1. User adds workspace.
2. App validates the path.
3. App creates workspace storage.
4. Scanner reads file tree.
5. Ignore matcher excludes unwanted paths.
6. Hasher computes file hashes.
7. Parser extracts symbols and structural blocks.
8. Chunker creates meaningful chunks.
9. Metadata is written to SQLite.
10. FTS index is updated.
11. Embedding queue processes chunks.
12. Vectors are written to LanceDB.
13. Graph builder creates relationships.
14. Health check validates index consistency.
15. Dashboard updates workspace status.
```

Incremental indexing:

```text
on file created:
  queue file for indexing

on file changed:
  debounce
  hash file
  if hash changed:
    reparse
    update chunks
    update symbols
    update vectors
    update graph edges

on file deleted:
  mark file deleted
  remove active chunks
  remove active vectors
  remove graph edges

on file renamed:
  treat as delete + create unless file hash confirms move
```

Profile indexing:

```text
on profile created or updated:
  validate schema
  classify sensitivity
  redact secret fields from searchable text
  update FTS index
  generate profile embedding when allowed
  update profile graph relationships
  write audit log

on profile asset added:
  hash asset
  store under profile-assets
  extract safe metadata only
  mark asset sensitivity
  never expose raw sensitive asset through MCP unless explicitly permitted
```

---

## 13. Ignore Rules

WorkGrid Memory should support:

- `.gitignore`
- `.workgridignore`
- App-level ignore patterns
- Per-workspace ignore patterns

Default ignores:

```text
.git/**
node_modules/**
vendor/**
dist/**
build/**
coverage/**
.cache/**
.next/**
.nuxt/**
target/**
storage/logs/**
*.lock
*.map
*.min.js
*.png
*.jpg
*.jpeg
*.gif
*.webp
*.ico
*.pdf
*.zip
*.tar
*.gz
.env
.env.*
```

Secret files should be ignored or redacted by default.

`.env` handling:

- Index keys only if enabled
- Never index values
- Redact obvious secrets

Example:

```text
DB_HOST=<redacted>
OPENAI_API_KEY=<redacted>
AWS_SECRET_ACCESS_KEY=<redacted>
```

---

## 14. Chunking Strategy

Do not chunk code like prose.

Preferred chunk order:

```text
1. Symbol chunk
   Function, method, class, interface, type, route handler

2. Structural chunk
   Config object, SQL migration, model schema, route group

3. Documentation chunk
   Markdown heading section, docblock, README section

4. Fallback chunk
   Line-based chunk when parsing fails
```

Each chunk must store:

- Content
- File path
- Start line
- End line
- Symbol ID when available
- Chunk type
- Content hash
- Token estimate

Chunk size targets:

- Small function: keep whole
- Large function: split by logical blocks
- Large class: split by methods
- Markdown: split by headings
- SQL: split by statements or migration blocks

---

## 15. Symbol Extraction

Initial symbols:

- Function
- Class
- Method
- Interface
- Type
- Constant
- Import/export
- Route
- Controller
- Model
- Migration
- Database table
- Environment variable key
- Config key

Symbol metadata:

- Name
- Kind
- Signature
- File
- Start line
- End line
- Parent symbol
- Docblock/comment
- Visibility when available
- Language

---

## 16. Graph Model

Graph edges should represent relationships between files, chunks, and symbols.

Edge types:

```text
imports
exports
calls
references
defines
extends
implements
uses_env
queries_table
handles_route
belongs_to_file
near_symbol
tested_by
configures
```

The graph does not need to be perfect in MVP. It must be useful enough to answer:

- What files are related to this file?
- Where is this function defined?
- What calls this method?
- What model or table does this route touch?
- What config/env values does this feature depend on?
- Which tests probably cover this code?

---

## 17. Retrieval Pipeline

When an agent asks a question, WorkGrid Memory should retrieve context using a hybrid pipeline.

Example query:

```text
Where is invoice total calculated?
```

Pipeline:

```text
1. Normalize query.
2. Extract possible symbols and keywords.
3. Run keyword/FTS search.
4. Run vector search.
5. Run symbol table lookup.
6. Expand graph around strong matches.
7. Merge and deduplicate results.
8. Rerank results.
9. Build compact context pack.
10. Return evidence with file paths and line numbers.
```

Suggested scoring:

```text
final_score =
  0.40 * vector_score +
  0.25 * keyword_score +
  0.20 * symbol_score +
  0.10 * graph_proximity_score +
  0.05 * recency_score
```

The formula can evolve later. For MVP, it just needs to be consistent, explainable, and testable.

Profile-aware context packing:

```text
1. Run workspace retrieval first.
2. Detect whether the request needs global profile context.
3. Search eligible profiles by trigger terms, tags, attributes, and embeddings.
4. Filter by sensitivity and MCP permissions.
5. Add only the minimum relevant profile context.
6. Label profile context separately from workspace evidence.
7. Prefer explicit user/workspace instructions over generic profiles.
8. Report when profile context was withheld because of permissions.
```

Examples:

```text
Query: "Create a commit message for this change"
Workspace context: changed files, symbols, diff summary
Profile context: Jericho Commit Style skill

Query: "Explain this auth controller"
Workspace context: auth files and symbols
Profile context: none, unless a relevant workspace-specific profile is linked

Query: "Write a client update for this school IMS project"
Workspace context: affected project files
Profile context: client communication style, school/client profile, user's preferred tone
```

---

## 18. MCP Server Design

WorkGrid Memory should expose a local MCP server.

Primary transport:

```text
Streamable HTTP
http://127.0.0.1:3847/mcp
```

Compatibility transport:

```text
stdio bridge CLI
workgrid-memory-mcp
```

Why both?

- Streamable HTTP fits a running desktop app.
- stdio fits MCP clients that expect to launch a server subprocess.
- The stdio bridge can connect to the running app or read the local index directly.

---

## 19. MCP Tools

### 19.1. `search_workspace`

Search code using hybrid retrieval.

Input:

```json
{
  "workspace": "sapi",
  "query": "where is login validation handled?",
  "topK": 8,
  "includeSnippets": true
}
```

Output:

```json
{
  "status": "ok",
  "results": [
    {
      "file": "app/Controllers/AuthController.php",
      "symbol": "AuthController::login",
      "startLine": 42,
      "endLine": 91,
      "score": 0.91,
      "reason": "Route handler validates credentials and creates a session.",
      "snippet": "..."
    }
  ]
}
```

### 19.2. `get_file_context`

Returns:

- File summary
- Language
- Symbols
- Imports/exports
- Related files
- Relevant chunks
- Last indexed timestamp

### 19.3. `explain_symbol`

Returns:

- Definition
- Signature
- File and line range
- Docblock/comment
- Callers/callees when available
- Related chunks
- Confidence

### 19.4. `find_references`

Returns references using:

- Symbol graph
- Exact text search
- Import/export analysis
- Optional language server data later

### 19.5. `get_related_files`

Returns files related by:

- Imports
- Route/controller/model relationships
- Shared database tables
- Shared configs
- Test/source mapping

### 19.6. `get_workspace_map`

Returns a compact project map:

- Framework
- Main entry points
- Routes
- Controllers
- Services
- Models
- Database layer
- Frontend pages/components
- Jobs/queues
- Config files
- Test folders

### 19.7. `verify_claim`

Checks whether a claim is supported by indexed evidence.

Input:

```json
{
  "workspace": "sapi",
  "claim": "The app uses JWT for authentication"
}
```

Output:

```json
{
  "verdict": "supported | contradicted | insufficient_evidence",
  "confidence": 0.86,
  "evidence": [
    {
      "file": "app/Auth/AuthService.php",
      "lines": "20-44",
      "summary": "Authentication uses session cookies, not JWT."
    }
  ]
}
```

This is one of the most important tools in the product. It turns WorkGrid Memory from “search app” into “anti-hallucination evidence engine.”

### 19.8. `search_profiles`

Search global profiles using name, type, attributes, instructions, tags, and embeddings.

Input:

```json
{
  "query": "how does Jericho want commits written?",
  "types": ["skill", "preference"],
  "includeSensitive": false
}
```

Output:

```json
{
  "status": "ok",
  "results": [
    {
      "profileId": "profile-commit-style",
      "name": "Jericho Commit Style",
      "type": "skill",
      "sensitivity": "internal",
      "reason": "Triggered by commit-related request",
      "summary": "Use concise imperative commits with optional module scope."
    }
  ]
}
```

### 19.9. `get_profile_context`

Returns a specific profile context pack.

The tool must respect sensitivity, field-level permissions, and MCP exposure policy. It should never dump an entire sensitive profile unless the user explicitly allowed that scope.

Returns:

- Profile summary
- Type
- Relevant attributes
- Relevant instructions
- Related profiles
- Workspace links
- Sensitivity labels
- Source/audit metadata when useful

### 19.10. `get_relevant_profiles`

Given a workspace and task, returns only the profiles that should be considered.

Example:

```json
{
  "workspace": "sapi",
  "task": "generate commit message for current changes"
}
```

Possible output:

```json
{
  "profiles": [
    {
      "name": "Jericho Commit Style",
      "type": "skill",
      "reason": "Task involves commit generation"
    },
    {
      "name": "Jericho Development Workflow",
      "type": "process",
      "reason": "Task involves coding workflow"
    }
  ]
}
```

### 19.11. `build_context_pack`

Builds a merged but clearly separated context pack from workspace memory and global profiles.

Input:

```json
{
  "workspace": "sapi",
  "task": "prepare commit message and summarize affected billing changes",
  "includeWorkspace": true,
  "includeProfiles": true
}
```

Output must separate context classes:

```json
{
  "workspaceContext": [],
  "profileContext": [],
  "withheldProfileContext": [],
  "warnings": []
}
```

This tool is the safest default for agents because it controls how much profile information is included instead of letting agents rummage around like raccoons in a filing cabinet.

---

## 20. MCP Resources

Suggested resource URIs:

```text
workgrid://workspace/{workspaceId}
workgrid://workspace/{workspaceId}/map
workgrid://workspace/{workspaceId}/file/{path}
workgrid://workspace/{workspaceId}/symbol/{symbolId}
workgrid://workspace/{workspaceId}/recent-changes
workgrid://profile/{profileId}
workgrid://profile/{profileId}/instructions
workgrid://profile/{profileId}/relationships
workgrid://profiles/search/{query}
workgrid://context-pack/{workspaceId}/{taskHash}
```

Resources should expose stable, read-only, structured context.

---

## 21. Security Model

### 21.1. Workspace access

The app may only index user-approved workspace roots.

### 21.2. Path safety

All file access must:

- Resolve canonical paths
- Block path traversal
- Block symlink escapes unless explicitly allowed
- Reject access outside workspace root

### 21.3. Secret handling

Default behavior:

- Ignore known secret files
- Redact obvious secret values
- Never send secret values to embedding providers
- Never expose secret values over MCP

### 21.4. MCP security

For HTTP MCP:

- Bind only to `127.0.0.1`
- Require local token
- Rotate token on demand
- Show active server status in UI
- Allow per-workspace exposure toggle
- Keep tools read-only by default


### 21.5. Profile security

Profile security must be stricter than workspace security because profiles can contain personal information, photos, family details, client context, preferences, and other sensitive records.

Rules:

- Profiles are global but not automatically exposed globally.
- MCP profile access is disabled by default.
- Sensitive profiles require explicit opt-in before MCP exposure.
- Field-level sensitivity must be respected.
- Photos and identity-related assets are sensitive by default.
- Secret fields are never embedded, logged, or exposed.
- Profile context must be minimized to what the task needs.
- Context packs must label profile context separately from workspace evidence.
- The app must provide audit logs for profile reads over MCP.

Personal profile data should be treated as user-owned local data, not training material, not telemetry, and definitely not a fun surprise for whatever agent connects next.

### 21.6. Logging

Logs must avoid writing full secrets, raw tokens, sensitive profile fields, raw tokens, and huge snippets.

---

## 22. UI Design

Main sections:

```text
Workspaces
Profiles
Search
Context Packs
MCP Server
Index Jobs
Settings
Logs
```

Workspace dashboard:

```text
Workspace: sapi
Path: /home/jericho/www/sapi
Status: Ready
Files indexed: 2,413
Symbols indexed: 18,902
Chunks indexed: 9,431
Vector DB size: 182 MB
Last indexed: 2026-06-09 23:12
MCP exposure: Enabled
```

Actions:

```text
Reindex
Pause indexing
Clear memory
Start MCP
Stop MCP
Copy MCP config
Open logs
```

Search page:

- Query input
- Workspace selector
- Filters
- Result list
- File path + line range
- Symbol badges
- Reason matched
- Snippet preview

MCP page:

- Server on/off
- HTTP endpoint
- stdio bridge config
- Connected clients
- Token management
- Workspace permissions
- Profile permissions
- Tool list

Profile manager:

```text
Profiles
  People
  Pets
  Places
  Objects
  Products
  Ideas
  Processes
  Skills / Instructions
  Clients
  Custom Types
```

Profile detail page:

```text
Profile: Jericho Commit Style
Type: Skill / Instruction
Sensitivity: Internal
MCP Exposure: Enabled
Triggers: commit, git, changelog, release notes
Linked Workspaces: All coding workspaces
Last Reviewed: 2026-06-09
```

Actions:

```text
Add profile
Add attribute
Add instruction
Attach asset
Link to workspace
Enable/disable MCP exposure
Review profile
Archive profile
```

Context pack preview:

- Workspace evidence
- Profile context
- Withheld sensitive context
- Permission warnings
- Final compact payload shown before MCP use when debugging

---

## 23. Settings

Suggested settings:

```json
{
  "indexing.autoIndex": true,
  "indexing.watchFiles": true,
  "indexing.maxFileSizeKb": 512,
  "indexing.exclude": [
    "node_modules/**",
    "vendor/**",
    "dist/**"
  ],
  "embeddings.provider": "ollama",
  "embeddings.model": "nomic-embed-text",
  "vectorDb.provider": "lancedb",
  "mcp.enabled": false,
  "mcp.transport": "http",
  "mcp.httpPort": 3847,
  "mcp.requireToken": true,
  "profiles.enabled": true,
  "profiles.mcpExposureDefault": "disabled",
  "profiles.embedSensitiveFields": false,
  "profiles.assets.enabled": true,
  "profiles.contextPack.maxProfiles": 5,
  "security.redactSecrets": true,
  "security.redactSensitiveProfiles": true,
  "retrieval.hybridSearch": true,
  "retrieval.profileAwareContext": true
}
```

---

## 24. Health and Diagnostics

Each workspace should report:

- Index status
- Last successful index
- Number of failed files
- Vector count vs chunk count
- Embedding provider status
- Database integrity check
- Profile database integrity check
- Profile vector count vs profile chunks
- MCP availability
- Profile MCP exposure status
- File watcher status

Health states:

```text
new
indexing
ready
degraded
stale
error
paused
```

---

## 25. Disaster and Degraded Modes

WorkGrid Memory should be designed to survive partial failure.

### 25.1. Embedding provider offline

Fallback:

- Keep metadata indexing
- Keep FTS search
- Mark vector search unavailable
- Queue embeddings for later

### 25.2. Vector DB corruption

Fallback:

- Preserve SQLite metadata
- Rebuild LanceDB from chunks
- Show warning
- Do not delete source metadata automatically

### 25.3. SQLite corruption

Fallback:

- Keep periodic snapshots
- Attempt recovery
- Rebuild from source files if needed
- Keep logs explaining what was lost

### 25.4. Parser crash

Fallback:

- Mark file parse failed
- Use fallback line-based chunking
- Continue indexing other files

### 25.5. Huge repo

Fallback:

- Ask user to confirm indexing
- Apply stricter ignore rules
- Limit max file size
- Batch jobs
- Show progress
- Allow pause/resume

### 25.6. MCP server failure

Fallback:

- Show error in UI
- Keep index available locally
- Allow restart
- Write server logs


### 25.7. Profile database corruption

Fallback:

- Preserve workspace indexes
- Mark profile memory degraded
- Attempt profile DB recovery
- Restore from profile snapshots when available
- Disable profile MCP tools until integrity is verified

### 25.8. Sensitive profile overexposure risk

Fallback:

- Disable profile MCP tools immediately
- Rotate MCP token
- Write audit log entry
- Show affected profiles and fields
- Require manual re-enable
- Keep workspace MCP tools available if safe

### 25.9. Conflicting or stale profiles

Fallback:

- Prefer recently reviewed profiles
- Show conflict warning
- Ask user to resolve in profile manager
- Let workspace-specific rules override generic global profiles
- Mark old profile as superseded instead of deleting it

### 25.10. Profile retrieval noise

Fallback:

- Limit number of profiles per context pack
- Require stronger trigger match for sensitive profiles
- Penalize weak vector-only matches
- Show why each profile was included
- Allow user to pin or ban profiles per workspace

---

## 26. Version Roadmap

### 26.1. MVP

- Add workspace
- Scan files
- Ignore rules
- SQLite metadata
- FTS search
- Local embeddings
- LanceDB vectors
- Basic semantic search
- Tree-sitter symbols for PHP and TS/JS
- Global profile registry
- Profile attributes and instructions
- Profile FTS search
- Basic profile-aware context packs
- MCP HTTP server
- stdio bridge
- `search_workspace`
- `get_file_context`
- `verify_claim`
- Dashboard

### 26.2. v0.2

- More language parsers
- Better route/model/config detection
- Improved graph edges
- Profile relationships
- Profile asset attachments
- Agent config generator
- Index health reports
- Better logs

### 26.3. v0.3

- Companion VS Code extension
- Companion CLI
- Workspace snapshots
- Profile import/export
- Import/export indexes
- Configurable reranking
- More MCP resources

### 26.4. v1.0

- Stable multi-workspace support
- Reliable incremental indexing
- Robust MCP compatibility
- Stable profile permissions and audit logs
- Installer packages
- Signed releases
- Documentation
- Test fixture suite

---

## 27. Reference Notes

These are the technical assumptions used by this design:

- MCP defines `stdio` and Streamable HTTP as standard transports.
- MCP tools use structured definitions with names, descriptions, input schemas, and optional structured output.
- Tauri supports bundling sidecar binaries through `externalBin`.
- Tauri filesystem access should be scoped through app permissions.
- Global profiles must be permissioned separately from workspace memory.
- Codex usage can be rate-limited depending on plan and task complexity, so development should not depend on Codex alone.

Reference URLs:

- https://modelcontextprotocol.io/specification/2025-06-18/basic/transports
- https://modelcontextprotocol.io/specification/2025-06-18/server/tools
- https://v2.tauri.app/develop/sidecar/
- https://v2.tauri.app/plugin/file-system/
- https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan
