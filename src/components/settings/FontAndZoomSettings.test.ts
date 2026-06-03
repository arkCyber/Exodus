/**
 * Exodus Browser — FontAndZoomSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import FontAndZoomSettings from './FontAndZoomSettings.vue';

describe('FontAndZoomSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and zoom select', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="font-zoom-settings"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="default-zoom"]').exists()).toBe(true);
  });

  it('loads default settings', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const zoomSelect = wrapper.find('[data-testid="default-zoom"]');
    if (zoomSelect.exists()) {
      const element = zoomSelect.element as HTMLSelectElement;
      expect(element.value).toBe('100');
    }
  });

  it('emits status when zoom changes', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const zoomSelect = wrapper.find('[data-testid="default-zoom"]');
    await zoomSelect.setValue('125');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('changes font family', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const fontSelect = wrapper.find('[data-testid="standard-font"]');
    await fontSelect.setValue('Arial');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('changes font size', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const fontSizeInput = wrapper.find('[data-testid="font-size"]');
    await fontSizeInput.setValue('18');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('toggles smooth scrolling', async () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setValue(false);
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-font-zoom-settings', JSON.stringify({
      defaultZoom: 125,
      standardFont: 'Arial',
      serifFont: 'Georgia',
      sansSerifFont: 'Verdana',
      monospaceFont: 'Courier New',
      fontSize: 18,
      smoothScrolling: false
    }));
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.find('button');
    await resetButton.trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('displays Chinese strings when locale is zh', () => {
    const wrapper = mount(FontAndZoomSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('字体和缩放');
  });
});
