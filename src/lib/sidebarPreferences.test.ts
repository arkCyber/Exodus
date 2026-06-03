/**
 * Unit tests for Firefox-style sidebar preferences.
 */

import { describe, expect, it, beforeEach } from 'vitest';
import {
  applySidebarPreferencesPatch,
  defaultSidebarPanel,
  loadSidebarPreferences,
  saveSidebarPreferences,
  isToolEnabled,
  SIDEBAR_TOOL_CATALOG,
} from './sidebarPreferences';

describe('sidebarPreferences', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('loads defaults with all tools enabled', () => {
    const p = loadSidebarPreferences();
    expect(p.position).toBe('right');
    expect(p.enabledTools.length).toBe(SIDEBAR_TOOL_CATALOG.length);
  });

  it('defaults to tabs panel when vertical tabs in sidebar', () => {
    const p = applySidebarPreferencesPatch(loadSidebarPreferences(), {
      verticalTabsInSidebar: true,
    });
    expect(defaultSidebarPanel(p)).toBe('tabs');
  });

  it('toggles tool enablement via patch', () => {
    const p = applySidebarPreferencesPatch(loadSidebarPreferences(), {
      enabledTools: ['ai', 'bookmarks'],
    });
    expect(isToolEnabled(p, 'ai')).toBe(true);
    expect(isToolEnabled(p, 'pocket')).toBe(false);
    saveSidebarPreferences(p);
    expect(loadSidebarPreferences().enabledTools).toEqual(['ai', 'bookmarks']);
  });
});
