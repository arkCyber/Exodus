import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';

vi.mock('$lib/tabGroups', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/tabGroups')>();
  return {
    ...actual,
    listTabGroups: vi.fn(async () => [
      {
        id: 'g1',
        title: 'Work',
        color: 'blue',
        tab_ids: ['t1'],
        created_at: 0,
        last_modified: 0,
        collapsed: false,
      },
    ]),
    createTabGroup: vi.fn(async () => 'g2'),
    addTabToGroup: vi.fn(),
  };
});

import { useBrowserTabGroups } from './useBrowserTabGroups';

describe('useBrowserTabGroups', () => {
  const tabs = ref([
    { id: 't1', title: 'A', url: 'https://a.com', pinned: false },
    { id: 't2', title: 'B', url: 'https://b.com', pinned: false },
  ]);
  const activeTabId = ref('t1');
  const onStatus = vi.fn();

  beforeEach(() => {
    onStatus.mockClear();
  });

  it('loads groups and sorts tabs', async () => {
    const tg = useBrowserTabGroups({
      getTabs: () => tabs.value,
      getActiveTabId: () => activeTabId.value,
      onStatus,
    });
    await tg.loadTabGroups();
    expect(tg.tabGroups.value.length).toBe(1);
    const sorted = tg.sortedTabs.value;
    expect(sorted.map((t) => t.id)).toContain('t1');
  });

  it('opens tab context menu', () => {
    const tg = useBrowserTabGroups({
      getTabs: () => tabs.value,
      getActiveTabId: () => activeTabId.value,
      onStatus,
    });
    tg.openTabContextMenu({ preventDefault: () => {} } as MouseEvent, 't2');
    expect(tg.tabContextMenu.value?.tabId).toBe('t2');
  });
});
