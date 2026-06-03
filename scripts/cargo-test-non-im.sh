#!/usr/bin/env bash
# Exodus — run Rust lib tests excluding IM/messaging modules (default CI/dev pass).
# IM tests require network setup; enable with: ./scripts/cargo-test-im.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/src-tauri"

echo "▶ cargo test --lib (IM tests skipped; no im-tests feature)"
cargo test --lib -- --test-threads=1 "$@"

echo "✓ Non-IM Rust tests finished"
