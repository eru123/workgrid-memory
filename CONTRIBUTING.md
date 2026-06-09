# Contributing to WorkGrid Memory

Thank you for considering a contribution to WorkGrid Memory.

WorkGrid Memory is a local-first desktop memory system for indexing developer workspaces, managing global reusable profiles, and exposing grounded context to AI agents through MCP.

This project deals with code, local files, profile data, and potentially sensitive user context. Contributions should prioritize correctness, privacy, predictable behavior, and boring reliability. Boring reliability is not glamorous, but neither is corrupting a user's local memory index.

## Project Goals

WorkGrid Memory aims to provide:

- Local-first workspace indexing
- Workspace-scoped code memory
- Global profile memory
- MCP-based context access for AI agents
- Hybrid search using keyword, vector, symbol, and graph retrieval
- Privacy controls for profile and workspace data
- Clear evidence with file paths, symbols, and line ranges

## Project Non-Goals

For now, WorkGrid Memory should not become:

- A cloud-only memory service
- A hosted surveillance-shaped productivity platform
- A general-purpose social graph
- A replacement for Git
- A replacement for an IDE
- A tool that silently exposes private profiles to AI agents
- A write-enabled MCP automation engine without strict permissions

## Development Stack

Planned stack:

- React
- TypeScript
- Tauri
- Rust
- SQLite
- SQLite FTS5
- LanceDB or compatible local vector storage
- Tree-sitter
- MCP server layer
- Local embedding providers such as Ollama

## Repository Structure

A future structure may look like this:

```text
workgrid-memory/
  apps/
    desktop/
  crates/
    engine/
    mcp/
    shared/
  packages/
    ui/
    types/
  docs/
    DESIGN.md
    IMPLEMENTATION.md
  .github/
```

The final structure may change. That is allowed. Chaos is not architecture, though, so update the docs when it changes.

## How to Contribute

1. Fork the repository.
2. Create a feature branch.
3. Keep changes focused.
4. Add or update tests when applicable.
5. Update documentation for user-facing or architecture-level changes.
6. Open a pull request using the pull request template.

Suggested branch naming:

```text
feat/workspace-indexing
fix/profile-redaction
docs/mcp-tools
refactor/storage-layer
test/file-scanner
```

## Commit Style

Use concise conventional commits when possible:

```text
feat(indexer): add workspace file scanner
fix(profiles): prevent sensitive fields from MCP output
docs(security): document profile exposure policy
refactor(storage): split profile and workspace stores
test(mcp): add search workspace tool fixture
```

Recommended types:

- `feat`
- `fix`
- `docs`
- `refactor`
- `test`
- `chore`
- `ci`
- `perf`
- `security`

## Pull Request Expectations

A good pull request should include:

- Clear summary
- Reason for the change
- Screenshots for UI changes
- Test notes
- Security/privacy notes if the change touches files, profiles, MCP, or storage
- Migration notes if storage schemas changed

Avoid giant pull requests unless the change genuinely cannot be split. Giant PRs are where review quality goes to die wearing a badge.

## Working With Profiles

Profiles may represent people, pets, places, products, objects, workflows, processes, ideas, or other reusable context.

When contributing profile-related features:

- Default to private
- Do not expose sensitive fields through MCP by default
- Keep field-level privacy support
- Avoid assumptions about identity, relationships, or personal details
- Add audit-friendly behavior for profile access
- Make retrieval explainable

## Working With Workspace Data

Workspace data must remain workspace-scoped.

Do not design features that silently mix one workspace's code context into another workspace.

When cross-workspace features are needed, they must be explicit, visible, and permission-controlled.

## MCP Tool Rules

MCP tools should be read-only by default.

Every MCP result should prefer structured, evidence-based output:

- File path
- Line range
- Symbol name
- Match reason
- Confidence score
- Privacy warnings when relevant

Do not add write tools casually. The phrase "the agent can just edit it" has launched many debugging tragedies.

## Security and Privacy Requirements

Contributors must be careful with:

- Local file access
- Profile data
- Photos and assets
- Secret redaction
- Environment files
- MCP exposure
- Logs
- Local HTTP server behavior
- Token storage

Never log secrets or sensitive profile values.

## Testing Guidelines

Add tests for:

- File scanning
- Ignore rules
- Secret redaction
- Profile field privacy
- MCP tool output
- Search result ranking
- Storage migrations
- Parser fallbacks

Recommended fixture style:

```text
fixtures/
  php-laravel-app/
  ts-react-app/
  mixed-workspace/
  profile-memory/
```

## Documentation Guidelines

Update documentation when changing:

- Architecture
- Storage schema
- MCP tools
- Profile behavior
- Security behavior
- Installation steps
- Developer workflow

Important docs:

- `README.md`
- `DESIGN.md`
- `IMPLEMENTATION.md`
- `SECURITY.md`

## Reporting Bugs

Use the bug report issue template.

Include:

- OS
- App version or commit
- Workspace size
- Language/framework
- Steps to reproduce
- Expected result
- Actual result
- Logs if safe to share

Remove secrets and private profile data before sharing logs.

## Requesting Features

Use the feature request issue template.

Good feature requests explain:

- The problem
- The proposed behavior
- Why it matters
- Security/privacy impact
- Possible alternatives

## Code Review Standards

Reviewers should check:

- Correctness
- Privacy boundaries
- MCP exposure behavior
- Error handling
- Test coverage
- Performance on large workspaces
- Failure behavior when embeddings, parsers, or agents are unavailable

## Development Disaster Notes

If AI coding tools become unavailable or rate-limited:

- Keep tasks small
- Use `IMPLEMENTATION.md` as the recovery source
- Switch between Codex, Claude, CodeWhale, DeepSeek, or manual implementation
- Commit working milestones frequently
- Avoid architecture rewrites from a single agent output

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
