/**
 * Exodus Browser — historyManager unit tests.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  addManagedHistoryEntry,
  getRecentManagedHistory,
  mergeBrowsingHistoryLists,
} from './historyManager';
import type { HistoryPage } from './browserTypes';

describe('historyManager', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('addManagedHistoryEntry invokes backend', async () => {
    invokeMock.mockResolvedValue(undefined);
    await addManagedHistoryEntry('https://example.com', 'Example');
    expect(invokeMock).toHaveBeenCalledWith('add_history_entry', {
      url: 'https://example.com',
      title: 'Example',
    });
  });

  it('getRecentManagedHistory returns entries', async () => {
    invokeMock.mockResolvedValue([{ id: '1', url: 'https://a.com', title: 'A' }]);
    const rows = await getRecentManagedHistory(10);
    expect(rows).toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith('get_recent_history', { limit: 10 });
  });

  it('mergeBrowsingHistoryLists dedupes by URL', () => {
    const visits: HistoryPage[] = [
      {
        id: 'v1',
        url: 'https://example.com',
        title: 'Visit',
        timestamp: '2026-01-01T00:00:00.000Z',
        visit_count: 2,
      },
    ];
    const merged = mergeBrowsingHistoryLists(visits, [
      {
        id: 'm1',
        url: 'https://example.com',
        title: 'Managed',
        visit_time: 0,
        visit_count: 5,
        last_visit: Math.floor(new Date('2026-05-01T00:00:00Z').getTime() / 1000),
        transition_type: 'typed',
      },
    ]);
    expect(merged).toHaveLength(1);
    expect(merged[0].title).toBe('Managed');
    expect(merged[0].visit_count).toBe(5);
  });
});
