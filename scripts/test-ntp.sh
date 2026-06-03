#!/usr/bin/env bash
# Exodus Browser — new tab page (NTP) unit + E2E tests.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run \
  src/lib/diagnosticLog.test.ts \
  src/lib/tauri.test.ts \
  src/lib/newTabWallpaper.test.ts \
  src/lib/newTabPage.test.ts \
  src/lib/ntpLayoutStore.test.ts \
  src/lib/ntpTopSitesStore.test.ts \
  src/lib/ntpQuickLinksStore.test.ts \
  src/views/BrowserPage.siteManagement.test.ts \
  src/components/NewTabPage.test.ts \
  src/components/settings/NewTabLayoutSettings.test.ts

pnpm exec playwright test e2e/new-tab-page.spec.ts e2e/new-tab-reuse.spec.ts e2e/new-tab-parity.spec.ts e2e/new-tab-wallpaper-tabs.spec.ts e2e/new-tab-top-sites.spec.ts e2e/new-tab-quick-links.spec.ts e2e/new-tab-layout.spec.ts --project=chromium

echo "✓ New tab page tests passed"
