#!/usr/bin/env bash
set -euo pipefail

echo "=== WorkGrid Memory Package ==="
echo ""

cd apps/desktop
pnpm tauri build

echo ""
echo "=== Package complete ==="
