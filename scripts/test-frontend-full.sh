#!/usr/bin/env bash
# Exodus Browser — full frontend unit test + typecheck gate.
# Usage: ./scripts/test-frontend-full.sh   or   pnpm test:frontend:full

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " Frontend full — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Vue/TS check (vue-tsc)..."
pnpm check

echo ""
echo "▶ Vitest (all src/**/*.test.ts)..."
pnpm vitest run

echo ""
echo "✓ Frontend full tests passed"
