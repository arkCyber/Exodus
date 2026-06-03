#!/usr/bin/env bash
# Exodus Browser — Chrome-style settings unit + E2E tests.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pnpm vitest run \
  src/lib/appVersion.test.ts \
  src/lib/chromeSettingsSectionUi.test.ts \
  src/lib/clearBrowsingDataSettingsUi.test.ts \
  src/lib/passwordManagerSettingsUi.test.ts \
  src/lib/historyManagerSettingsUi.test.ts \
  src/lib/extensionsSettingsUi.test.ts \
  src/components/settings/DownloadsSettings.test.ts \
  src/components/settings/ClearBrowsingDataSettings.test.ts \
  src/lib/settingsCloseNavigation.test.ts \
  src/components/ExtensionsSettings.test.ts \
  src/components/ChromeSettingsPage.extensions.test.ts \
  src/lib/appLocale.test.ts \
  src/lib/appearanceSettingsUi.test.ts \
  src/lib/chromeSettingsNav.test.ts \
  src/lib/chromeSettingsDeepLink.test.ts \
  src/composables/useChromeSettingsAutoSave.test.ts \
  src/composables/useTheme.test.ts \
  src/components/settings/AppearancePreferencesSettings.test.ts \
  src/components/settings/PasswordManagerSettings.smoke.test.ts \
  src/components/settings/HistoryManagerSettings.smoke.test.ts \
  src/components/settings/PrivacyShieldSettings.test.ts \
  src/components/ChromeSettingsPage.test.ts \
  src/components/SettingsModal.test.ts \
  src/components/ChromeInternalView.test.ts

pnpm exec playwright test e2e/chrome-settings.spec.ts e2e/chrome-internal.spec.ts e2e/privacy.spec.ts --project=chromium --workers=1

echo "✓ Settings tests passed"
