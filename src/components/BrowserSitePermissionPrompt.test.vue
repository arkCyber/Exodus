/**
 * Exodus Browser — BrowserSitePermissionPrompt component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import BrowserSitePermissionPrompt from './BrowserSitePermissionPrompt.vue';

vi.mock('$lib/extensions/api', () => ({
  resolveBrowserSitePermission: vi.fn(),
}));

describe('BrowserSitePermissionPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when request is null', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: { request: null }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(false);
  });

  it('renders dialog when request is provided', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(true);
    expect(wrapper.find('.perm-dialog').exists()).toBe(true);
  });

  it('displays site permission title', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Site permission');
  });

  it('displays origin in message', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    expect(wrapper.find('strong').text()).toBe('https://example.com');
  });

  it('displays camera permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('use your camera');
  });

  it('displays microphone permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'microphone'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('use your microphone');
  });

  it('displays mic permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'mic'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('use your microphone');
  });

  it('displays geolocation permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'geolocation'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('know your location');
  });

  it('displays location permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'location'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('know your location');
  });

  it('displays notifications permission label', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'notifications'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('show notifications');
  });

  it('displays default label for unknown permission', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'unknown-permission'
        }
      }
    });
    
    expect(wrapper.find('p').text()).toContain('use unknown-permission');
  });

  it('calls resolveBrowserSitePermission with granted=true on allow', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    resolveBrowserSitePermission.mockResolvedValue(undefined);
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveBrowserSitePermission).toHaveBeenCalledWith('req-1', true);
  });

  it('calls resolveBrowserSitePermission with granted=false on block', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    resolveBrowserSitePermission.mockResolvedValue(undefined);
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    await wrapper.find('.btn.secondary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveBrowserSitePermission).toHaveBeenCalledWith('req-1', false);
  });

  it('emits resolved after answering', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    resolveBrowserSitePermission.mockResolvedValue(undefined);
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('disables buttons while busy', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    resolveBrowserSitePermission.mockImplementation(() => new Promise(() => {})); // Never resolves
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.btn');
    buttons.forEach(button => {
      expect(button.attributes('disabled')).toBeDefined();
    });
  });

  it('handles resolve errors gracefully', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    resolveBrowserSitePermission.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('does not answer when request is null', async () => {
    const { resolveBrowserSitePermission } = require('$lib/extensions/api');
    
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: { request: null }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    
    expect(resolveBrowserSitePermission).not.toHaveBeenCalled();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').attributes('role')).toBe('presentation');
    expect(wrapper.find('.perm-dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.perm-dialog').attributes('aria-labelledby')).toBe('site-perm-title');
  });

  it('displays correct button text', () => {
    const wrapper = mount(BrowserSitePermissionPrompt, {
      props: {
        request: {
          requestId: 'req-1',
          origin: 'https://example.com',
          kind: 'camera'
        }
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    expect(buttons[0].text()).toBe('Block');
    expect(buttons[1].text()).toBe('Allow');
  });
});
