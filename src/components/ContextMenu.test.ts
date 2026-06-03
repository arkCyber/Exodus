/**
 * Exodus Browser — ContextMenu component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import ContextMenu from './ContextMenu.vue';

function mountMenu(props: Record<string, unknown>) {
  return mount(ContextMenu, {
    props,
    attachTo: document.body,
  });
}

describe('ContextMenu', () => {
  it('renders when visible is true', () => {
    const wrapper = mountMenu({
      visible: true,
      x: 100,
      y: 100,
      items: [
        { id: 'back', label: 'Back', icon: '←', action: () => {} },
        { id: 'forward', label: 'Forward', icon: '→', action: () => {} },
      ],
    });

    expect(document.querySelector('.context-menu')).toBeTruthy();
    wrapper.unmount();
  });

  it('does not render when visible is false', () => {
    const wrapper = mountMenu({
      visible: false,
      x: 100,
      y: 100,
      items: [{ id: 'back', label: 'Back', icon: '←', action: () => {} }],
    });

    expect(document.querySelector('.context-menu')).toBeFalsy();
    wrapper.unmount();
  });

  it('renders menu items', () => {
    const wrapper = mountMenu({
      visible: true,
      x: 100,
      y: 100,
      items: [
        { id: 'back', label: 'Back', icon: '←', action: () => {} },
        { id: 'forward', label: 'Forward', icon: '→', action: () => {} },
      ],
    });

    expect(document.querySelectorAll('.context-menu-item').length).toBe(2);
    wrapper.unmount();
  });

  it('renders separator items', () => {
    const wrapper = mountMenu({
      visible: true,
      x: 100,
      y: 100,
      items: [
        { id: 'back', label: 'Back', icon: '←', action: () => {} },
        { id: 'separator1', label: '', separator: true },
        { id: 'forward', label: 'Forward', icon: '→', action: () => {} },
      ],
    });

    expect(document.querySelector('.context-menu-item.separator')).toBeTruthy();
    wrapper.unmount();
  });

  it('disables menu items when disabled is true', () => {
    const wrapper = mountMenu({
      visible: true,
      x: 100,
      y: 100,
      items: [{ id: 'back', label: 'Back', icon: '←', action: () => {}, disabled: true }],
    });

    expect(document.querySelector('.context-menu-item.disabled')).toBeTruthy();
    wrapper.unmount();
  });

  it('emits close when menu item is clicked', async () => {
    const wrapper = mountMenu({
      visible: true,
      x: 100,
      y: 100,
      items: [{ id: 'back', label: 'Back', icon: '←', action: () => {} }],
    });

    const item = document.querySelector('.context-menu-item') as HTMLElement;
    item?.click();
    await wrapper.vm.$nextTick();
    expect(wrapper.emitted('close')).toBeTruthy();
    wrapper.unmount();
  });

  it('positions menu at correct coordinates', () => {
    const wrapper = mountMenu({
      visible: true,
      x: 200,
      y: 300,
      items: [{ id: 'back', label: 'Back', icon: '←', action: () => {} }],
    });

    const menu = document.querySelector('.context-menu') as HTMLElement;
    expect(menu?.getAttribute('style')).toContain('left: 200px');
    expect(menu?.getAttribute('style')).toContain('top: 300px');
    wrapper.unmount();
  });
});
