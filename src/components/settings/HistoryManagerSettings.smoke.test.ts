/**
 * Exodus Browser — HistoryManagerSettings smoke tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import HistoryManagerSettings from './HistoryManagerSettings.vue';

vi.mock('$lib/historyManager', () => ({
  clearAllManagedHistory: vi.fn(),
  getManagedHistoryStats: vi.fn(async () => ({ total_entries: 0, unique_domains: 0 })),
  getRecentManagedHistory: vi.fn(async () => []),
  loadHistoryManagerSettings: vi.fn(async () => ({
    enabled: true,
    remember_browsing: true,
    retention_days: 90,
  })),
  removeManagedHistoryEntry: vi.fn(),
  saveHistoryManagerSettings: vi.fn(),
  searchManagedHistory: vi.fn(async () => []),
}));

describe('HistoryManagerSettings smoke', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders history panel after load', async () => {
    const wrapper = mount(HistoryManagerSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="history-manager-panel"]').exists()).toBe(true);
    expect(wrapper.text()).toContain('Browsing history');
  });

  it('renders Chinese title for zh locale', async () => {
    const wrapper = mount(HistoryManagerSettings, { props: { uiLocale: 'zh' } });
    await flushPromises();
    expect(wrapper.text()).toContain('浏览历史记录');
  });
});
