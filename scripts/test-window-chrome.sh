#!/usr/bin/env bash
# Exodus Browser — window chrome drag + toolbar interaction tests.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run src/lib/windowDrag.test.ts src/components/AddressBar.test.ts src/components/WindowTitleBar.test.ts
pnpm exec playwright test e2e/window-chrome.spec.ts --project=chromium

echo "✓ Window chrome tests passed"
