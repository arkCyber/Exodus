/**
 * Exodus Browser — bookmark helper tests.
 */

import { describe, expect, it } from 'vitest';
import type { BookmarkItem } from '$lib/browserTypes';
import { bookmarksOnBar, bookmarkFolderNames, bookmarksInFolder, BOOKMARK_BAR_MAX } from '$lib/bookmarks';

const sample: BookmarkItem[] = [
  { id: '1', url: 'https://a.com', title: 'A', created_at: '', folder: '' },
  { id: '2', url: 'https://b.com', title: 'B', created_at: '', folder: 'Work' },
  { id: '3', url: 'https://c.com', title: 'C', created_at: '', folder: 'Work' },
];

describe('bookmarks', () => {
  it('BOOKMARK_BAR_MAX is 10', () => {
    expect(BOOKMARK_BAR_MAX).toBe(10);
  });

  it('bookmarksOnBar excludes folders', () => {
    expect(bookmarksOnBar(sample)).toHaveLength(1);
    expect(bookmarksOnBar(sample)[0].id).toBe('1');
  });

  it('bookmarksOnBar sorts by bar_order', () => {
    const items: BookmarkItem[] = [
      { id: 'a', url: 'https://a', title: 'A', created_at: '', bar_order: 2 },
      { id: 'b', url: 'https://b', title: 'B', created_at: '', bar_order: 0 },
    ];
    expect(bookmarksOnBar(items).map((b) => b.id)).toEqual(['b', 'a']);
  });

  it('bookmarkFolderNames returns sorted unique folders', () => {
    expect(bookmarkFolderNames(sample)).toEqual(['Work']);
  });

  it('bookmarksInFolder filters by folder', () => {
    expect(bookmarksInFolder(sample, 'Work')).toHaveLength(2);
  });
});
