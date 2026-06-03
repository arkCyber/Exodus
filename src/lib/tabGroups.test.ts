/**
 * Exodus Browser — tab group helper tests.
 */
import { describe, expect, it } from 'vitest';
import { sortTabsWithGroups, tabGroupColorCss } from './tabGroups';

describe('tabGroups', () => {
  it('tabGroupColorCss maps colors', () => {
    expect(tabGroupColorCss('blue')).toBe('#3b82f6');
    expect(tabGroupColorCss('Grey')).toBe('#6b7280');
  });

  it('sortTabsWithGroups keeps pinned first and groups tabs', () => {
    const tabs = [
      { id: 'a', pinned: true },
      { id: 'b' },
      { id: 'c' },
      { id: 'd' },
    ];
    const groups = [
      {
        id: 'g1',
        title: 'Work',
        color: 'blue',
        tab_ids: ['d', 'c'],
        created_at: 0,
        last_modified: 0,
        collapsed: false,
      },
    ];
    expect(sortTabsWithGroups(tabs, groups, 'b')).toEqual(['a', 'd', 'c', 'b']);
  });

  it('sortTabsWithGroups hides collapsed group tabs except active', () => {
    const tabs = [{ id: 'x' }, { id: 'y' }, { id: 'z' }];
    const groups = [
      {
        id: 'g1',
        title: 'G',
        color: 'red',
        tab_ids: ['x', 'y'],
        created_at: 0,
        last_modified: 0,
        collapsed: true,
      },
    ];
    expect(sortTabsWithGroups(tabs, groups, 'z')).toEqual(['z']);
    expect(sortTabsWithGroups(tabs, groups, 'y')).toEqual(['y', 'z']);
  });
});
