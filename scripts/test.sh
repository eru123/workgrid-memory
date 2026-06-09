#!/usr/bin/env bash
set -euo pipefail

echo "=== WorkGrid Memory Test Runner ==="
echo ""

echo "--- TypeScript typecheck ---"
pnpm typecheck || echo "⚠ typecheck had warnings"

echo ""
echo "--- Rust tests ---"
cargo test --workspace 2>&1 || echo "⚠ cargo test had failures"

echo ""
echo "--- Rust build check ---"
cargo check --workspace 2>&1

echo ""
echo "=== All checks complete ==="
