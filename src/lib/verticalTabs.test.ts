/**
 * Exodus Browser — verticalTabs unit tests.
 */
import { describe, expect, it } from 'vitest';
import { isVerticalTabsRight, verticalTabStripWidth, type VerticalTabSettings } from './verticalTabs';

const base: VerticalTabSettings = {
  enabled: true,
  position: 'Left',
  width_mode: 'Auto',
  fixed_width: 280,
  show_icons: true,
  show_titles: true,
  show_close_buttons: true,
  collapse_inactive: false,
  tab_spacing: 4,
};

describe('verticalTabs', () => {
  it('verticalTabStripWidth respects fixed mode', () => {
    expect(verticalTabStripWidth({ ...base, width_mode: 'Fixed', fixed_width: 300 })).toBe(300);
    expect(verticalTabStripWidth({ ...base, width_mode: 'Compact' })).toBe(140);
  });

  it('isVerticalTabsRight detects right position', () => {
    expect(isVerticalTabsRight({ ...base, position: 'Right' })).toBe(true);
    expect(isVerticalTabsRight({ ...base, position: 'Left' })).toBe(false);
  });
});
