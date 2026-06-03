/**
 * Exodus Browser — shared NTP layout mode (first-run defaults vs user customized).
 * Central storage keys for top sites, quick links, and customization flag.
 */

/** User-added top sites (custom order). */
export const NTP_TOP_SITES_STORAGE_KEY = 'exodus-top-sites';

/** Pinned top sites (shown first on the grid). */
export const NTP_PINNED_SITES_STORAGE_KEY = 'exodus-pinned-sites';

/** Bundled sites the user removed from the grid. */
export const NTP_REMOVED_SITES_STORAGE_KEY = 'exodus-removed-ntp-sites';

/** User-added quick-link chips. */
export const NTP_QUICK_LINKS_STORAGE_KEY = 'exodus-ntp-quick-links';

/** Bundled chips the user removed. */
export const NTP_REMOVED_QUICK_LINKS_STORAGE_KEY = 'exodus-removed-ntp-quick-links';

/** Fast-path flag set on first user edit. */
export const NTP_LAYOUT_CUSTOMIZED_KEY = 'exodus-ntp-layout-customized-v1';

const CUSTOMIZATION_KEYS = [
  NTP_TOP_SITES_STORAGE_KEY,
  NTP_PINNED_SITES_STORAGE_KEY,
  NTP_REMOVED_SITES_STORAGE_KEY,
  NTP_QUICK_LINKS_STORAGE_KEY,
  NTP_REMOVED_QUICK_LINKS_STORAGE_KEY,
  NTP_LAYOUT_CUSTOMIZED_KEY,
] as const;

/** Whether the user has customized the new-tab page (any add/remove). */
export function isNtpLayoutCustomized(): boolean {
  try {
    if (localStorage.getItem(NTP_LAYOUT_CUSTOMIZED_KEY) === '1') return true;
    return CUSTOMIZATION_KEYS.some(
      (key) => key !== NTP_LAYOUT_CUSTOMIZED_KEY && localStorage.getItem(key) != null,
    );
  } catch {
    return false;
  }
}

/** Mark the layout as user-customized (disables auto-refill). */
export function markNtpLayoutCustomized(): void {
  try {
    localStorage.setItem(NTP_LAYOUT_CUSTOMIZED_KEY, '1');
  } catch (error) {
    console.error('[ntpLayoutStore] mark customized failed:', error);
  }
}

/** Clear the customized flag (call after wiping all NTP layout storage). */
export function clearNtpLayoutCustomizedFlag(): void {
  try {
    localStorage.removeItem(NTP_LAYOUT_CUSTOMIZED_KEY);
  } catch (error) {
    console.error('[ntpLayoutStore] clear flag failed:', error);
  }
}

/** Wipe all NTP layout storage keys (factory reset). */
export function clearAllNtpLayoutStorage(): void {
  try {
    for (const key of CUSTOMIZATION_KEYS) {
      localStorage.removeItem(key);
    }
  } catch (error) {
    console.error('[ntpLayoutStore] clear all storage failed:', error);
  }
}

/** Factory reset: clear storage and return to first-run defaults on next build. */
export function resetAllNtpLayout(): void {
  clearAllNtpLayoutStorage();
}
