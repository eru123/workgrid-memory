# WorkGrid Memory — IMPLEMENTATION.md

Status: Draft  
Purpose: Realistic implementation plan from zero to release  
Tone: Practical, staged, and resistant to agent chaos  
Primary stack: React + Tauri + Rust + SQLite + LanceDB + MCP

---

## 1. Implementation Goal

Build WorkGrid Memory as a local-first desktop app that indexes developer workspaces, creates searchable project memory, maintains global reusable profiles, and exposes grounded context to AI agents through MCP.

The implementation must prioritize reliability over spectacle. The final product should be useful even before it becomes beautiful.

The first stable release should support:

- Adding multiple workspaces
- Indexing files with ignore rules
- Creating chunks and metadata
- Running keyword search
- Running vector search
- Extracting common symbols
- Creating and searching global profiles
- Supporting profile instructions such as commit style and personal workflows
- Serving read-only MCP tools for workspace and profile context
- Returning cited evidence with file paths, line ranges, profile sources, and sensitivity labels
- Surviving common failures without deleting user data like a tiny disaster wearing an app icon

---

## 2. Development Rules

### 2.1. Main branch must stay runnable

Never merge directly into `main` unless:

- App boots
- Tests pass
- Type checking passes
- Existing indexed workspace can still be opened
- MCP server still starts or fails gracefully

### 2.2. Small tasks only

Agents should work on small scoped tasks. Do not ask an agent:

```text
Build the entire WorkGrid Memory app.
```

That is how you get 4,000 files, 11 package managers, and one spiritual crisis.

Preferred task shape:

```text
Implement SQLite workspace registry with tests.
```

### 2.3. Every agent task needs a handoff packet

Each task given to Codex, Claude, CodeWhale, or any other agent should include:

```text
Goal
Files allowed to edit
Files not allowed to edit
Expected output
Test command
Rollback instruction
Acceptance criteria
```

### 2.4. Prefer boring architecture

The app should be easy to debug:

- SQLite before complex graph DB
- Local file storage before sync
- Read-only MCP before write tools
- Workspace memory before profile-aware context merging
- Profile privacy before profile convenience
- FTS before advanced reranking
- One workspace index before multi-workspace optimization

---

## 3. Repository Structure

Recommended monorepo:

```text
workgrid-memory/
  README.md
  DESIGN.md
  IMPLEMENTATION.md
  package.json
  pnpm-workspace.yaml

  apps/
    desktop/
      src/
      src-tauri/
      package.json

  crates/
    engine/
      src/
      Cargo.toml

    mcp-server/
      src/
      Cargo.toml

    shared/
      src/
      Cargo.toml

  packages/
    ui/
      src/
      package.json

    schemas/
      src/
      package.json

    profile-schemas/
      src/
      package.json

  fixtures/
    profile-samples/
      people-sample.json
      pet-sample.json
      commit-style-sample.json
      workflow-sample.json

    php-laravel-sample/
    ts-react-sample/
    mixed-project-sample/

  docs/
    architecture/
    decisions/
    test-results/

  scripts/
    dev.sh
    test.sh
    package.sh
```

Alternative if using Node/TypeScript sidecar for MCP and LanceDB:

```text
apps/
  desktop/
  mcp-sidecar/
  engine-sidecar/
```

Keep the repository flexible until the Rust-vs-Node engine decision is proven by prototypes.

---

## 4. Phase 0 — Product and Technical Decisions

Goal: lock the minimum architecture before writing too much code.

Tasks:

- [ ] Confirm product name: WorkGrid Memory
- [ ] Confirm desktop-first approach: React + Tauri
- [ ] Confirm local-first storage
- [ ] Confirm initial operating systems
- [ ] Confirm MVP languages: PHP, TS, JS, Markdown, JSON, YAML, SQL
- [ ] Decide whether engine is Rust-first or Node sidecar-first
- [ ] Decide vector database MVP: LanceDB
- [ ] Decide metadata database: SQLite
- [ ] Decide embedding provider MVP: Ollama
- [ ] Decide MCP transports: HTTP primary, stdio bridge secondary
- [ ] Decide default MCP behavior: off by default
- [ ] Decide default tool permissions: read-only
- [ ] Decide global profile system MVP types
- [ ] Decide profile sensitivity levels
- [ ] Decide profile MCP exposure default: disabled
- [ ] Decide profile asset storage strategy
- [ ] Decide profile instruction/skill format

Deliverables:

- [ ] `DESIGN.md`
- [ ] `IMPLEMENTATION.md`
- [ ] `README.md` skeleton
- [ ] Architecture Decision Records in `docs/decisions/`

Acceptance criteria:

- A developer can read the docs and understand what to build first.
- There is no unresolved decision blocking the skeleton project.

---

## 5. Phase 1 — Project Bootstrap

Goal: create the app skeleton and development workflow.

Tasks:

- [ ] Initialize Git repository
- [ ] Create Tauri + React app
- [ ] Add TypeScript strict mode
- [ ] Add Rust formatting and linting
- [ ] Add pnpm workspace
- [ ] Add basic app layout
- [ ] Add app sidebar
- [ ] Add placeholder pages:
  - [ ] Workspaces
  - [ ] Profiles
  - [ ] Search
  - [ ] Context Packs
  - [ ] MCP Server
  - [ ] Index Jobs
  - [ ] Settings
  - [ ] Logs
- [ ] Add basic app logging
- [ ] Add dev scripts
- [ ] Add CI workflow if using GitHub

Suggested commands:

```bash
pnpm create tauri-app apps/desktop
pnpm install
pnpm dev
```

Deliverables:

- App boots in development
- Sidebar navigation works
- Placeholder pages render
- Basic logs are written

Acceptance criteria:

```bash
pnpm typecheck
pnpm lint
pnpm test
pnpm tauri dev
```

All commands should pass or have documented temporary exceptions.

---

## 6. Phase 2 — Workspace Registry

Goal: allow the user to add and manage project folders.

Tasks:

- [ ] Add workspace picker from Tauri/Rust side
- [ ] Validate selected path exists
- [ ] Canonicalize selected path
- [ ] Reject duplicate workspace roots
- [ ] Create global app SQLite database
- [ ] Create `workspaces` table
- [ ] Store workspace name, path, timestamps, status
- [ ] Show workspace list in UI
- [ ] Show workspace details page
- [ ] Add remove workspace action
- [ ] Add “open in file manager” action if supported

Security tasks:

- [ ] Never access a path that user did not select
- [ ] Store paths locally only
- [ ] Prepare future permission scopes

Deliverables:

- User can add workspace
- User can remove workspace
- Workspace persists after app restart

Acceptance criteria:

- Adding the same path twice is blocked
- Deleted/missing workspace path shows degraded status
- App does not crash when workspace path disappears

---

## 7. Phase 3 — File Scanner and Ignore Rules

Goal: scan workspace files safely and predictably.

Tasks:

- [ ] Implement recursive file scanner
- [ ] Add default ignore patterns
- [ ] Parse `.gitignore`
- [ ] Parse `.workgridignore`
- [ ] Add max file size setting
- [ ] Detect language by extension
- [ ] Compute file hash
- [ ] Store file metadata in workspace SQLite
- [ ] Show file count in UI
- [ ] Show ignored count
- [ ] Show scan errors

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

Deliverables:

- Workspace scan creates file records
- Ignored files do not get indexed
- Large files are skipped safely

Acceptance criteria:

- Scanner can handle at least 10,000 files without freezing UI
- Scanner can be cancelled
- Scan progress appears in UI
- Errors are logged but do not stop the entire scan

---

## 8. Phase 4 — SQLite Metadata and FTS Search

Goal: make indexed content searchable before adding vectors.

Tasks:

- [ ] Create per-workspace SQLite database
- [ ] Create `files`, `chunks`, `symbols`, `edges`, `index_jobs` tables
- [ ] Create `chunks_fts` virtual table
- [ ] Implement basic text chunker
- [ ] Store chunks with file path and line ranges
- [ ] Insert chunks into FTS
- [ ] Build keyword search API
- [ ] Add search page in UI
- [ ] Display file path, line range, and snippet

Deliverables:

- User can search workspace by keyword
- Search returns cited snippets
- Results open the containing file path externally or copy path

Acceptance criteria:

- Search for an exact function name finds the file
- Search for a config key finds the file
- Search result always includes file path and line range

---

## 9. Phase 5 — Embeddings and Vector Search

Goal: add semantic search.

Tasks:

- [ ] Add embedding provider abstraction
- [ ] Implement Ollama embedding provider
- [ ] Add embedding settings page
- [ ] Add embedding health check
- [ ] Add embedding queue
- [ ] Add LanceDB vector store
- [ ] Store chunk ID, workspace ID, file ID, vector, and metadata
- [ ] Implement vector search
- [ ] Add vector search to search UI
- [ ] Add vector rebuild action

Failure handling:

- [ ] If Ollama is offline, keep FTS usable
- [ ] Queue embeddings for later
- [ ] Mark vector index as stale
- [ ] Show user-friendly error

Deliverables:

- Semantic search works for indexed chunks
- User can search concepts, not just exact words

Acceptance criteria:

- Query “authentication logic” finds login/auth-related code
- Query “database migration for users” finds migration/model chunks
- App remains usable if embedding provider is unavailable

---

## 10. Phase 6 — Symbol Extraction

Goal: understand code structure.

Tasks:

- [ ] Add parser abstraction
- [ ] Add Tree-sitter integration or equivalent parser approach
- [ ] Implement JS/TS symbol extractor
- [ ] Implement PHP symbol extractor
- [ ] Implement Markdown heading extractor
- [ ] Implement JSON/YAML config extractor
- [ ] Implement SQL statement/migration extractor
- [ ] Store symbols in SQLite
- [ ] Link chunks to symbols
- [ ] Show symbols in workspace dashboard
- [ ] Add symbol search

Initial symbol kinds:

```text
function
class
method
interface
type
constant
route
controller
model
migration
table
config_key
env_key
```

Deliverables:

- Symbols appear for supported languages
- Search can prioritize symbol matches

Acceptance criteria:

- A PHP class is indexed as a class
- A TypeScript function is indexed as a function
- Markdown headings become document sections
- Parser failure falls back to plain text chunking

---

## 11. Phase 7 — Graph and Related Files

Goal: connect files and symbols.

Tasks:

- [ ] Create graph edge builder
- [ ] Add import/export edges for JS/TS
- [ ] Add PHP namespace/class reference edges
- [ ] Add config key references
- [ ] Add env key references
- [ ] Add SQL table references
- [ ] Add route-to-controller edges where detectable
- [ ] Add file-to-symbol edges
- [ ] Add related files API
- [ ] Add related files UI panel

Deliverables:

- User can view related files for a selected file
- Retrieval can expand from strong result to nearby files

Acceptance criteria:

- Imported file relationships are detected
- Route/controller relationship works for supported patterns
- Related files are ranked and explain why they are related

---

## 12. Phase 8 — Hybrid Retrieval

Goal: combine FTS, vector, symbols, and graph into reliable results.

Tasks:

- [ ] Implement query normalization
- [ ] Extract candidate symbols from query
- [ ] Run FTS search
- [ ] Run vector search
- [ ] Run symbol search
- [ ] Expand graph around strong matches
- [ ] Deduplicate results
- [ ] Implement scoring formula
- [ ] Return reason for every result
- [ ] Build compact context pack

Suggested scoring:

```text
final_score =
  0.40 * vector_score +
  0.25 * keyword_score +
  0.20 * symbol_score +
  0.10 * graph_proximity_score +
  0.05 * recency_score
```

Deliverables:

- One retrieval API powers UI and MCP
- Every result includes evidence

Acceptance criteria:

- Exact symbol queries prefer exact symbol matches
- Conceptual queries use vector results
- Related files appear only when helpful
- Weak evidence returns low confidence

---

## 13. Phase 9 — Global Profile System

Goal: create the global reusable profile memory layer.

This phase adds profiles for anything with definable characteristics: people, pets, places, objects, products, ideas, jobs, processes, preferences, skills, client context, and personal workflows.

Hard rule:

```text
Workspace memory is workspace-only.
Profile memory is global and reusable, but permissioned.
```

Tasks:

- [ ] Create `profiles.sqlite`
- [ ] Add profile tables:
  - [ ] `profiles`
  - [ ] `profile_attributes`
  - [ ] `profile_instructions`
  - [ ] `profile_assets`
  - [ ] `profile_relationships`
  - [ ] `profile_workspace_links`
  - [ ] `profile_audit_log`
- [ ] Add profile type registry
- [ ] Add default profile types:
  - [ ] Person
  - [ ] Pet
  - [ ] Place
  - [ ] Object
  - [ ] Product
  - [ ] Organization
  - [ ] Job / role
  - [ ] Idea
  - [ ] Process
  - [ ] Preference
  - [ ] Skill / instruction
  - [ ] Client
  - [ ] Custom type
- [ ] Add sensitivity levels:
  - [ ] public
  - [ ] internal
  - [ ] private
  - [ ] sensitive
  - [ ] secret
- [ ] Add profile CRUD UI
- [ ] Add dynamic attributes UI
- [ ] Add profile instruction editor
- [ ] Add trigger terms for instruction profiles
- [ ] Add profile tags
- [ ] Add relationship editor
- [ ] Add workspace relevance links
- [ ] Add profile FTS search
- [ ] Add profile embedding queue for safe fields
- [ ] Add profile vector search
- [ ] Add profile context pack preview
- [ ] Add field-level sensitivity handling
- [ ] Add profile audit log

Profile examples to create as fixtures:

- [ ] User commit style profile
- [ ] User development workflow profile
- [ ] Person profile sample
- [ ] Pet profile sample
- [ ] Client preference profile sample
- [ ] Process/checklist profile sample

Deliverables:

- Global profile manager works
- Profiles are stored separately from workspace indexes
- Profile search works with FTS
- Profile semantic search works when embeddings are available
- Profile instructions can be matched by trigger terms
- Profiles can be linked to workspaces without copying into workspace DBs

Acceptance criteria:

- User can create a profile for a person, pet, product, idea, process, and commit style.
- User can mark profile fields as sensitive.
- Sensitive fields are not embedded by default.
- Profiles are not exposed through MCP unless enabled.
- Profile context pack preview clearly labels profile context separately from workspace evidence.

Disaster checks:

- Profile DB corruption does not damage workspace DBs.
- Workspace deletion does not delete global profiles.
- Disabling profiles does not disable workspace search.
- Profile search works even if vector search is unavailable.
- Sensitive profile fields do not appear in logs.

---

## 14. Phase 10 — MCP Server

Goal: expose WorkGrid Memory workspace and profile context to agents.

Tasks:

- [ ] Add local HTTP MCP server
- [ ] Bind to `127.0.0.1`
- [ ] Add configurable port
- [ ] Add local auth token
- [ ] Add workspace MCP tools
- [ ] Add profile MCP tools
- [ ] Add context pack MCP tool
- [ ] Add MCP on/off toggle
- [ ] Add server logs
- [ ] Add stdio bridge CLI
- [ ] Add “copy MCP config” button
- [ ] Add basic client templates:
  - [ ] Claude Desktop
  - [ ] Codex CLI / compatible config if supported
  - [ ] Cline
  - [ ] Continue
  - [ ] Generic MCP client

MCP tools:

- [ ] `search_workspace`
- [ ] `get_file_context`
- [ ] `explain_symbol`
- [ ] `find_references`
- [ ] `get_related_files`
- [ ] `get_workspace_map`
- [ ] `verify_claim`
- [ ] `search_profiles`
- [ ] `get_profile_context`
- [ ] `get_relevant_profiles`
- [ ] `build_context_pack`

Deliverables:

- External agent can call WorkGrid Memory
- External agent can request profile context only when permitted
- MCP can be turned off
- MCP status is visible

Acceptance criteria:

- MCP server starts and stops from UI
- Tool call returns valid structured JSON
- Results include file paths and line ranges
- Server rejects unauthorized HTTP requests
- stdio bridge works with at least one MCP client

---

## 15. Phase 11 — Security and Redaction

Goal: prevent embarrassing local data leaks.

Tasks:

- [ ] Add secret detection
- [ ] Redact known token formats
- [ ] Ignore `.env` values
- [ ] Block access outside workspace root
- [ ] Resolve symlinks safely
- [ ] Add MCP workspace allowlist
- [ ] Add MCP profile allowlist
- [ ] Add per-tool enable/disable flags
- [ ] Add field-level profile sensitivity filtering
- [ ] Add sensitive profile MCP exposure warnings
- [ ] Add audit log for MCP calls
- [ ] Add audit log for profile reads over MCP
- [ ] Add “clear workspace memory” feature
- [ ] Add “rebuild from scratch” feature

Secret patterns to detect:

```text
AWS_ACCESS_KEY_ID
AWS_SECRET_ACCESS_KEY
OPENAI_API_KEY
ANTHROPIC_API_KEY
DEEPSEEK_API_KEY
DATABASE_URL
JWT_SECRET
PRIVATE_KEY
PASSWORD
TOKEN
SECRET
```

Deliverables:

- Secrets are not embedded by default
- MCP cannot read arbitrary files
- MCP cannot read disabled profiles
- Sensitive profile fields are withheld by default
- Logs do not expose secret values or sensitive profile fields

Acceptance criteria:

- `.env` secret values are redacted
- Path traversal attempts fail
- Symlink escape attempts fail
- MCP audit log records tool, workspace/profile, timestamp, and status
- Profile MCP tools fail closed when permissions are unclear

---

## 16. Phase 12 — UI Completion

Goal: make the product usable without reading logs like a monk reading entrails.

Tasks:

- [ ] Workspace dashboard
- [ ] Profile manager page
- [ ] Profile detail page
- [ ] Profile instruction editor
- [ ] Context pack preview page
- [ ] Index progress view
- [ ] Search page
- [ ] Result details panel
- [ ] File context view
- [ ] MCP server page
- [ ] Settings page
- [ ] Logs page
- [ ] Error banners
- [ ] Empty states
- [ ] Loading states
- [ ] Degraded status states

Important UI states:

```text
No workspace added
Workspace missing
Indexing in progress
Index stale
Embedding provider offline
Profile memory disabled
Sensitive profile context withheld
MCP server stopped
MCP server failed
Search has no results
Secret redaction active
```

Deliverables:

- Non-technical user can add workspace and start indexing
- Developer can diagnose failures without opening devtools

Acceptance criteria:

- App has no dead-end pages
- All long-running jobs show progress
- Errors are actionable

---

## 17. Phase 13 — Packaging and Release

Goal: ship installable builds.

Tasks:

- [ ] Configure Tauri bundling
- [ ] Bundle sidecar binaries
- [ ] Add versioning
- [ ] Add app icon
- [ ] Build Linux package
- [ ] Build Windows package
- [ ] Build macOS package if available
- [ ] Add update strategy
- [ ] Add release notes
- [ ] Add checksum generation
- [ ] Add smoke test after install

Deliverables:

- Installable desktop app
- Bundled MCP bridge
- Profile asset storage path
- Profile DB migration path
- Bundled or documented dependencies

Acceptance criteria:

- Fresh install launches
- User can add workspace
- Indexing works
- Search works
- MCP starts
- App can be uninstalled without deleting source projects

---

## 18. Testing Plan

### 18.1. Unit tests

Test:

- Ignore matching
- Hashing
- Chunking
- Symbol extraction
- Secret redaction
- Path safety
- Scoring formula
- Query normalization

### 18.2. Integration tests

Test:

- Scan fixture workspace
- Build SQLite index
- Build FTS index
- Build vector index
- Run retrieval
- Run MCP tool call

### 18.3. Fixture repositories

Create fixtures:

```text
fixtures/
  php-laravel-sample/
  ts-react-sample/
  mixed-project-sample/
  secret-redaction-sample/
  huge-files-sample/
  broken-syntax-sample/
  profile-samples/
    person-profile.json
    pet-profile.json
    commit-style-profile.json
    sensitive-profile.json
```

### 18.4. Manual test checklist

Before release:

- [ ] Add workspace
- [ ] Reindex workspace
- [ ] Search keyword
- [ ] Search semantic query
- [ ] View symbols
- [ ] Create profile
- [ ] Search profile
- [ ] Create commit style instruction profile
- [ ] Preview context pack with workspace + profile context
- [ ] Verify sensitive profile fields are withheld
- [ ] Start MCP
- [ ] Call MCP tool from client
- [ ] Stop MCP
- [ ] Delete workspace from app
- [ ] Confirm global profiles were not deleted
- [ ] App restart preserves state
- [ ] Embedding offline mode works
- [ ] Secret redaction works

---

## 19. Disaster Development Plan

This section exists because development tools fail, agents get rate-limited, models hallucinate, packages break, and computers remain dramatic despite decades of alleged progress.

---

### 19.1. Codex rate-limited

Problem:

Codex is unavailable, slowed, or usage-limited.

Response:

- [ ] Stop giving Codex large tasks
- [ ] Switch to smaller tasks
- [ ] Move architecture/review tasks to Claude
- [ ] Move local repo tasks to CodeWhale if available
- [ ] Use Continue/Cline with a cheaper provider for small edits
- [ ] Use manual coding for core changes
- [ ] Continue work from task handoff files

Fallback provider split:

```text
Codex:
- Complex implementation
- Test fixing
- Multi-file refactors

Claude:
- Architecture review
- Rust/Tauri reasoning
- Documentation
- Security review
- Debug explanations

CodeWhale:
- Local codebase tasks
- Repeated edits
- Project-specific implementation with local context

DeepSeek/Qwen/Kimi/GLM via compatible tools:
- Cheap code generation
- Boilerplate
- Test scaffolding
- Simple bug fixes

Manual:
- Database migrations
- Security boundaries
- Release packaging
- Final review
```

Required handoff file:

```text
docs/handoff/current-task.md
```

Template:

```markdown
# Current Task Handoff

## Goal

## Current branch

## Files changed

## Files allowed to edit

## Files forbidden to edit

## Known failing tests

## Last successful command

## Next command to run

## Acceptance criteria

## Rollback plan
```

---

### 19.2. Agent introduces broken architecture

Problem:

Agent creates a massive abstraction or rewrites unrelated modules.

Response:

- [ ] Reject the patch
- [ ] Restore from Git
- [ ] Re-scope the task to smaller files
- [ ] Add “files allowed to edit” constraint
- [ ] Add explicit non-goals
- [ ] Ask another agent to review before merging

Rule:

No agent may edit more than one subsystem unless explicitly authorized.

---

### 19.3. Agent fabricates APIs

Problem:

Agent uses non-existing Tauri, LanceDB, MCP, or Rust APIs.

Response:

- [ ] Require links to official docs in implementation notes
- [ ] Add compile check before accepting
- [ ] Add minimal proof-of-concept before full integration
- [ ] Prefer official examples over invented wrappers

Acceptance rule:

No dependency integration is accepted until a minimal runnable example works.

---

### 19.4. Dependency breaks or native package fails

Problem:

SQLite, LanceDB, Tree-sitter, or Tauri sidecar packaging fails on one OS.

Response:

- [ ] Isolate dependency behind interface
- [ ] Create minimal reproduction
- [ ] Test on Linux first
- [ ] Add fallback mode
- [ ] Document unsupported platform if needed

Examples:

```text
LanceDB fails:
  Keep SQLite FTS and metadata working.
  Mark vector search unavailable.

Tree-sitter fails:
  Use fallback chunking.
  Mark symbol extraction degraded.

Sidecar fails:
  Run engine in dev mode externally.
  Fix packaging later.
```

---

### 19.5. Index database corruption

Problem:

Workspace SQLite or vector index corrupts.

Response:

- [ ] App detects open/query failure
- [ ] Mark workspace degraded
- [ ] Offer rebuild
- [ ] Preserve corrupted DB copy for debugging
- [ ] Rebuild from source files

Never silently delete indexes. Silent deletion is rude even by software standards.

---

### 19.6. User adds huge monorepo

Problem:

Workspace has hundreds of thousands of files.

Response:

- [ ] Warn before indexing
- [ ] Apply default ignore aggressively
- [ ] Estimate file count first
- [ ] Let user select folders
- [ ] Batch indexing
- [ ] Allow pause/resume
- [ ] Enforce max file size

MVP limit recommendation:

```text
Soft warning: 20,000 files
Hard confirmation: 50,000 files
Default max file size: 512 KB
```

---

### 19.7. Embedding provider unavailable

Problem:

Ollama is not installed, model missing, or service offline.

Response:

- [ ] Continue metadata indexing
- [ ] Continue FTS indexing
- [ ] Queue embeddings
- [ ] Show setup instructions
- [ ] Allow hosted embedding fallback
- [ ] Allow vector search disabled mode

User-facing status:

```text
Embedding provider offline. Keyword search is still available. Semantic search will resume after embeddings are available.
```

---

### 19.8. MCP client incompatibility

Problem:

A target agent does not support HTTP MCP or expects stdio only.

Response:

- [ ] Use stdio bridge CLI
- [ ] Generate client-specific config
- [ ] Keep generic config available
- [ ] Add compatibility notes per client

Minimum supported MCP modes:

```text
HTTP:
  WorkGrid Memory desktop app listens on localhost.

stdio:
  workgrid-memory-mcp bridge connects to local app or index.
```

---

### 19.9. File watcher overload

Problem:

Large project generates too many file events.

Response:

- [ ] Debounce file events
- [ ] Batch updates
- [ ] Collapse repeated changes
- [ ] Pause watcher during full reindex
- [ ] Let user disable watch mode

Default behavior:

```text
Debounce: 1000ms
Batch window: 5000ms
Max batch before full rescan: configurable
```

---

### 19.10. Profile data overexposure

Problem:

An MCP tool returns too much profile context, exposes sensitive profile fields, or includes unrelated personal data.

Response:

- [ ] Disable profile MCP tools immediately
- [ ] Rotate MCP token
- [ ] Mark profile context pack builder as unsafe
- [ ] Inspect MCP audit log
- [ ] Add regression test using sensitive profile fixture
- [ ] Re-enable only after field-level filtering passes

Rules:

```text
Workspace MCP may continue if safe.
Profile MCP must fail closed.
Sensitive fields are denied by default.
```

---

### 19.11. Conflicting profile instructions

Problem:

Global profile says one thing, workspace rules say another.

Example:

```text
Global Commit Style: use conventional commits.
Workspace Rule: use ticket prefix only, e.g. BILL-123 summary.
```

Response:

- [ ] Prefer workspace-specific rule
- [ ] Show conflict warning
- [ ] Include both sources in context pack diagnostics
- [ ] Let user pin workspace override
- [ ] Mark stale profile instruction for review

Priority order:

```text
1. Explicit current user instruction
2. Workspace-specific profile link or rule
3. Project docs and repo conventions
4. Global profile instruction
5. Default app behavior
```

---

### 19.12. Profile bloat and noisy retrieval

Problem:

Too many profiles match every request, causing agents to receive irrelevant personal context.

Response:

- [ ] Cap profile count per context pack
- [ ] Require trigger match for instruction profiles
- [ ] Penalize weak vector-only matches
- [ ] Add per-workspace profile allow/deny list
- [ ] Show why a profile was included
- [ ] Add "never include for this workspace" action

Default limits:

```text
Max profiles per context pack: 5
Max sensitive profiles per context pack: 0 unless explicitly allowed
Instruction profile requires trigger term or manual pin
```

---

### 19.13. Profile asset failure

Problem:

Image/document asset is missing, corrupted, moved, or unreadable.

Response:

- [ ] Keep profile text metadata available
- [ ] Mark asset missing
- [ ] Show repair option
- [ ] Never delete profile automatically
- [ ] Avoid exposing broken asset references over MCP

---

### 19.14. Bad release build

Problem:

Packaged app works in dev but fails after install.

Response:

- [ ] Add smoke test script
- [ ] Test fresh install
- [ ] Verify sidecar paths
- [ ] Verify database paths
- [ ] Verify permissions
- [ ] Verify MCP server starts
- [ ] Keep previous release available

Release is blocked if:

- App cannot launch
- Workspace cannot be added
- Indexing cannot start
- Search cannot run
- MCP cannot start or fail gracefully

---

## 20. Agent Workflow

Use this workflow for AI-assisted development.

### 20.1. Before giving task to an agent

Prepare:

- Current branch
- Relevant files
- Task goal
- Constraints
- Test command
- Expected output

### 20.2. During agent work

Require:

- Small diffs
- No dependency changes unless asked
- No unrelated formatting
- No architecture changes without approval
- Tests or manual verification notes

### 20.3. After agent work

Run:

```bash
pnpm typecheck
pnpm test
cargo test
pnpm tauri dev
```

Then review:

- Security boundaries
- File access
- Error handling
- Logs
- User-facing messages
- Migration safety

---

## 21. Standard Agent Prompt Template

Use this when delegating work:

```markdown
# Task

Implement: <small task name>

## Context

This project is WorkGrid Memory, a React + Tauri desktop app that indexes workspaces, manages global profiles, and exposes permissioned context through MCP.

## Goal

<clear goal>

## Files you may edit

- path/to/file
- path/to/other-file

## Files you must not edit

- DESIGN.md
- IMPLEMENTATION.md
- unrelated subsystem files

## Requirements

- Keep the app runnable.
- Add or update tests.
- Do not introduce new dependencies unless necessary.
- Do not rewrite unrelated code.
- Do not mix workspace data into global profiles.
- Do not expose sensitive profile fields.
- Use existing project patterns.

## Acceptance criteria

- <criterion 1>
- <criterion 2>

## Commands to run

```bash
pnpm typecheck
pnpm test
cargo test
```

## Output expected

Summarize:
- What changed
- How to test
- Any risks
```

---

## 22. Development Milestones

### 22.1. Milestone A — Skeleton App ✅

Target:

- Tauri app boots
- React UI shell exists
- Workspace registry works

Exit criteria:

- User can add a workspace
- Workspace persists across restart

### 22.2. Milestone B — Searchable Index ✅

Target:

- Scanner works
- SQLite metadata works
- FTS search works

Exit criteria:

- User can search exact text and get line-numbered results

### 22.3. Milestone C — Semantic Memory ✅

Target:

- Embedding provider works
- Vector search works (SQLite BLOB)
- Semantic search works (hybrid FTS+vector)

Exit criteria:

- User can search by concept and get relevant chunks

### 22.4. Milestone D — Code Intelligence ✅

Target:

- Symbols extracted
- Basic graph works (import/export/route/SQL edges)
- Hybrid retrieval works (FTS + vector merged)

Exit criteria:

- Search can prioritize functions/classes/routes

### 22.5. Milestone E — Global Profile Memory ✅

Target:

- Profile registry works (CRUD, attributes, instructions, assets, relationships, workspace links)
- Profile search works (FTS)
- Profile instructions work (trigger terms + matching)
- Profile context packs are permissioned (MCP exposure toggle)

Exit criteria:

- User can create a commit style profile and have it appear in a relevant context pack without exposing unrelated personal profiles.

### 22.6. Milestone F — MCP Integration ✅

Target:

- MCP server runs (HTTP on 127.0.0.1, background thread)
- Workspace tools return structured results (11 tools)
- Profile tools respect permissions (auth token, profile exposure)
- ~~stdio bridge works~~ (deferred)

Exit criteria:

- At least one external agent can call `search_workspace`
- At least one external agent can call `build_context_pack`

### 22.7. Milestone G — Release Candidate 🚧

Target:

- Security pass (secret redaction done, auth token done)
- Packaging pass (pending)
- Fixture tests pass (pending)
- Docs complete (in progress)

Exit criteria:

- Installable app works on target OS

---

## 23. Definition of Done

WorkGrid Memory v1.0 is done when:

- [ ] App installs cleanly
- [x] App manages multiple workspaces
- [x] Indexing is incremental (rebuild supported, watcher deferred)
- [x] Search works with FTS and vectors (hybrid FTS+vector)
- [x] Symbols are extracted for core languages (JS/TS/PHP/Python/Rust/MD)
- [x] Global profiles can be created, searched, linked, and archived
- [x] Profile instructions can guide tasks such as commit message generation
- [x] Profile context remains separate from workspace evidence
- [x] MCP server can be started/stopped (HTTP on 127.0.0.1, auth token)
- [x] MCP tools return cited evidence (11 tools, file paths + line ranges)
- [x] MCP profile tools respect exposure settings (mcp_exposure toggle)
- [x] Secret redaction is enabled by default
- [x] Sensitive profile fields are denied by default (sensitivity levels)
- [x] Broken providers degrade gracefully (embedding falls back to FTS-only)
- [x] Logs help diagnose failures (tracing spans throughout)
- [x] Docs explain setup and usage (DESIGN.md, IMPLEMENTATION.md, ADRs)
- [ ] Test fixtures pass
- [ ] Release build passes smoke test

---

## 24. Final Build Philosophy

Build the boring reliable core first:

```text
Workspace registry
File scanner
SQLite metadata
FTS search
Embeddings
Vector search
Symbols
Hybrid retrieval
Global profiles
Profile-aware context packs
MCP
UI polish
Packaging
```

Do not start with fancy graphs, cloud sync, or agent chat. Those are distractions wearing a product manager costume.

WorkGrid Memory succeeds if an external AI agent can ask:

```text
Where is this behavior implemented?
How should I write this commit message based on my own style?
Which workspace evidence and global profiles support this answer?
```

And receive:

```text
Here are the exact files, symbols, line ranges, and permitted profile context that support the answer.
```

That is the product.
