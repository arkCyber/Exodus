/**
 * Unit tests for new-tab top sites helper.
 */

import { describe, expect, it } from 'vitest';
import { buildTopSitesFromHistory } from './newTabTopSites';
import type { ManagedHistoryEntry } from './historyManager';

describe('buildTopSitesFromHistory', () => {
  it('dedupes by hostname and returns QuickLink rows', () => {
    const entries: ManagedHistoryEntry[] = [
      {
        id: '1',
        url: 'https://news.ycombinator.com/item',
        title: 'HN Item',
        visit_time: 1,
        visit_count: 5,
        last_visit: 100,
        transition_type: 'link',
      },
      {
        id: '2',
        url: 'https://github.com/exodus',
        title: 'GitHub',
        visit_time: 2,
        visit_count: 10,
        last_visit: 200,
        transition_type: 'link',
      },
    ];
    const sites = buildTopSitesFromHistory(entries, 4);
    expect(sites).toHaveLength(2);
    expect(sites[0].url).toContain('github.com');
  });
});
