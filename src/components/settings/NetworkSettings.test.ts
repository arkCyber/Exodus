/**
 * Exodus Browser — NetworkSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import NetworkSettings from './NetworkSettings.vue';

describe('NetworkSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and proxy settings', async () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="network-settings"]').exists()).toBe(true);
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      const element = checkbox.element as HTMLInputElement;
      expect(element.checked).toBe(false);
    }
  });

  it('emits status when settings change', async () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      await select.setValue('manual');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('toggles checkbox settings', async () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      await checkbox.setValue(true);
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-network-settings', JSON.stringify({
      proxyMode: 'manual',
      proxyHost: '127.0.0.1',
      proxyPort: 8080,
      dnsOverHttps: true,
      dnsProvider: 'cloudflare',
      httpProtocol: 'https'
    }));
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.find('button');
    await resetButton.trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays Chinese strings when locale is zh', () => {
    const wrapper = mount(NetworkSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('网络');
  });
});
