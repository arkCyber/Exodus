#!/usr/bin/env bash
# Exodus — start Allama + desktop app with AI sidebar ready for chat tests.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export ALLAMA_PORT="${ALLAMA_PORT:-11435}"

if [[ -z "${ALLAMA_BINARY:-}" ]]; then
  for candidate in \
    "$ROOT/../Allama/allama/target/release/allama" \
    "$HOME/Allama/allama/target/release/allama" \
    "$ROOT/allama/target/release/allama"; do
    if [[ -x "$candidate" ]]; then
      export ALLAMA_BINARY="$candidate"
      break
    fi
  done
fi

echo "== Exodus AI environment =="
echo "   Project: $ROOT"
echo "   Allama:  ${ALLAMA_BINARY:-auto-detect} → port $ALLAMA_PORT"
echo ""

sh "$ROOT/scripts/start-allama-chat.sh"

echo ""
echo "== Quick chat test (optional) =="
if sh "$ROOT/scripts/test-allama-chat.sh" 2>/dev/null; then
  echo "✓ HTTP chat test passed"
else
  echo "⚠ Chat test skipped or failed — you can still try the in-app sidebar"
fi

echo ""
echo "== Starting Exodus (Tauri dev) =="
echo "   AI sidebar opens by default · new tab shows wallpaper"
echo ""

exec pnpm tauri dev
