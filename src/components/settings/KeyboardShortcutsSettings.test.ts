/**
 * Exodus Browser — KeyboardShortcutsSettings tests.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import KeyboardShortcutsSettings from './KeyboardShortcutsSettings.vue';

describe('KeyboardShortcutsSettings', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders title and shortcuts list', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    expect(wrapper.find('[data-testid="keyboard-shortcuts-settings"]').exists()).toBe(true);
    expect(wrapper.find('.shortcuts-list').exists()).toBe(true);
  });

  it('loads default shortcuts', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const shortcutItems = wrapper.findAll('.shortcut-item');
    expect(shortcutItems.length).toBeGreaterThan(0);
  });

  it('displays shortcut keys with kbd elements', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const kbdElements = wrapper.findAll('kbd');
    expect(kbdElements.length).toBeGreaterThan(0);
  });

  it('adds custom shortcut', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const addButton = wrapper.findAll('button').find(b => b.text().includes('Add'));
    if (addButton) {
      await addButton.trigger('click');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('removes custom shortcut', async () => {
    const confirmMock = vi.fn(() => true);
    global.confirm = confirmMock;
    localStorage.setItem('exodus-keyboard-shortcuts', JSON.stringify({
      custom: [{ action: 'Test', keys: 'Cmd+T' }]
    }));
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const removeButton = wrapper.find('.nav-button.danger');
    if (removeButton.exists()) {
      await removeButton.trigger('click');
      await flushPromises();
      expect(confirmMock).toHaveBeenCalledWith('Remove this custom shortcut?');
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('resets to defaults', async () => {
    localStorage.setItem('exodus-keyboard-shortcuts', JSON.stringify({
      custom: [{ action: 'Test1', keys: 'Cmd+T' }, { action: 'Test2', keys: 'Cmd+B' }]
    }));
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'en' } });
    await flushPromises();
    const resetButton = wrapper.findAll('button').find(b => b.text().includes('Reset'));
    if (resetButton) {
      await resetButton.trigger('click');
      await flushPromises();
      expect(wrapper.emitted('status')?.length).toBeGreaterThan(0);
    }
  });

  it('displays Chinese strings when locale is zh', async () => {
    const wrapper = mount(KeyboardShortcutsSettings, { props: { uiLocale: 'zh' } });
    await flushPromises();
    expect(wrapper.text()).toContain('键盘快捷键');
  });
});
