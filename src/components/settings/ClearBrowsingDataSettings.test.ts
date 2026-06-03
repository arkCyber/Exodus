/**
 * Exodus Browser — ClearBrowsingDataSettings component tests.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import ClearBrowsingDataSettings from './ClearBrowsingDataSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
}));

vi.mock('$lib/browserIntegrations', () => ({
  clearBrowsingData: vi.fn(async () => 'cookies, history'),
}));

import { clearBrowsingData } from '$lib/browserIntegrations';

describe('ClearBrowsingDataSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders clear data checkboxes', async () => {
    const wrapper = mount(ClearBrowsingDataSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="clear-browsing-data-panel"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="clear-data-cookies"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="clear-data-history"]').exists()).toBe(true);
  });

  it('invokes clearBrowsingData on submit', async () => {
    const wrapper = mount(ClearBrowsingDataSettings);
    await flushPromises();
    await wrapper.find('[data-testid="clear-data-submit"]').trigger('click');
    await flushPromises();
    expect(clearBrowsingData).toHaveBeenCalledWith(
      expect.objectContaining({ clearCookies: true, clearHistory: true }),
    );
  });

  it('shows Chinese title for zh locale', async () => {
    const wrapper = mount(ClearBrowsingDataSettings, { props: { uiLocale: 'zh' } });
    await flushPromises();
    expect(wrapper.text()).toContain('清除浏览数据');
  });
});
