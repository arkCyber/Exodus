/**
 * Unit tests for Chrome/Brave layout tokens.
 */

import { describe, it, expect } from 'vitest';
import {
  CHROME_LAYOUT,
  CHROME_ROW_ORDER,
  defaultChromeChromeHeightPx,
} from './chromeLayout';

describe('CHROME_LAYOUT', () => {
  it('uses compact tab bar height (Chrome ~34px)', () => {
    expect(CHROME_LAYOUT.tabBarHeight).toBe(34);
    expect(CHROME_LAYOUT.tabBarHeight).toBeLessThanOrEqual(36);
  });

  it('uses Chrome-aligned toolbar, omnibox, and icon sizes', () => {
    expect(CHROME_LAYOUT.toolbarHeight).toBe(48);
    expect(CHROME_LAYOUT.omniboxHeight).toBe(34);
    expect(CHROME_LAYOUT.toolbarButtonSize).toBe(32);
    expect(CHROME_LAYOUT.toolbarIconSize).toBe(16);
  });

  it('uses compact bookmark bar', () => {
    expect(CHROME_LAYOUT.bookmarkBarHeight).toBe(32);
    expect(CHROME_LAYOUT.bookmarkBarHeight).toBeLessThan(36);
  });

  it('uses compact new-tab tile icons', () => {
    expect(CHROME_LAYOUT.ntpTileIcon).toBe(32);
    expect(CHROME_LAYOUT.ntpTileIcon).toBeLessThan(48);
  });

  it('defines Chrome row order (tabs → toolbar with inline extensions → bookmarks)', () => {
    expect(CHROME_ROW_ORDER).toEqual(['tabstrip', 'toolbar', 'bookmarks']);
  });

  it('defaultChromeChromeHeightPx sums chrome rows', () => {
    expect(defaultChromeChromeHeightPx()).toBe(34 + 48 + 32);
    expect(defaultChromeChromeHeightPx({ showBookmarkBar: false })).toBe(34 + 48);
    expect(
      defaultChromeChromeHeightPx({ showExtensionBar: true }),
    ).toBe(34 + 48 + 32 + 32);
  });
});
