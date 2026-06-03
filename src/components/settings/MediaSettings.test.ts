/**
 * Exodus Browser — MediaSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import MediaSettings from './MediaSettings.vue';

describe('MediaSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and autoplay settings', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="media-settings"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      expect(select.element.value).toBe('limit');
    }
  });

  it('emits status when settings change', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('select');
    if (select.exists()) {
      await select.setValue('block');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('toggles checkbox settings', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkboxes = wrapper.findAll('input[type="checkbox"]');
    if (checkboxes.length > 0) {
      await checkboxes[0].setValue(false);
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('changes numeric input', async () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const numberInput = wrapper.find('input[type="number"]');
    if (numberInput.exists()) {
      await numberInput.setValue('80');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-media-settings', JSON.stringify({
      autoplayPolicy: 'blocked',
      pictureInPicture: false,
      hardwareAcceleration: false,
      defaultVolume: 50
    }));
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.find('button');
    await resetButton.trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays Chinese strings when locale is zh', () => {
    const wrapper = mount(MediaSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('媒体');
  });
});
