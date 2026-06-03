/**
 * Exodus Browser — AppearancePreferencesSettings tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import AppearancePreferencesSettings from './AppearancePreferencesSettings.vue';

describe('AppearancePreferencesSettings', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark', 'light-theme');
    document.documentElement.removeAttribute('data-theme');
  });

  it('renders theme and language selects', async () => {
    const wrapper = mount(AppearancePreferencesSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="appearance-theme-select"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="appearance-locale-select"]').exists()).toBe(true);
  });

  it('applies dark theme when selected', async () => {
    const wrapper = mount(AppearancePreferencesSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('[data-testid="appearance-theme-select"]');
    await select.setValue('dark');
    await flushPromises();
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('emits localeChange when language changes', async () => {
    const wrapper = mount(AppearancePreferencesSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    await wrapper.find('[data-testid="appearance-locale-select"]').setValue('zh');
    await flushPromises();
    expect(wrapper.emitted('localeChange')?.[0]).toEqual(['zh']);
    expect(localStorage.getItem('exodus-ui-locale')).toBe('zh');
  });
});
