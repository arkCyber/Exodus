#!/usr/bin/env bash
# Exodus Browser — Playwright against a running `pnpm tauri dev` (TAURI_E2E=1).
# Usage:
#   Terminal 1: pnpm tauri dev
#   Terminal 2: ./scripts/test-e2e-tauri.sh
#
# Optional: PLAYWRIGHT_BASE_URL=http://localhost:1421

set -euo pipefail

# Drop accidental inline comments/extra words from copy-paste (pnpm test:e2e:tauri # note)
ARGS=()
for arg in "$@"; do
  [[ "$arg" == \#* ]] && break
  [[ "$arg" == --* ]] && ARGS+=("$arg") && continue
  [[ "$arg" == -* ]] && ARGS+=("$arg") && continue
done
set -- "${ARGS[@]}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASE_URL="${PLAYWRIGHT_BASE_URL:-http://localhost:1421}"

echo "▶ Waiting for dev server at ${BASE_URL}..."
for i in $(seq 1 60); do
  if curl -sf "${BASE_URL}/" -o /dev/null; then
    echo "   Server ready."
    break
  fi
  if [[ "$i" -eq 60 ]]; then
    echo "ERROR: Start 'pnpm tauri dev' first (or set PLAYWRIGHT_BASE_URL)." >&2
    exit 1
  fi
  sleep 2
done

export TAURI_E2E=1
export PLAYWRIGHT_BASE_URL="${BASE_URL}"

pnpm exec playwright test e2e/vue-shell-qa.spec.ts --project=chromium

echo "✓ Tauri E2E smoke passed (including backend-only tests when configured)"
