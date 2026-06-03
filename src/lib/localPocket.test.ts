/**
 * Local Pocket / reading list unit tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { pocketListReadingList } from './localPocket';

describe('pocketListReadingList', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('returns unread non-archived items tagged reading-list', async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: '1',
        url: 'https://a.com',
        title: 'A',
        content: '',
        excerpt: '',
        author: null,
        tags: ['reading-list'],
        saved_at: '2026-01-01',
        read_at: null,
        is_favorite: false,
        is_archived: false,
        reading_time_minutes: 1,
        word_count: 10,
      },
      {
        id: '2',
        url: 'https://b.com',
        title: 'B',
        content: '',
        excerpt: '',
        author: null,
        tags: ['pocket'],
        saved_at: '2026-01-01',
        read_at: null,
        is_favorite: false,
        is_archived: false,
        reading_time_minutes: 1,
        word_count: 10,
      },
      {
        id: '3',
        url: 'https://c.com',
        title: 'C',
        content: '',
        excerpt: '',
        author: null,
        tags: ['reading-list'],
        saved_at: '2026-01-01',
        read_at: '2026-01-02',
        is_favorite: false,
        is_archived: false,
        reading_time_minutes: 1,
        word_count: 10,
      },
    ]);

    const list = await pocketListReadingList();
    expect(list).toHaveLength(1);
    expect(list[0]?.id).toBe('1');
    expect(invokeMock).toHaveBeenCalledWith('pocket_list_articles');
  });
});
