/**
 * Exodus Browser — ExtensionPermissionPrompt component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ExtensionPermissionPrompt from './ExtensionPermissionPrompt.vue';

vi.mock('$lib/extensions/extensionDisplayName', () => ({
  extensionDisplayName: vi.fn((req) => req?.extensionName || 'Extension'),
}));

vi.mock('$lib/extensions/api', () => ({
  resolveExtensionPermission: vi.fn(),
}));

describe('ExtensionPermissionPrompt', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not render when request is null', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: { request: null }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(false);
  });

  it('renders dialog when request is provided', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs', 'storage']
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').exists()).toBe(true);
    expect(wrapper.find('.perm-dialog').exists()).toBe(true);
  });

  it('displays permission request title', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    expect(wrapper.find('h3').text()).toBe('Permission request');
  });

  it('displays extension name', () => {
    const { extensionDisplayName } = require('$lib/extensions/extensionDisplayName');
    extensionDisplayName.mockReturnValue('Test Extension');
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    expect(wrapper.find('strong').text()).toBe('Test Extension');
  });

  it('categorizes API permissions correctly', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs', 'storage', 'bookmarks']
        }
      }
    });
    
    expect(wrapper.find('.perm-sub').text()).toBe('API permissions:');
    expect(wrapper.findAll('code').length).toBe(3);
  });

  it('categorizes host permissions correctly', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['https://*.example.com/*', '<all_urls>']
        }
      }
    });
    
    expect(wrapper.findAll('.perm-sub').length).toBe(1);
    expect(wrapper.findAll('code').length).toBe(2);
  });

  it('separates API and host permissions', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs', 'https://*.example.com/*', 'storage']
        }
      }
    });
    
    expect(wrapper.findAll('.perm-sub').length).toBe(2);
  });

  it('calls resolveExtensionPermission with granted=true on allow', async () => {
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    resolveExtensionPermission.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveExtensionPermission).toHaveBeenCalledWith('req-1', true);
  });

  it('calls resolveExtensionPermission with granted=false on deny', async () => {
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    resolveExtensionPermission.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    await wrapper.find('.btn.secondary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(resolveExtensionPermission).toHaveBeenCalledWith('req-1', false);
  });

  it('emits resolved after answering', async () => {
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    resolveExtensionPermission.mockResolvedValue(undefined);
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('disables buttons while busy', async () => {
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    resolveExtensionPermission.mockImplementation(() => new Promise(() => {})); // Never resolves
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
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
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    resolveExtensionPermission.mockRejectedValue(new Error('Failed'));
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('resolved')).toBeTruthy();
  });

  it('does not answer when request is null', async () => {
    const { resolveExtensionPermission } = require('$lib/extensions/api');
    
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: { request: null }
    });
    
    await wrapper.find('.btn.primary').trigger('click');
    
    expect(resolveExtensionPermission).not.toHaveBeenCalled();
  });

  it('has correct ARIA attributes', () => {
    const wrapper = mount(ExtensionPermissionPrompt, {
      props: {
        request: {
          extensionId: 'test-id',
          extensionName: 'Test Extension',
          requestId: 'req-1',
          permissions: ['tabs']
        }
      }
    });
    
    expect(wrapper.find('.perm-backdrop').attributes('role')).toBe('presentation');
    expect(wrapper.find('.perm-dialog').attributes('role')).toBe('dialog');
    expect(wrapper.find('.perm-dialog').attributes('aria-labelledby')).toBe('perm-title');
  });
});
