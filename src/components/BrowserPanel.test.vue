/**
 * Exodus Browser — BrowserPanel component tests.
 */
import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import BrowserPanel from './BrowserPanel.vue';

describe('BrowserPanel', () => {
  it('does not render when open is false', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: false, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-modal').exists()).toBe(false);
  });

  it('renders when open is true', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-modal').exists()).toBe(true);
  });

  it('displays title', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('h2').text()).toBe('Test Panel');
  });

  it('generates unique title ID', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-modal').attributes('aria-labelledby')).toBe('panel-title-test-panel');
  });

  it('generates title ID with spaces replaced by hyphens', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'My Test Panel' }
    });
    
    expect(wrapper.find('.panel-modal').attributes('aria-labelledby')).toBe('panel-title-my-test-panel');
  });

  it('renders slot content', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' },
      slots: {
        default: '<div class="test-content">Test Content</div>'
      }
    });
    
    expect(wrapper.find('.panel-content').exists()).toBe(true);
    expect(wrapper.find('.test-content').exists()).toBe(true);
    expect(wrapper.find('.test-content').text()).toBe('Test Content');
  });

  it('emits close when close button is clicked', async () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    await wrapper.find('.close-btn').trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('emits close when backdrop is clicked', async () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    await wrapper.find('.panel-modal').trigger('click');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('does not emit close when panel content is clicked', async () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' },
      slots: {
        default: '<div class="test-content">Test Content</div>'
      }
    });
    
    await wrapper.find('.panel-content').trigger('click');
    
    expect(wrapper.emitted('close')).toBeFalsy();
  });

  it('emits close when Escape key is pressed', async () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    await wrapper.find('.panel-modal').trigger('keydown.escape');
    
    expect(wrapper.emitted('close')).toBeTruthy();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-modal').attributes('role')).toBe('dialog');
    expect(wrapper.find('.panel-modal').attributes('aria-modal')).toBe('true');
    expect(wrapper.find('.panel-modal').attributes('tabindex')).toBe('-1');
  });

  it('has aria-label on close button', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.close-btn').attributes('aria-label')).toBe('Close');
  });

  it('renders close button with × symbol', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.close-btn').text()).toBe('×');
  });

  it('renders panel header', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-header').exists()).toBe(true);
  });

  it('renders panel content area', () => {
    const wrapper = mount(BrowserPanel, {
      props: { open: true, title: 'Test Panel' }
    });
    
    expect(wrapper.find('.panel-content').exists()).toBe(true);
  });
});
