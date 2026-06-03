/**
 * SidebarSyncedTabsPanel tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SidebarSyncedTabsPanel from './SidebarSyncedTabsPanel.vue';

vi.mock('$lib/syncedTabs', () => ({
  refreshSyncedTabs: vi.fn(async (openTabs: { id: string; title: string; url: string }[]) => [
    {
      deviceId: 'this-device',
      deviceName: 'This computer',
      tabs: openTabs.map((t) => ({
        id: t.id,
        title: t.title,
        url: t.url,
        lastActive: '2026-01-01',
      })),
    },
  ]),
}));

vi.mock('$lib/mobileSync', () => ({
  isMobileSyncEnabled: vi.fn(async () => false),
}));

import { refreshSyncedTabs } from '$lib/syncedTabs';

describe('SidebarSyncedTabsPanel', () => {
  beforeEach(() => {
    vi.mocked(refreshSyncedTabs).mockClear();
  });

  it('loads devices on mount', async () => {
    const wrapper = mount(SidebarSyncedTabsPanel, {
      props: { openTabs: [{ id: 't1', title: 'Tab', url: 'https://example.com' }] },
    });
    await flushPromises();
    expect(wrapper.text()).toContain('This computer');
    expect(wrapper.text()).toContain('example.com');
  });

  it('refreshes when openTabs change', async () => {
    const wrapper = mount(SidebarSyncedTabsPanel, {
      props: { openTabs: [] },
    });
    await flushPromises();
    expect(refreshSyncedTabs).toHaveBeenCalledTimes(1);

    await wrapper.setProps({
      openTabs: [{ id: 't2', title: 'New', url: 'https://new.test' }],
    });
    await flushPromises();
    expect(vi.mocked(refreshSyncedTabs).mock.calls.length).toBeGreaterThanOrEqual(2);
    expect(wrapper.text()).toContain('new.test');
  });
});
