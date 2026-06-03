/**
 * Exodus Browser — BrowserSitePermissionsSettings tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import BrowserSitePermissionsSettings from './BrowserSitePermissionsSettings.vue';

vi.mock('$lib/extensions/api', () => ({
  listBrowserSitePermissions: vi.fn(async () => [
    { origin: 'https://a.test', kind: 'camera', granted: true },
  ]),
  revokeBrowserSitePermission: vi.fn(async () => {}),
}));

describe('BrowserSitePermissionsSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loads and lists site permissions', async () => {
    const wrapper = mount(BrowserSitePermissionsSettings);
    await flushPromises();
    expect(wrapper.text()).toContain('https://a.test');
    expect(wrapper.text()).toContain('Camera');
  });
});
