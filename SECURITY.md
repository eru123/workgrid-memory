# Security Policy

WorkGrid Memory handles local workspaces, code indexes, profile memory, personal context, photos/assets, and MCP access. Treat this project as privacy-sensitive by default.

If you discover a vulnerability, please report it responsibly.

## Supported Versions

WorkGrid Memory is currently in early development. Until the first stable release, only the latest `main` branch is considered supported.

| Version | Supported |
| ------- | --------- |
| main | Yes |
| pre-release builds | Best effort |
| older commits | No |

## Reporting a Vulnerability

Please do not open a public GitHub issue for security vulnerabilities.

Report security concerns by email:

```text
yeoligoakino@gmail.com
```

Use the subject:

```text
[Security] WorkGrid Memory vulnerability report
```

Include as much detail as safely possible:

- Summary of the issue
- Steps to reproduce
- Affected component
- Expected impact
- Screenshots or logs if safe
- Suggested fix, if available

Do not include real secrets, private profile data, private photos, or customer data in the report.

## Security Scope

Security-sensitive areas include:

- Workspace file access
- Profile memory
- Profile assets
- Photo and document handling
- MCP server access
- MCP tool output
- Local HTTP server behavior
- stdio bridge behavior
- Secret redaction
- Logs
- Storage migrations
- Import/export
- Backup/restore
- Embedding provider configuration
- Agent/provider integration

## Out of Scope

The following are generally out of scope unless they expose private data or allow code execution:

- Cosmetic UI bugs
- Missing icons
- Typographical errors
- Feature requests
- Performance complaints without security impact

## Security Design Rules

WorkGrid Memory should follow these rules:

- Local-first by default
- No cloud sync unless explicitly configured
- Workspace data stays workspace-scoped
- Global profiles require privacy controls
- Sensitive profile fields are MCP-disabled by default
- Profile assets are private by default
- `.env` values must not be indexed
- Secret-like values must be redacted from logs
- MCP tools are read-only by default
- Local HTTP MCP binds to `127.0.0.1`
- HTTP MCP requires a local token
- File access must be restricted to approved workspace paths
- Symlink/path traversal must be blocked
- Context packs must include privacy warnings when fields are excluded

## Vulnerability Examples

Please report issues such as:

- MCP exposes private profile fields without permission
- MCP exposes workspace files from another workspace
- Secret values are indexed or logged
- Path traversal allows reading files outside approved folders
- Local HTTP server binds publicly
- MCP token is leaked in logs
- Profile assets can be read without permission
- Workspace data from one project appears in another project
- Malicious workspace file causes code execution during indexing
- Import/export leaks hidden sensitive fields

## Responsible Disclosure

Please allow reasonable time for investigation and remediation before public disclosure.

Expected process:

1. Report received
2. Maintainer confirms receipt
3. Issue is reproduced and assessed
4. Fix is prepared
5. Release or patch notes are published if applicable
6. Reporter may be credited unless they prefer to remain anonymous

## Security Hardening Checklist

Before release, the project should verify:

- Secret redaction tests pass
- Profile exposure tests pass
- MCP access tests pass
- Path traversal tests pass
- Local HTTP server binding is restricted
- Logs do not contain sensitive data
- Import/export respects privacy settings
- Workspace indexes remain scoped
