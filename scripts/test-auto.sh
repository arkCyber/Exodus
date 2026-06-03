#!/usr/bin/env bash
# Exodus Browser — automated frontend regression (unit + optional E2E + optional verify).
# Usage:
#   ./scripts/test-auto.sh              # Vue shell Vitest subset (~50 tests)
#   ./scripts/test-auto.sh --all        # + full Vitest (all src/**/*.test.ts)
#   ./scripts/test-auto.sh --e2e        # shell subset + Playwright (CI PR gate)
#   ./scripts/test-auto.sh --e2e --all  # shell + full Vitest + Playwright
#   ./scripts/test-auto.sh --verify     # + verify-quick (Rust subset)
#   ./scripts/test-auto.sh --full       # --e2e --all + verify-quick
#
# Manual QA in Tauri still requires: pnpm tauri dev (see docs/MANUAL_QA_VUE_SHELL.md)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RUN_E2E=false
RUN_VERIFY=false
RUN_ALL_UNIT=false

for arg in "$@"; do
  # Ignore shell comments accidentally pasted after the command (e.g. pnpm test:auto:ci # note)
  if [[ "$arg" == \#* ]]; then
    break
  fi
  case "$arg" in
    --e2e) RUN_E2E=true ;;
    --all) RUN_ALL_UNIT=true ;;
    --verify) RUN_VERIFY=true ;;
    --full) RUN_E2E=true; RUN_ALL_UNIT=true; RUN_VERIFY=true ;;
    -h|--help)
      echo "Usage: $0 [--e2e] [--all] [--verify] [--full]"
      echo "Run one command per line. Do not paste inline # comments with pnpm."
      exit 0
      ;;
    --) break ;;
    -*)
      echo "Unknown option: $arg" >&2
      echo "Usage: $0 [--e2e] [--all] [--verify] [--full]" >&2
      exit 1
      ;;
    *)
      echo "Ignoring extra argument (use flags only): $arg" >&2
      ;;
  esac
done

echo "════════════════════════════════════════"
echo " Exodus test:auto — $(date '+%Y-%m-%d %H:%M:%S')"
echo "════════════════════════════════════════"

echo ""
echo "▶ Vue shell unit regression..."
pnpm test:vue-shell

echo ""
echo "▶ Firefox-style sidebar regression..."
pnpm test:sidebar

if [[ "$RUN_ALL_UNIT" == "true" ]] || [[ "${RUN_ALL_UNIT:-}" == "1" ]]; then
  echo ""
  echo "▶ Full Vitest suite..."
  pnpm test
fi

if [[ "$RUN_E2E" == "true" ]]; then
  echo ""
  echo "▶ Playwright E2E (dedicated Vite @ :1431)..."
  CI=1 pnpm exec playwright test \
    e2e/vue-shell-qa.spec.ts \
    e2e/sidebar-firefox.spec.ts \
    e2e/bookmarks.spec.ts \
    e2e/chrome-internal.spec.ts \
    --project=chromium \
    --workers=1
fi

if [[ "$RUN_VERIFY" == "true" ]]; then
  echo ""
  echo "▶ verify-quick (typecheck + integration + Rust subset)..."
  sh scripts/verify-quick.sh
fi

echo ""
echo "════════════════════════════════════════"
echo " ✓ Automated tests passed"
if [[ "$RUN_E2E" == "false" ]]; then
  echo "   Tip: run with --e2e for browser UI smoke tests"
fi
if [[ "$RUN_VERIFY" == "false" ]]; then
  echo "   Tip: run with --verify for Rust + integration gate"
fi
echo "   Manual Tauri QA: pnpm tauri dev → docs/MANUAL_QA_VUE_SHELL.md"
echo "════════════════════════════════════════"
