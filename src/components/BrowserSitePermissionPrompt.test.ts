/**
 * Exodus Browser — BrowserSitePermissionPrompt tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import BrowserSitePermissionPrompt from './BrowserSitePermissionPrompt.vue';

vi.mock('$lib/extensions/api', () => ({
  resolveBrowserSitePermission: vi.fn(async () => {}),
}));

describe('BrowserSitePermissionPrompt', () => {
  it('renders when request is set', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'r1',
          origin: 'https://example.com',
          kind: 'camera',
          webviewLabel: 'tab-1',
        },
      },
    });
    expect(wrapper.text()).toContain('https://example.com');
    expect(wrapper.text()).toContain('camera');
  });

  it('emits resolved on allow', async () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'r1',
          origin: 'https://example.com',
          kind: 'geolocation',
          webviewLabel: 'tab-1',
        },
      },
    });
    await wrapper.find('.btn.primary').trigger('click');
    expect(wrapper.emitted('resolved')).toHaveLength(1);
  });
});
