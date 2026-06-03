/**
 * Exodus Browser — chrome:// internal page URLs (Chrome parity routing).
 */

/** Supported chrome:// hosts mapped to in-app UI. */
export type ChromeInternalPage =
  | 'settings'
  | 'extensions'
  | 'apps'
  | 'history'
  | 'bookmarks'
  | 'downloads'
  | 'newtab'
  | 'unknown';

const KNOWN_PAGES = new Set<ChromeInternalPage>([
  'settings',
  'extensions',
  'apps',
  'history',
  'bookmarks',
  'downloads',
  'newtab',
]);

/**
 * Whether a URL is a chrome:// internal page.
 */
export function isChromeInternalUrl(url: string): boolean {
  const trimmed = url.trim().toLowerCase();
  return trimmed.startsWith('chrome://') || trimmed.startsWith('chrome:');
}

/**
 * Normalize omnibox input to a canonical chrome:// URL.
 */
export function normalizeChromeInternalUrl(input: string): string {
  const raw = input.trim().toLowerCase();
  if (raw.startsWith('chrome://')) return raw;
  if (raw.startsWith('chrome:')) {
    const rest = raw.slice('chrome:'.length).replace(/^\/+/, '');
    return `chrome://${rest}`;
  }
  return `chrome://${raw.replace(/^\/+/, '')}`;
}

/**
 * Parse chrome:// host into a known internal page id.
 */
export function parseChromeInternalUrl(url: string): ChromeInternalPage | null {
  if (!isChromeInternalUrl(url)) return null;
  const host = normalizeChromeInternalUrl(url)
    .slice('chrome://'.length)
    .split(/[/?#]/)[0]
    .toLowerCase();
  if (!host) return 'newtab';
  if (host === 'new-tab-page' || host === 'newtab') return 'newtab';
  if (KNOWN_PAGES.has(host as ChromeInternalPage)) return host as ChromeInternalPage;
  return 'unknown';
}

/**
 * Tab / document title for a chrome:// page.
 */
export function chromeInternalTitle(page: ChromeInternalPage): string {
  switch (page) {
    case 'settings':
      return 'Settings';
    case 'extensions':
      return 'Extensions';
    case 'apps':
      return 'Apps';
    case 'history':
      return 'History';
    case 'bookmarks':
      return 'Bookmarks';
    case 'downloads':
      return 'Downloads';
    case 'newtab':
      return 'New Tab';
    default:
      return 'Exodus';
  }
}

/**
 * Vue hash route path for a chrome internal page (e.g. /chrome/settings).
 */
export function chromeInternalRoutePath(page: ChromeInternalPage): string | null {
  if (page === 'unknown' || page === 'newtab') return null;
  return `/chrome/${page}`;
}

/**
 * Map /chrome/:page route param to chrome:// URL.
 */
export function chromeInternalUrlFromRouteParam(page: string): string {
  return normalizeChromeInternalUrl(page);
}
