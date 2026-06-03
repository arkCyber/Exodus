/**
 * Exodus Browser — ExtensionHostInstallPrompt component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ExtensionHostInstallPrompt from './ExtensionHostInstallPrompt.vue';

vi.mock('$lib/extensions/api', () => ({
  resolveExtensionHostInstall: vi.fn(),
}));

describe('ExtensionHostInstallPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when request is null', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: { request: null }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(false);
  });

  it('renders dialog when request is provided', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(true);
    expect(wrapper.find('.perm-dialog').exists()).toBe(true);
  });

  it('displays extension site access title', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Extension site access');
  });

  it('displays extension name', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    expect(wrapper.find('strong').text()).toBe('Test Extension');
  });

  it('displays host permissions list', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*', 'https://*.test.com/*']
        }
      }
    });
    
    const codes = wrapper.findAll('code');
    expect(codes.length).toBe(2);
    expect(codes[0].text()).toBe('https://*.example.com/*');
    expect(codes[1].text()).toBe('https://*.test.com/*');
  });

  it('displays hint text', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    expect(wrapper.find('.hint').text()).toBe('You can change this later in extension settings.');
  });

  it('calls resolveExtensionHostInstall with granted=true on allow', async () => {
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    resolveExtensionHostInstall.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveExtensionHostInstall).toHaveBeenCalledWith('req-1', true);
  });

  it('calls resolveExtensionHostInstall with granted=false on deny', async () => {
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    resolveExtensionHostInstall.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    await wrapper.find('.btn.secondary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveExtensionHostInstall).toHaveBeenCalledWith('req-1', false);
  });

  it('emits resolved after answering', async () => {
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    resolveExtensionHostInstall.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('disables buttons while busy', async () => {
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    resolveExtensionHostInstall.mockImplementation(() => new Promise(() => {})); // Never resolves
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
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
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    resolveExtensionHostInstall.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('does not answer when request is null', async () => {
    const { resolveExtensionHostInstall } = require('$lib/extensions/api');
    
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: { request: null }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    
    expect(resolveExtensionHostInstall).not.toHaveBeenCalled();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').attributes('role')).toBe('presentation');
    expect(wrapper.find('.perm-dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.perm-dialog').attributes('aria-labelledby')).toBe('host-install-title');
  });

  it('displays correct button text', () => {
    const wrapper = mount(ExtensionHostInstallPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          hostPermissions: ['https://*.example.com/*']
        }
      }
    });
    
    const buttons = wrapper.findAll('.btn');
    expect(buttons[0].text()).toBe('Deny sites');
    expect(buttons[1].text()).toBe('Allow sites');
  });
});
