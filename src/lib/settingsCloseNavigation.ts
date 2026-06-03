/**
 * Exodus Browser — restore target when leaving chrome://settings (close button).
 */

import { logStartup } from '@/lib/startupLog';
import { isChromeInternalUrl } from '@/lib/chromeInternal';
import { isNewTabUrl, NEWTAB_INTERNAL_URL } from '@/lib/newTabPage';
import { isChromeSettingsUrl } from '@/lib/chromeSettingsNav';

logStartup('settingsCloseNavigation module loaded');

export type TabSettingsReturn = {
  url: string;
  settingsReturnUrl?: string | null;
};

/**
 * URL to load after closing settings: prior page if saved, otherwise new tab.
 */
export function resolveSettingsCloseTarget(
  tab: TabSettingsReturn | null | undefined,
  fallbackUrl: string = NEWTAB_INTERNAL_URL,
): string {
  const restore = tab?.settingsReturnUrl?.trim();
  if (!restore) return fallbackUrl;
  if (isChromeInternalUrl(restore) || isNewTabUrl(restore) || isChromeSettingsUrl(restore)) {
    return fallbackUrl;
  }
  return restore;
}

/**
 * Whether to remember `previousUrl` when entering settings/extensions overlay.
 */
export function shouldRememberSettingsReturnUrl(previousUrl: string, settingsSectionHop: boolean): boolean {
  if (settingsSectionHop) return false;
  const trimmed = previousUrl.trim();
  if (!trimmed) return false;
  if (isNewTabUrl(trimmed) || isChromeInternalUrl(trimmed)) return false;
  return true;
}
