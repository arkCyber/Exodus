#!/usr/bin/env bash
# Exodus Browser — run full automated test suite (frontend + Rust).
# Usage: ./scripts/verify.sh   or   pnpm verify

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " Exodus verify — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Svelte typecheck (svelte-check)..."
pnpm check

echo ""
echo "▶ Frontend unit tests (Vitest)..."
pnpm test

echo ""
echo "▶ Rust: exodus-core..."
cargo test -p exodus-core

echo ""
echo "▶ Tauri invoke ↔ handler sync..."
sh scripts/check-invoke-commands.sh

echo ""
echo "▶ P2P CDN focused tests..."
sh scripts/test-p2p-cdn.sh

echo ""
echo "▶ Allama microservice tests..."
sh scripts/test-allama.sh

echo ""
echo "▶ Rust: exodus-tauri (non-IM tests; IM gated by im-tests feature)..."
sh scripts/cargo-test-non-im.sh

echo ""
echo "════════════════════════════════════════"
echo " ✓ All checks passed"
echo "════════════════════════════════════════"
