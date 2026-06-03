#!/usr/bin/env bash
# Exodus Browser — automated tests for workspace extensions/ dev samples.
# Usage: sh scripts/test-dev-extensions.sh   or   pnpm test:extensions

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " Dev extensions test — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Node validator (manifest, syntax, quality bar)..."
node scripts/validate-dev-extensions.mjs

echo ""
echo "▶ Rust integration (load, inject, audits)..."
cd src-tauri
cargo test dev_extensions_tests --lib -- --test-threads=1

echo ""
echo "✓ Dev extensions tests passed"
