/**
 * Tests for indexed memory grouping.
 */
import { describe, expect, it } from 'vitest';
import { groupIndexedByDate } from './indexedMemory';

describe('groupIndexedByDate', () => {
  it('returns empty array when no pages', () => {
    expect(groupIndexedByDate([])).toEqual([]);
  });

  it('groups pages under Today when timestamp is today', () => {
    const now = new Date().toISOString();
    const groups = groupIndexedByDate([
      { id: '1', url: 'https://a.test', title: 'A', timestamp: now },
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0].label).toBe('Today');
    expect(groups[0].pages).toHaveLength(1);
  });

  it('groups multiple pages on the same day', () => {
    const now = new Date().toISOString();
    const groups = groupIndexedByDate([
      { id: '1', url: 'https://a.test', title: 'A', timestamp: now },
      { id: '2', url: 'https://b.test', title: 'B', timestamp: now },
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0].pages).toHaveLength(2);
  });
});
