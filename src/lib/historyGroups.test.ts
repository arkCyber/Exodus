/**
 * Exodus Browser — history grouping unit tests.
 */

import { describe, expect, it } from 'vitest';
import { groupHistoryByDate } from './historyGroups';
import type { HistoryPage } from './browserTypes';

describe('groupHistoryByDate', () => {
  it('groups pages under Today', () => {
    const now = new Date().toISOString();
    const pages: HistoryPage[] = [
      { id: '1', url: 'https://a.com', title: 'A', timestamp: now },
      { id: '2', url: 'https://b.com', title: 'B', timestamp: now },
    ];
    const groups = groupHistoryByDate(pages);
    expect(groups.length).toBeGreaterThan(0);
    expect(groups[0].label).toBe('Today');
    expect(groups[0].pages.length).toBe(2);
  });

  it('returns empty for no pages', () => {
    expect(groupHistoryByDate([])).toEqual([]);
  });
});
