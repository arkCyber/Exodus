/**
 * Exodus Browser — ContextMenu component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ContextMenu from './ContextMenu.vue';

describe('ContextMenu', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when visible is false', () => {
    const wrapper = mount(ContextMenu, {
      props: { visible: false }
    });
    
    expect(wrapper.find('.context-menu').exists()).toBe(false);
  });

  it('renders when visible is true', () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 100,
        y: 200
      }
    });
    
    expect(wrapper.find('.context-menu').exists()).toBe(true);
  });

  it('positions menu at correct coordinates', () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        x: 150,
        y: 250
      }
    });
    
    const menu = wrapper.find('.context-menu');
    expect(menu.attributes('style')).toContain('left: 150px');
    expect(menu.attributes('style')).toContain('top: 250px');
  });

  it('renders menu items', () => {
    const items = [
      { id: '1', label: 'Copy' },
      { id: '2', label: 'Paste' },
      { id: '3', label: 'Delete' }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    expect(wrapper.findAll('.context-menu-item').length).toBe(3);
  });

  it('renders item icon when provided', () => {
    const items = [
      { id: '1', label: 'Copy', icon: '📋' }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    expect(wrapper.find('.menu-icon').text()).toBe('📋');
  });

  it('renders item shortcut when provided', () => {
    const items = [
      { id: '1', label: 'Copy', shortcut: '⌘C' }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    expect(wrapper.find('.menu-shortcut').text()).toBe('⌘C');
  });

  it('applies disabled class to disabled items', () => {
    const items = [
      { id: '1', label: 'Copy', disabled: true }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    expect(wrapper.find('.context-menu-item').classes()).toContain('disabled');
  });

  it('applies separator class to separator items', () => {
    const items = [
      { id: '1', label: 'Copy' },
      { id: '2', label: '', separator: true }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    const separatorItem = wrapper.findAll('.context-menu-item')[1];
    expect(separatorItem.classes()).toContain('separator');
  });

  it('calls item action when clicked', async () => {
    const actionSpy = vi.fn();
    const items = [
      { id: '1', label: 'Copy', action: actionSpy }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    await wrapper.find('.context-menu-item').trigger('click');
    
    expect(actionSpy).toHaveBeenCalled();
  });

  it('emits close when item is clicked', async () => {
    const items = [
      { id: '1', label: 'Copy', action: vi.fn() }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    await wrapper.find('.context-menu-item').trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('does not call action for disabled items', async () => {
    const actionSpy = vi.fn();
    const items = [
      { id: '1', label: 'Copy', disabled: true, action: actionSpy }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    await wrapper.find('.context-menu-item').trigger('click');
    
    expect(actionSpy).not.toHaveBeenCalled();
  });

  it('does not call action for separator items', async () => {
    const actionSpy = vi.fn();
    const items = [
      { id: '1', label: '', separator: true, action: actionSpy }
    ];
    
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items
      }
    });
    
    await wrapper.find('.context-menu-item').trigger('click');
    
    expect(actionSpy).not.toHaveBeenCalled();
  });

  it('emits close when clicking outside', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items: [{ id: '1', label: 'Copy' }]
      }
    });
    
    document.dispatchEvent(new MouseEvent('click'));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('emits close when Escape key is pressed', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items: [{ id: '1', label: 'Copy' }]
      }
    });
    
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('does not emit close for other keys', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items: [{ id: '1', label: 'Copy' }]
      }
    });
    
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('close')).toBeFalsy();
  });

  it('stops click propagation on menu', async () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items: [{ id: '1', label: 'Copy' }]
      }
    });
    
    const menu = wrapper.find('.context-menu');
    await menu.trigger('click');
    
    // Should not emit close because click.stop is used
    expect(wrapper.emitted('close')).toBeFalsy();
  });

  it('renders empty menu when no items provided', () => {
    const wrapper = mount(ContextMenu, {
      props: {
        visible: true,
        items: []
      }
    });
    
    expect(wrapper.findAll('.context-menu-item').length).toBe(0);
  });
});
