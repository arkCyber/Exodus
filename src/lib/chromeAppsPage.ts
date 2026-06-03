/**
 * Exodus Browser — chrome://apps tile model and built-in app shortcuts.
 */

import { logStartup } from '@/lib/startupLog';
import type { ExtensionInfo } from '@/lib/extensions/types';
import type { BookmarkBarLocale } from '@/lib/bookmarkBarUi';
import { extensionIconLetter } from '@/lib/extensionToolbarIcon';

logStartup('chromeAppsPage module loaded');

/** Single tile on the Chrome-style apps page. */
export type ChromeAppTileKind = 'builtin' | 'extension';

export interface ChromeAppTile {
  id: string;
  kind: ChromeAppTileKind;
  name: string;
  /** Navigate target for built-in shortcuts. */
  url?: string;
  /** Extension with browser action popup. */
  extensionId?: string;
  hasPopup?: boolean;
  iconLetter: string;
}

export interface ChromeAppsPageStrings {
  pageTitle: string;
  empty: string;
  manageExtensions: string;
  openSettings: string;
  openBookmarks: string;
  builtInExtensions: string;
  builtInSettings: string;
  builtInBookmarks: string;
}

const EN: ChromeAppsPageStrings = {
  pageTitle: 'Apps',
  empty: 'No apps yet. Install extensions to see them here.',
  manageExtensions: 'Manage extensions',
  openSettings: 'Settings',
  openBookmarks: 'Bookmarks',
  builtInExtensions: 'Extensions',
  builtInSettings: 'Settings',
  builtInBookmarks: 'Bookmarks',
};

const ZH: ChromeAppsPageStrings = {
  pageTitle: '应用',
  empty: '暂无应用。安装扩展后将显示在此处。',
  manageExtensions: '管理扩展程序',
  openSettings: '设置',
  openBookmarks: '书签',
  builtInExtensions: '扩展程序',
  builtInSettings: '设置',
  builtInBookmarks: '书签',
};

/** Localized chrome://apps copy. */
export function chromeAppsPageStrings(locale?: BookmarkBarLocale): ChromeAppsPageStrings {
  if (locale === 'zh') return ZH;
  if (locale === 'en') return EN;
  if (typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('zh')) {
    return ZH;
  }
  return EN;
}

/** First grapheme for a tile icon fallback. */
export function appTileIconLetter(name: string): string {
  return extensionIconLetter(name);
}

/** Built-in shortcuts shown at the top of the apps grid (Chrome parity). */
export function builtinChromeAppTiles(locale?: BookmarkBarLocale): ChromeAppTile[] {
  const s = chromeAppsPageStrings(locale);
  return [
    {
      id: 'builtin-extensions',
      kind: 'builtin',
      name: s.builtInExtensions,
      url: 'chrome://extensions',
      iconLetter: 'E',
    },
    {
      id: 'builtin-settings',
      kind: 'builtin',
      name: s.builtInSettings,
      url: 'chrome://settings',
      iconLetter: 'S',
    },
    {
      id: 'builtin-bookmarks',
      kind: 'builtin',
      name: s.builtInBookmarks,
      url: 'chrome://bookmarks',
      iconLetter: 'B',
    },
  ];
}

/**
 * Build apps grid tiles: built-ins first, then enabled extensions (alphabetical).
 */
export function buildChromeAppTiles(
  extensions: ExtensionInfo[],
  locale?: BookmarkBarLocale,
): ChromeAppTile[] {
  const builtin = builtinChromeAppTiles(locale);
  const extTiles: ChromeAppTile[] = extensions
    .filter((ext) => ext.enabled)
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((ext) => ({
      id: `ext-${ext.id}`,
      kind: 'extension' as const,
      name: ext.name,
      extensionId: ext.id,
      hasPopup: Boolean(ext.actionPopup),
      iconLetter: appTileIconLetter(ext.name),
    }));
  return [...builtin, ...extTiles];
}
