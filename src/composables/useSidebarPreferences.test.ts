/**
 * useSidebarPreferences composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';

vi.mock('$lib/verticalTabs', () => ({
  loadVerticalTabSettings: vi.fn(async () => ({
    enabled: false,
    position: 'Right',
    width_mode: 'Auto',
    fixed_width: 220,
    show_icons: true,
    show_titles: true,
    show_close_buttons: true,
    collapse_inactive: false,
    tab_spacing: 4,
  })),
  saveVerticalTabSettings: vi.fn(async () => undefined),
}));

import { useSidebarPreferences } from './useSidebarPreferences';

describe('useSidebarPreferences', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it('resolvePanel falls back when tool disabled', () => {
    const { prefs, updatePrefs, resolvePanel } = useSidebarPreferences();
    updatePrefs({ enabledTools: ['bookmarks'] });
    expect(resolvePanel('pocket')).toBe('bookmarks');
    expect(prefs.value.enabledTools).toEqual(['bookmarks']);
  });

  it('disables vertical tabs when tabs tool is unchecked', async () => {
    const { prefs, updatePrefs, toggleTool } = useSidebarPreferences();
    updatePrefs({ verticalTabsInSidebar: true, enabledTools: ['tabs', 'ai'] });
    toggleTool('tabs');
    expect(prefs.value.enabledTools).not.toContain('tabs');
    expect(prefs.value.verticalTabsInSidebar).toBe(false);
  });
});
