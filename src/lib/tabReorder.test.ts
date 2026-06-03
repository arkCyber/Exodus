import { describe, it, expect } from 'vitest';
import { reorderTabsById } from './tabReorder';

describe('reorderTabsById', () => {
  const tabs = [
    { id: 'a', pinned: true },
    { id: 'b', pinned: true },
    { id: 'c' },
    { id: 'd' },
  ];

  it('moves unpinned tab before another unpinned tab', () => {
    const next = reorderTabsById(tabs, 'd', 'c');
    expect(next.map((t) => t.id)).toEqual(['a', 'b', 'd', 'c']);
  });

  it('does not move pinned tab among unpinned', () => {
    const next = reorderTabsById(tabs, 'a', 'c');
    expect(next.map((t) => t.id)).toEqual(['a', 'b', 'c', 'd']);
  });
});
