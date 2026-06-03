/**
 * Exodus Browser — Chrome / Brave layout tokens (px) for tests and JS layout math.
 */

/** Canonical chrome dimensions aligned with Chrome / Brave desktop UI. */
export const CHROME_LAYOUT = {
  tabBarHeight: 34,
  toolbarHeight: 48,
  bookmarkBarHeight: 32,
  bookmarkItemHeight: 22,
  bookmarkFaviconSize: 16,
  omniboxHeight: 34,
  toolbarButtonSize: 32,
  toolbarIconSize: 16,
  extensionBarHeight: 32,
  sidebarContentWidth: 320,
  sidebarIconRail: 48,
  ntpSearchHeight: 44,
  ntpTileIcon: 32,
} as const;

export type ChromeLayoutTokens = typeof CHROME_LAYOUT;

/** Vertical chrome row order (matches Chrome: tabs → toolbar with inline extensions → bookmarks). */
export const CHROME_ROW_ORDER = [
  'tabstrip',
  'toolbar',
  'bookmarks',
] as const;

/** Sum of default visible chrome rows (tabs + toolbar + bookmarks). */
export function defaultChromeChromeHeightPx(options?: {
  showBookmarkBar?: boolean;
  showExtensionBar?: boolean;
}): number {
  let h = CHROME_LAYOUT.tabBarHeight + CHROME_LAYOUT.toolbarHeight;
  if (options?.showBookmarkBar !== false) h += CHROME_LAYOUT.bookmarkBarHeight;
  if (options?.showExtensionBar) h += CHROME_LAYOUT.extensionBarHeight;
  return h;
}
