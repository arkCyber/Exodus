/**
 * Exodus Browser — ExtensionsSettings component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import ExtensionsSettings from './ExtensionsSettings.vue';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

vi.mock('$lib/extensions/api', () => ({
  installExtensionFolder: vi.fn(),
  installExtensionCrx: vi.fn(),
  listExtensions: vi.fn(),
  listStoreExtensions: vi.fn(),
  fetchRemoteStoreExtensions: vi.fn(),
  rescanExtensions: vi.fn(),
  setExtensionEnabled: vi.fn(),
  uninstallExtension: vi.fn(),
  setConfirmHostPermissionsOnInstall: vi.fn(),
  listExtensionSitePermissions: vi.fn(),
  revokeExtensionSitePermissions: vi.fn(),
  revokeAllExtensionSitePermissions: vi.fn()
}));

const mockApi = {
  installExtensionFolder: vi.fn(),
  installExtensionCrx: vi.fn(),
  listExtensions: vi.fn(),
  listStoreExtensions: vi.fn(),
  fetchRemoteStoreExtensions: vi.fn(),
  rescanExtensions: vi.fn(),
  setExtensionEnabled: vi.fn(),
  uninstallExtension: vi.fn(),
  setConfirmHostPermissionsOnInstall: vi.fn(),
  listExtensionSitePermissions: vi.fn(),
  revokeExtensionSitePermissions: vi.fn(),
  revokeAllExtensionSitePermissions: vi.fn()
};

vi.mock('$lib/extensions/backgroundHosts', () => ({
  ensureExtensionBackgrounds: vi.fn()
}));

describe('ExtensionsSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    global.confirm = vi.fn(() => true);
    global.prompt = vi.fn(() => '/test/path');
  });

  const mockProps = {
    contentHost: document.body
  };

  it('renders settings section', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    expect(wrapper.find('.settings-section').exists()).toBe(true);
  });

  it('renders title', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    expect(wrapper.find('h3').text()).toBe('Web Extensions');
  });

  it('renders hint text', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    expect(wrapper.find('.settings-hint').text()).toContain('Manifest V3 extensions');
  });

  it('renders confirm host checkbox', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    const checkbox = wrapper.find('input[type="checkbox"]');
    expect(checkbox.exists()).toBe(true);
  });

  it('renders store URL input', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    const urlInput = wrapper.find('input[type="url"]');
    expect(urlInput.exists()).toBe(true);
    expect(urlInput.attributes('placeholder')).toBe('https://example.com/extensions/catalog.json');
  });

  it('renders save store URL button', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.nav-button');
    expect(buttons.some(b => b.text() === 'Save store URL')).toBe(true);
  });

  it('renders extension action buttons', () => {
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    const buttons = wrapper.findAll('.extensions-actions .nav-button');
    expect(buttons.length).toBe(4);
    expect(buttons[0].text()).toBe('Refresh');
    expect(buttons[1].text()).toBe('Rescan');
    expect(buttons[2].text()).toBe('Install folder…');
    expect(buttons[3].text()).toBe('Install .crx / .zip…');
  });

  it('disables refresh button when loading', async () => {
    mockApi.listExtensions.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await wrapper.find('.extensions-actions .nav-button').trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.extensions-actions .nav-button').attributes('disabled')).toBeDefined();
  });

  it('renders store items section when store items exist', async () => {
    const { listStoreExtensions } = require('$lib/extensions/api');
    listStoreExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test Extension', version: '1.0', path: '/path', installed: false }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.subsection-title').text()).toBe('Extension store (dev)');
  });

  it('does not render store items section when no store items', async () => {
    const { listStoreExtensions } = require('$lib/extensions/api');
    listStoreExtensions.mockResolvedValue([]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.subsection-title').length).toBe(1);
  });

  it('renders store item', async () => {
    const { listStoreExtensions } = require('$lib/extensions/api');
    listStoreExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test Extension', version: '1.0', path: '/path', installed: false }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const storeItems = wrapper.findAll('.extension-list.compact .extension-item');
    expect(storeItems.length).toBe(1);
    expect(storeItems[0].text()).toContain('Test Extension');
  });

  it('disables install button for installed store items', async () => {
    const { listStoreExtensions } = require('$lib/extensions/api');
    listStoreExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test Extension', version: '1.0', path: '/path', installed: true }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const installButton = wrapper.find('.extension-list.compact .nav-button');
    expect(installButton.attributes('disabled')).toBeDefined();
    expect(installButton.text()).toBe('Installed');
  });

  it('shows loading hint when loading extensions', async () => {
    mockApi.listExtensions
    listExtensions.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.settings-hint').some(h => h.text() === 'Loading extensions…')).toBe(true);
  });

  it('shows no extensions hint when no extensions', async () => {
    mockApi.listExtensions.mockResolvedValue([]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.findAll('.settings-hint').some(h => h.text() === 'No extensions installed.')).toBe(true);
  });

  it('renders installed extensions', async () => {
    mockApi.listExtensions.mockResolvedValue([
      {
        id: 'test-ext',
        name: 'Test Extension',
        version: '1.0',
        description: 'A test extension',
        permissions: ['storage', 'tabs'],
        enabled: true,
        actionPopup: 'popup.html'
      }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const extItems = wrapper.findAll('.extension-list:not(.compact) .extension-item');
    expect(extItems.length).toBe(1);
    expect(extItems[0].text()).toContain('Test Extension');
  });

  it('displays extension metadata', async () => {
    mockApi.listExtensions.mockResolvedValue([
      {
        id: 'test-ext',
        name: 'Test Extension',
        version: '1.0',
        description: 'A test extension',
        permissions: ['storage', 'tabs'],
        enabled: true,
        actionPopup: 'popup.html'
      }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const meta = wrapper.find('.extension-meta');
    expect(meta.text()).toContain('v1.0');
    expect(meta.text()).toContain('test-ext');
    expect(meta.text()).toContain('A test extension');
    expect(meta.text()).toContain('storage, tabs');
    expect(meta.text()).toContain('popup.html');
  });

  it('renders enabled checkbox for extensions', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const enabledCheckbox = wrapper.find('.extension-buttons input[type="checkbox"]');
    expect(enabledCheckbox.exists()).toBe(true);
    expect(enabledCheckbox.element.checked).toBe(true);
  });

  it('renders site access button', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.extension-buttons .nav-button');
    expect(buttons.some(b => b.text() === 'Site access')).toBe(true);
  });

  it('renders uninstall button', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.extension-buttons .nav-button');
    expect(buttons.some(b => b.text() === 'Uninstall')).toBe(true);
  });

  it('emits status on refresh', async () => {
    mockApi.listExtensions.mockResolvedValue([]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('calls refresh on mount', async () => {
    mockApi.listExtensions.mockResolvedValue([]);
    
    mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    
    expect(mockApi.listExtensions).toHaveBeenCalled();
  });

  it('loads extension settings on mount', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue({
      extension_store_url: 'https://example.com/catalog.json',
      confirm_host_permissions_on_install: true
    });
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    expect(invoke).toHaveBeenCalledWith('get_ai_config');
  });

  it('emits status on toggle confirm host', async () => {
    const { setConfirmHostPermissionsOnInstall } = require('$lib/extensions/api');
    setConfirmHostPermissionsOnInstall.mockResolvedValue();
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await wrapper.find('input[type="checkbox"]').trigger('change');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('emits status on save store URL', async () => {
    const { invoke } = require('@tauri-apps/api/core');
    invoke.mockResolvedValue();
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await wrapper.find('input[type="url"]').setValue('https://example.com/catalog.json');
    await wrapper.findAll('.nav-button')[0].trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(invoke).toHaveBeenCalledWith('extension_set_store_url', { url: 'https://example.com/catalog.json' });
    expect(wrapper.emitted('status')).toBeTruthy();
  });

  it('expands site access panel on button click', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-panel').exists()).toBe(true);
  });

  it('collapses site access panel on second click', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await wrapper.vm.$nextTick();
    await siteAccessButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-panel').exists()).toBe(false);
  });

  it('loads host patterns when site access is expanded', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue(['https://example.com']);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(mockApi.listExtensionSitePermissions).toHaveBeenCalledWith('test-ext');
  });

  it('displays host patterns', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue(['https://example.com', 'https://test.com']);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const patterns = wrapper.findAll('.host-pattern-row');
    expect(patterns.length).toBe(2);
  });

  it('shows no patterns message when no host patterns', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue([]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-panel').text()).toContain('No granted site patterns yet');
  });

  it('shows loading message when loading host patterns', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockImplementation(async () => {
      await new Promise(resolve => setTimeout(resolve, 100));
    });
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-panel').text()).toContain('Loading granted sites…');
  });

  it('renders revoke all button when patterns exist', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue(['https://example.com']);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-toolbar .nav-button').text()).toBe('Revoke all');
  });

  it('applies danger class to revoke all button', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue(['https://example.com']);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    expect(wrapper.find('.host-patterns-toolbar .nav-button').classes()).toContain('danger');
  });

  it('renders revoke button for each pattern', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    mockApi.listExtensionSitePermissions.mockResolvedValue(['https://example.com']);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const siteAccessButton = wrapper.findAll('.extension-buttons .nav-button')[1];
    await siteAccessButton.trigger('click');
    await new Promise(resolve => setTimeout(resolve, 50));
    await wrapper.vm.$nextTick();
    
    const revokeButtons = wrapper.findAll('.host-pattern-row .nav-button');
    expect(revokeButtons.length).toBe(1);
    expect(revokeButtons[0].text()).toBe('Revoke');
  });

  it('applies danger class to uninstall button', async () => {
    mockApi.listExtensions.mockResolvedValue([
      { id: 'test-ext', name: 'Test', version: '1.0', enabled: true, permissions: [] }
    ]);
    
    const wrapper = mount(ExtensionsSettings, {
      props: mockProps
    });
    
    await new Promise(resolve => setTimeout(resolve, 150));
    await wrapper.vm.$nextTick();
    
    const buttons = wrapper.findAll('.extension-buttons .nav-button');
    const uninstallButton = buttons.find(b => b.text() === 'Uninstall');
    expect(uninstallButton?.classes()).toContain('danger');
  });
});
