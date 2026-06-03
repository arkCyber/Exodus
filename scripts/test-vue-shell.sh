#!/usr/bin/env bash
# Exodus Browser — Vue shell migration regression subset (Safe Browsing, omnibox, shields, etc.).
# Usage: ./scripts/test-vue-shell.sh   or   pnpm test:vue-shell

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run \
  src/composables/useSafeBrowsingNavigation.test.ts \
  src/components/SafeBrowsingPrompt.test.ts \
  src/views/BrowserPage.test.ts \
  src/composables/useOmnibox.test.ts \
  src/composables/useCdnPageStatus.test.ts \
  src/components/settings/PrivacyShieldSettings.test.ts \
  src/components/AddressBar.test.ts \
  src/composables/useSiteShields.test.ts \
  src/composables/useBrowserTabLifecycle.test.ts \
  src/composables/useClosedTabs.test.ts \
  src/composables/useConfirmDialog.test.ts \
  src/composables/useBrowserSession.test.ts \
  src/components/ConfirmPrompt.test.ts \
  src/components/BrowserSitePermissionPrompt.test.ts \
  src/components/settings/BrowserSitePermissionsSettings.test.ts \
  src/components/FindBar.test.ts \
  src/lib/omnibox.test.ts

echo "✓ Vue shell regression tests passed"
