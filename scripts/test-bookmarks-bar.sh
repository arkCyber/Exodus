#!/usr/bin/env bash
# Exodus Browser — bookmark bar unit + E2E tests.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run \
  src/lib/bookmarks.test.ts \
  src/lib/bookmarkGroups.test.ts \
  src/lib/bookmarkBarUi.test.ts \
  src/lib/bookmarkBackendSync.test.ts \
  src/lib/chromeLayout.test.ts \
  src/lib/chromeAppsPage.test.ts \
  src/lib/extensionToolbarIcon.test.ts \
  src/components/BookmarkBar.test.ts \
  src/components/BookmarkEditor.test.ts \
  src/components/ExtensionActionBar.test.ts \
  src/components/ChromeInternalView.test.ts

pnpm exec playwright test e2e/bookmarks.spec.ts e2e/bookmark-groups.spec.ts e2e/chrome-internal.spec.ts --project=chromium --workers=1

echo "✓ Bookmark bar tests passed"
