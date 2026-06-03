/**
 * Exodus Browser — ChromeInternalView component tests.
 */

import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import ChromeInternalView from './ChromeInternalView.vue';

vi.mock('@/components/ChromeSettingsPage.vue', () => ({
  default: {
    name: 'ChromeSettingsPage',
    props: ['initialSection', 'contentHost', 'p2pRoomId', 'uiLocale'],
    template: '<div data-testid="chrome-settings-page" class="settings-page-stub">Settings</div>',
    emits: [
      'close',
      'status',
      'saved',
      'extensionsChanged',
      'trackingChanged',
      'openSidebarCustomize',
      'ntpLayoutReset',
      'wallpaperChange',
      'verticalLayoutChange',
      'openPanel',
      'navigate',
      'localeChange',
    ],
  },
}));

vi.mock('@/components/ChromeAppsPage.vue', () => ({
  default: {
    name: 'ChromeAppsPage',
    template: '<div class="chrome-apps-page-stub">Apps grid</div>',
  },
}));

describe('ChromeInternalView', () => {
  it('renders chrome internal page for settings URL', () => {
    const wrapper = mount(ChromeInternalView, {
      props: { url: 'chrome://settings' },
    });
    expect(wrapper.find('.chrome-internal-page').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chrome-settings-page"]').exists()).toBe(true);
  });

  it('renders extensions section for chrome://extensions', () => {
    const wrapper = mount(ChromeInternalView, {
      props: { url: 'chrome://extensions' },
    });
    expect(wrapper.find('[data-testid="chrome-settings-page"]').exists()).toBe(true);
  });

  it('renders independent apps grid for chrome://apps', () => {
    const wrapper = mount(ChromeInternalView, {
      props: { url: 'chrome://apps' },
    });
    expect(wrapper.find('.chrome-internal-page[data-page="apps"]').exists()).toBe(true);
    expect(wrapper.find('.chrome-apps-page-stub').exists()).toBe(true);
    expect(wrapper.find('[data-testid="chrome-settings-page"]').exists()).toBe(false);
  });

  it('emits close (not navigate) when settings page requests close', async () => {
    const wrapper = mount(ChromeInternalView, {
      props: { url: 'chrome://settings' },
    });
    const settings = wrapper.findComponent({ name: 'ChromeSettingsPage' });
    await settings.vm.$emit('close');
    expect(wrapper.emitted('close')).toHaveLength(1);
    expect(wrapper.emitted('navigate')).toBeUndefined();
  });
});
