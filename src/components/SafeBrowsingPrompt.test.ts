/**
 * Exodus Browser — SafeBrowsingPrompt component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SafeBrowsingPrompt from './SafeBrowsingPrompt.vue';

describe('SafeBrowsingPrompt', () => {
  it('renders warning when offer is set', () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: {
          url: 'https://phish.test',
          reason: 'Safe Browsing blocked this page',
        },
      },
    });
    expect(wrapper.text()).toContain('Security warning');
    expect(wrapper.text()).toContain('https://phish.test');
  });

  it('emits proceed and cancel', async () => {
    const wrapper = mount(SafeBrowsingPrompt, {
      props: {
        offer: { url: 'https://x.test', reason: 'warn' },
      },
    });
    await wrapper.find('.btn.danger').trigger('click');
    expect(wrapper.emitted('proceed')).toHaveLength(1);
    await wrapper.find('.btn.secondary').trigger('click');
    expect(wrapper.emitted('cancel')).toHaveLength(1);
  });
});
