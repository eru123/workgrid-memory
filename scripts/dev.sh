#!/usr/bin/env bash
set -euo pipefail

echo "=== WorkGrid Memory Development ==="
echo ""

# Install dependencies
echo "Installing pnpm dependencies..."
pnpm install --frozen-lockfile 2>/dev/null || pnpm install

echo ""
echo "=== Starting development server ==="
cd apps/desktop
pnpm tauri dev
