#!/usr/bin/env bash
# Exodus Browser — fast test pass (no svelte-check). Use during tight edit loops.
# Usage: ./scripts/test-quick.sh   or   pnpm test:quick

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "▶ Vitest..."
pnpm test

echo "▶ Rust tests (non-IM)..."
cargo test -p exodus-core
"$ROOT/scripts/cargo-test-non-im.sh"

echo "✓ Quick tests passed"
