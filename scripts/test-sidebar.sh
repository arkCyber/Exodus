#!/usr/bin/env bash
# Exodus Browser — Firefox-style sidebar regression suite.
# Usage: ./scripts/test-sidebar.sh   or   pnpm test:sidebar (if wired in package.json)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm exec vitest run \
  src/lib/sidebarPreferences.test.ts \
  src/lib/sidebarIcons.test.ts \
  src/lib/sidebarLayout.test.ts \
  src/lib/syncedTabs.test.ts \
  src/lib/localPocket.test.ts \
  src/composables/useSidebarPreferences.test.ts \
  src/composables/useBrowserSidebar.test.ts \
  src/composables/useBrowserTabBarHandlers.test.ts \
  src/components/BrowserSidebar.test.ts \
  src/components/sidebar/SidebarCustomizePanel.test.ts \
  src/components/sidebar/SidebarSyncedTabsPanel.test.ts \
  src/components/sidebar/SidebarMemoryPanel.test.ts \
  src/views/browserSidebarIntegration.test.ts \
  src/views/BrowserPage.sidebar.test.ts \
  src/components/sidebar/SidebarReadingListPanel.test.ts \
  src/lib/browserShortcuts.test.ts \
  src/components/AddressBar.test.ts \
  src/components/SettingsModal.test.ts

echo "✓ Sidebar regression tests passed"
