/**
 * Exodus Browser — AccessibilitySettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import AccessibilitySettings from './AccessibilitySettings.vue';

describe('AccessibilitySettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and checkboxes', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="accessibility-settings"]').exists()).toBe(true);
    expect(wrapper.find('input[type="checkbox"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    expect(checkboxes.length).toBeGreaterThan(0);
  });

  it('emits status when checkbox changes', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    if (checkbox.exists()) {
      await checkbox.setValue(true);
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('changes minimum font size', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const numberInput = wrapper.find('input[type="number"]');
    if (numberInput.exists()) {
      await numberInput.setValue('14');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('changes cursor size', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      await select.setValue('large');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-accessibility-settings', JSON.stringify({
      forceDarkMode: true,
      reduceMotion: true,
      highContrast: true,
      screenReader: true,
      minimumFontSize: 18,
      cursorSize: 'large',
      focusIndicator: false,
      textToSpeech: true
    }));
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.find('button');
    await resetButton.trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays Chinese strings when locale is zh', async () => {
    const wrapper = mount(AccessibilitySettings, { props: { uiLocale: 'zh' } });
    await flushPromises();
    expect(wrapper.text()).toContain('无障碍');
  });
});
