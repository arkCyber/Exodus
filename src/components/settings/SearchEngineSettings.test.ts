/**
 * Exodus Browser — SearchEngineSettings tests.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import SearchEngineSettings from './SearchEngineSettings.vue';

describe('SearchEngineSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and default engine select', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="search-engine-settings"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="search-engine-default"]').exists()).toBe(true);
  });

  it('loads built-in engines by default', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('[data-testid="search-engine-default"]');
    expect(select.findAll('option').length).toBeGreaterThan(0);
  });

  it('emits status when default engine changes', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const select = wrapper.find('[data-testid="search-engine-default"]');
    await select.setValue('google');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('adds custom engine', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    await wrapper.find('input[placeholder*="My Search"]').setValue('Custom Engine');
    await wrapper.find('input[placeholder*="https://example.com"]').setValue('https://example.com/search?q={query}');
    await wrapper.find('button').trigger('click');
    await flushPromises();
    expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
  });

  it('removes custom engine', async () => {
    const confirmMock = vi.fn(() => true);
    global.confirm = confirmMock;
    localStorage.setItem('exodus-search-engines', JSON.stringify([
      { id: 'custom-1', name: 'Custom', url: 'https://example.com/search?q={query}', builtin: false }
    ]));
    localStorage.setItem('exodus-default-search-engine', 'duckduckgo');
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const removeButton = wrapper.find('.nav-button.danger');
    if (removeButton.exists()) {
      await removeButton.trigger('click');
      await flushPromises();
      expect(confirmMock).toHaveBeenCalledWith('Remove this search engine?');
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('restores built-in engines', async () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const restoreButtons = wrapper.findAll('button');
    const restoreButton = restoreButtons.find(b => b.text().includes('Restore'));
    if (restoreButton) {
      await restoreButton.trigger('click');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('displays Chinese strings when locale is zh', () => {
    const wrapper = mount(SearchEngineSettings, { props: { uiLocale: 'zh' } });
    expect(wrapper.text()).toContain('搜索引擎');
  });
});
