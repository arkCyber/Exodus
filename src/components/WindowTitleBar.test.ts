/**
 * Exodus Browser — WindowTitleBar component tests.
 */
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import WindowTitleBar from './WindowTitleBar.vue';

vi.mock('$lib/windowDrag', () => ({
  startWindowDragFromMouseDown: vi.fn(),
}));

describe('WindowTitleBar', () => {
  it('renders title bar with drag region id', () => {
    const wrapper = mount(WindowTitleBar, {
      slots: { default: '<div class="child-toolbar">Toolbar</div>' },
    });
    const bar = wrapper.find('#exodus-window-titlebar');
    expect(bar.exists()).toBe(true);
    expect(bar.attributes('data-tauri-drag-region')).toBeDefined();
    expect(bar.classes()).toContain('exodus-window-titlebar');
  });
});
