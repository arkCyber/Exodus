#!/usr/bin/env bash
# Exodus Browser — WebAssembly compatibility test runner.
#
# Usage:
#   ./scripts/test-wasm.sh              # Playwright smoke page only (starts dev:e2e if needed)
#   ./scripts/test-wasm.sh --tauri      # Native tab webview tests (requires pnpm tauri dev)
#   ./scripts/test-wasm.sh --manual       # Print manual QA URLs only
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MODE="${1:-}"

manual_urls() {
  cat <<'EOF'

Manual WASM QA (open in Exodus omnibox):
  1. Local smoke (required):  http://localhost:1421/test/wasm-smoke.html
  2. W3C demo (network):      https://webassembly.org/demo/
  3. WebGL sanity (network):  https://get.webgl.org/

Pass criteria for (1): green banner "All required checks passed", body[data-wasm-all-pass="1"].

Optional: SharedArrayBuffer line may FAIL without COOP/COEP — that is expected on many sites.

EOF
}

if [[ "$MODE" == "--manual" ]]; then
  manual_urls
  exit 0
fi

echo "▶ Playwright: WASM smoke page (Chromium)…"
pnpm exec playwright test e2e/webassembly.spec.ts --project=chromium -g "WASM smoke page"

if [[ "$MODE" == "--tauri" ]]; then
  BASE_URL="${PLAYWRIGHT_BASE_URL:-http://localhost:1421}"
  echo "▶ Waiting for Tauri dev at ${BASE_URL}…"
  for i in $(seq 1 45); do
    if curl -sf "${BASE_URL}/" -o /dev/null; then
      echo "   Server ready."
      break
    fi
    if [[ "$i" -eq 45 ]]; then
      echo "ERROR: Start 'pnpm tauri dev' in another terminal first." >&2
      exit 1
    fi
    sleep 2
  done
  export TAURI_E2E=1
  export PLAYWRIGHT_BASE_URL="${BASE_URL}"
  echo "▶ Playwright: shell navigation + webview (when IPC available)…"
  pnpm exec playwright test e2e/webassembly.spec.ts --project=chromium -g "TAURI_E2E|native tab"
  echo ""
  echo "▶ Manual native webview check (required for full WASM proof in Tauri):"
  echo "   1. In Exodus app DevTools console:"
  echo "      await __EXODUS_E2E__.runWasmSmokeCheck()"
  echo "   2. Expect: { pass: true, raw: '1' }"
  echo "✓ Tauri WASM automation finished"
else
  echo ""
  echo "Tip: for shell + webview checks, run in another terminal: pnpm tauri dev"
  echo "     then: ./scripts/test-wasm.sh --tauri"
  manual_urls
fi

echo "✓ WASM automated checks done"
