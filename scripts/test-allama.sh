#!/usr/bin/env bash
# Exodus Browser — focused Allama microservice tests (Rust + frontend).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "▶ Allama Rust integration (allama_stack + related)..."
cd src-tauri
cargo test -p exodus-tauri --lib allama_stack
cargo test -p exodus-tauri --lib allama_process
cargo test -p exodus-tauri --lib allama_http_client
cargo test -p exodus-tauri --lib inference_engine::tests
cargo test -p exodus-tauri --lib hermes_agent::tests
cd "$ROOT"

echo "▶ Allama frontend client tests..."
pnpm exec vitest run src/lib/allamaClient.test.ts src/lib/allamaDefaults.test.ts src/lib/sidebarAiChat.test.ts

if [[ -x "${ALLAMA_BINARY:-}" ]] \
  || [[ -x "$ROOT/../Allama/allama/target/release/allama" ]] \
  || [[ -x "$HOME/Allama/allama/target/release/allama" ]]; then
  echo "▶ Native Allama HTTP verify (optional)..."
  ALLAMA_VERIFY_MODEL="${ALLAMA_VERIFY_MODEL:-cli-smoke-model}" \
    sh "$ROOT/scripts/verify-allama-native.sh"
else
  echo "▷ Skipping native verify (no allama binary; run: sh scripts/build-allama.sh)"
fi

echo "✓ Allama tests passed"
