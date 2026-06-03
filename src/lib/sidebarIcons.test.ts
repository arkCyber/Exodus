/**
 * Unit tests for Firefox-style sidebar icons.
 */

import { describe, expect, it } from 'vitest';
import { sidebarIconItemsFromPrefs, sidebarIconPath } from './sidebarIcons';
import { loadSidebarPreferences } from './sidebarPreferences';

describe('sidebarIcons', () => {
  it('builds icon items from preferences', () => {
    const items = sidebarIconItemsFromPrefs(loadSidebarPreferences());
    expect(items.map((p) => p.panel)).toContain('tabs');
    expect(items.map((p) => p.panel)).toContain('synced');
  });

  it('returns non-empty SVG paths for each icon', () => {
    const items = sidebarIconItemsFromPrefs(loadSidebarPreferences());
    for (const { icon } of items) {
      expect(sidebarIconPath(icon).length).toBeGreaterThan(10);
    }
    expect(sidebarIconPath('close').length).toBeGreaterThan(5);
  });
});
