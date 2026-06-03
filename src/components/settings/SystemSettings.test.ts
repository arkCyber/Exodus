/**
 * Exodus Browser — SystemSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SystemSettings from './SystemSettings.vue';

describe('SystemSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and checkboxes', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="system-settings"]').exists()).toBe(true);
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes.length).toBeGreaterThan(0);
  });

  it('emits status when checkbox changes', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      await checkbox.setValue(true);
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('changes update channel', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      await select.setValue('beta');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('displays system information', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('.info-card').exists()).toBe(true);
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-system-settings', JSON.stringify({
      defaultBrowser: true,
      backgroundApps: false,
      hardwareAcceleration: false,
      useGPURendering: false,
      updateAutomatically: false,
      updateChannel: 'nightly'
    }));
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.find('button');
    await resetButton.trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays Chinese strings when locale is zh', async () => {
    const wrapper = mount(SystemSettings, { props: { uiLocale: 'zh' } });
    await flushPromises();
    expect(wrapper.text()).toContain('系统');
  });
});
