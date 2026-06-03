#!/usr/bin/env bash
# Exodus — run IM/messaging integration tests only (requires im-tests feature).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/src-tauri"

echo "▶ cargo test --lib --features im-tests (group chat, contacts, gossip, social feed, …)"
cargo test --lib --features im-tests -- --test-threads=2 "$@"

echo "✓ IM Rust tests finished"
