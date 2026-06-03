#!/usr/bin/env bash
# Exodus Browser — fast verify (skips full exodus-tauri lib test suite).
# Usage: sh scripts/verify-quick.sh   or   pnpm verify:quick

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "════════════════════════════════════════"
echo " Exodus verify-quick — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Svelte typecheck..."
pnpm check

echo ""
echo "▶ Frontend unit tests..."
pnpm test

echo ""
echo "▶ Integration (extensions + AI/Hermes + invoke)..."
sh scripts/test-integration.sh

echo ""
echo "▶ Rust: exodus-core..."
cargo test -p exodus-core

echo ""
echo "▶ P2P CDN focused tests..."
sh scripts/test-p2p-cdn.sh

echo ""
echo "▶ Allama microservice tests..."
sh scripts/test-allama.sh

echo ""
echo "▶ Rust: plugin + Hermes focused tests..."
cd src-tauri
cargo test plugins::alarms::tests -- --test-threads=2
cargo test plugins::dev_extensions_tests -- --test-threads=2
cargo test hermes_agent::tests -- --test-threads=2
cargo test allama_stack_test -- --test-threads=2
cd "$ROOT"

echo ""
echo "════════════════════════════════════════"
echo " ✓ verify-quick passed"
echo "════════════════════════════════════════"
