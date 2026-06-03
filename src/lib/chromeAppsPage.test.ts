/**
 * chromeAppsPage — tile model and built-in shortcuts for chrome://apps.
 */

import { describe, it, expect } from 'vitest';
import {
  appTileIconLetter,
  buildChromeAppTiles,
  builtinChromeAppTiles,
  chromeAppsPageStrings,
} from './chromeAppsPage';
import type { ExtensionInfo } from '@/lib/extensions/types';

describe('chromeAppsPage', () => {
  it('returns localized strings for en and zh', () => {
    expect(chromeAppsPageStrings('en').pageTitle).toBe('Apps');
    expect(chromeAppsPageStrings('zh').pageTitle).toBe('应用');
  });

  it('builds icon letter from app name', () => {
    expect(appTileIconLetter('hello')).toBe('H');
    expect(appTileIconLetter('')).toBe('?');
  });

  it('includes built-in shortcuts before extensions', () => {
    const extensions: ExtensionInfo[] = [
      {
        id: 'ext-a',
        name: 'Alpha',
        enabled: true,
        actionPopup: 'popup.html',
      } as ExtensionInfo,
      {
        id: 'ext-b',
        name: 'Beta',
        enabled: false,
        actionPopup: null,
      } as ExtensionInfo,
    ];
    const tiles = buildChromeAppTiles(extensions, 'en');
    expect(tiles[0]?.id).toBe('builtin-extensions');
    expect(tiles[1]?.id).toBe('builtin-settings');
    expect(tiles[2]?.id).toBe('builtin-bookmarks');
    expect(tiles.some((t) => t.id === 'ext-ext-a')).toBe(true);
    expect(tiles.some((t) => t.id === 'ext-ext-b')).toBe(false);
  });

  it('lists builtin tiles with chrome internal URLs', () => {
    const builtin = builtinChromeAppTiles('en');
    expect(builtin.map((t) => t.url)).toEqual([
      'chrome://extensions',
      'chrome://settings',
      'chrome://bookmarks',
    ]);
  });
});
