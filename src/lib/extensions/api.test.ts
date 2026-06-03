/**
 * Exodus Browser — extensions API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { DEV_TOOLBAR_EXTENSIONS } from '@/lib/devExtensionPreview';
import {
  listExtensions,
  setExtensionEnabled,
  installExtensionFolder,
  uninstallExtension,
  rescanExtensions,
  syncExtensionTabs,
  queryExtensionTabs,
  updateExtensionTab,
  removeExtensionTabs,
  reloadExtensionTab,
  extensionStorageGet,
  extensionStorageSet,
  extensionStorageRemove,
  extensionStorageClear,
  installExtensionCrx,
  listStoreExtensions,
  extensionPopupUrl,
  extensionGetManifest,
  extensionEmitInstalledEvent,
  extensionPermissionsContains,
  extensionPermissionsGetAll,
  extensionPermissionsRequest,
  resolveExtensionPermission,
  fetchRemoteStoreExtensions,
  validateExtensionHostAccess,
  listExtensionSitePermissions,
  revokeExtensionSitePermissions,
  revokeAllExtensionSitePermissions,
  resolveExtensionHostInstall,
  setConfirmHostPermissionsOnInstall,
  resolveBrowserSitePermission,
  listBrowserSitePermissions,
  revokeBrowserSitePermission,
} from './api';

let tauriRuntime = true;

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => tauriRuntime,
}));

describe('extensions api', () => {
  beforeEach(() => {
    tauriRuntime = true;
    vi.clearAllMocks();
  });

  it('lists extensions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockExtensions = [{ id: 'ext-1', name: 'Test', version: '1.0.0' }];
    vi.mocked(invoke).mockResolvedValue(mockExtensions);

    const extensions = await listExtensions();

    expect(extensions).toEqual(mockExtensions);
    expect(invoke).toHaveBeenCalledWith('extension_list');
  });

  it('returns DEV_TOOLBAR_EXTENSIONS when not in Tauri', async () => {
    tauriRuntime = false;

    const extensions = await listExtensions();

    expect(extensions).toEqual(DEV_TOOLBAR_EXTENSIONS);
  });

  it('sets extension enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setExtensionEnabled('ext-1', true);

    expect(invoke).toHaveBeenCalledWith('extension_set_enabled', { extensionId: 'ext-1', enabled: true });
  });

  it('installs extension folder', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockExtension = { id: 'ext-1', name: 'Test', version: '1.0.0' };
    vi.mocked(invoke).mockResolvedValue(mockExtension);

    const extension = await installExtensionFolder('/path/to/extension');

    expect(extension).toEqual(mockExtension);
    expect(invoke).toHaveBeenCalledWith('extension_install_folder', { folderPath: '/path/to/extension' });
  });

  it('throws when installing folder not in Tauri', async () => {
    tauriRuntime = false;

    await expect(installExtensionFolder('/path')).rejects.toThrow('Not running in Tauri');
  });

  it('uninstalls extension', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await uninstallExtension('ext-1');

    expect(invoke).toHaveBeenCalledWith('extension_uninstall', { extensionId: 'ext-1' });
  });

  it('rescans extensions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(5);

    const count = await rescanExtensions();

    expect(count).toBe(5);
    expect(invoke).toHaveBeenCalledWith('extension_rescan');
  });

  it('syncs extension tabs', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const tabs = [{ tabId: 1, url: 'https://example.com' }] as any;
    await syncExtensionTabs(tabs);

    expect(invoke).toHaveBeenCalledWith('extension_sync_tabs', { tabs });
  });

  it('queries extension tabs', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockTabs = [{ tabId: 1, url: 'https://example.com' }];
    vi.mocked(invoke).mockResolvedValue(mockTabs);

    const tabs = await queryExtensionTabs({ active: true });

    expect(tabs).toEqual(mockTabs);
    expect(invoke).toHaveBeenCalledWith('extension_tabs_query', { query: { active: true } });
  });

  it('updates extension tab', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockTab = { tabId: 1, url: 'https://example.com' };
    vi.mocked(invoke).mockResolvedValue(mockTab);

    const tab = await updateExtensionTab(1, { url: 'https://new.com' });

    expect(tab).toEqual(mockTab);
    expect(invoke).toHaveBeenCalledWith('extension_tabs_update', { tabId: 1, updateProperties: { url: 'https://new.com' } });
  });

  it('removes extension tabs', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await removeExtensionTabs([1, 2]);

    expect(invoke).toHaveBeenCalledWith('extension_tabs_remove', { tabIds: [1, 2] });
  });

  it('reloads extension tab', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await reloadExtensionTab(1);

    expect(invoke).toHaveBeenCalledWith('extension_tabs_reload', { tabId: 1 });
  });

  it('gets extension storage', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockData = { key: 'value' };
    vi.mocked(invoke).mockResolvedValue(mockData);

    const data = await extensionStorageGet('ext-1', ['key']);

    expect(data).toEqual(mockData);
    expect(invoke).toHaveBeenCalledWith('extension_storage_get', { extensionId: 'ext-1', keys: ['key'] });
  });

  it('sets extension storage', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await extensionStorageSet('ext-1', { key: 'value' });

    expect(invoke).toHaveBeenCalledWith('extension_storage_set', { extensionId: 'ext-1', items: { key: 'value' } });
  });

  it('removes extension storage', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await extensionStorageRemove('ext-1', ['key']);

    expect(invoke).toHaveBeenCalledWith('extension_storage_remove', {
      extensionId: 'ext-1',
      keys: ['key'],
    });
  });

  it('clears extension storage', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await extensionStorageClear('ext-1');

    expect(invoke).toHaveBeenCalledWith('extension_storage_clear', { extensionId: 'ext-1' });
  });

  it('installs extension crx', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockExtension = { id: 'ext-1', name: 'Test', version: '1.0.0' };
    vi.mocked(invoke).mockResolvedValue(mockExtension);

    const extension = await installExtensionCrx('/path/to/extension.crx');

    expect(extension).toEqual(mockExtension);
    expect(invoke).toHaveBeenCalledWith('extension_install_crx', { packagePath: '/path/to/extension.crx' });
  });

  it('lists store extensions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockExtensions = [{ id: 'store-1', name: 'Store Extension' }];
    vi.mocked(invoke).mockResolvedValue(mockExtensions);

    const extensions = await listStoreExtensions();

    expect(extensions).toEqual(mockExtensions);
    expect(invoke).toHaveBeenCalledWith('extension_store_list');
  });

  it('gets extension popup URL', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('popup.html');

    const url = await extensionPopupUrl('ext-1');

    expect(url).toBe('popup.html');
    expect(invoke).toHaveBeenCalledWith('extension_popup_url', { extensionId: 'ext-1' });
  });

  it('gets extension manifest', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockManifest = { name: 'Test', version: '1.0.0' };
    vi.mocked(invoke).mockResolvedValue(mockManifest);

    const manifest = await extensionGetManifest('ext-1');

    expect(manifest).toEqual(mockManifest);
    expect(invoke).toHaveBeenCalledWith('extension_get_manifest', { extensionId: 'ext-1' });
  });

  it('emits installed event', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await extensionEmitInstalledEvent('ext-1', 'install');

    expect(invoke).toHaveBeenCalledWith('extension_emit_installed_event', { extensionId: 'ext-1', reason: 'install' });
  });

  it('checks if extension has permission', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const hasPermission = await extensionPermissionsContains('ext-1', 'tabs');

    expect(hasPermission).toBe(true);
    expect(invoke).toHaveBeenCalledWith('extension_permissions_contains', { extensionId: 'ext-1', permission: 'tabs' });
  });

  it('gets all extension permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(['tabs', 'storage']);

    const permissions = await extensionPermissionsGetAll('ext-1');

    expect(permissions).toEqual(['tabs', 'storage']);
    expect(invoke).toHaveBeenCalledWith('extension_permissions_get_all', { extensionId: 'ext-1' });
  });

  it('requests extension permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const granted = await extensionPermissionsRequest('ext-1', ['tabs']);

    expect(granted).toBe(true);
    expect(invoke).toHaveBeenCalledWith('extension_permissions_request', { extensionId: 'ext-1', permissions: ['tabs'] });
  });

  it('resolves extension permission', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resolveExtensionPermission('req-1', true);

    expect(invoke).toHaveBeenCalledWith('extension_permissions_resolve', { requestId: 'req-1', granted: true });
  });

  it('fetches remote store extensions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockExtensions = [{ id: 'remote-1', name: 'Remote Extension' }];
    vi.mocked(invoke).mockResolvedValue(mockExtensions);

    const extensions = await fetchRemoteStoreExtensions();

    expect(extensions).toEqual(mockExtensions);
    expect(invoke).toHaveBeenCalledWith('extension_store_fetch_remote');
  });

  it('validates extension host access', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const hasAccess = await validateExtensionHostAccess('ext-1', 'https://example.com');

    expect(hasAccess).toBe(true);
    expect(invoke).toHaveBeenCalledWith('extension_validate_host_access', { extensionId: 'ext-1', url: 'https://example.com' });
  });

  it('lists extension site permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(['https://*/*']);

    const permissions = await listExtensionSitePermissions('ext-1');

    expect(permissions).toEqual(['https://*/*']);
    expect(invoke).toHaveBeenCalledWith('extension_site_permissions_list', { extensionId: 'ext-1' });
  });

  it('revokes extension site permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await revokeExtensionSitePermissions('ext-1', ['https://example.com/*']);

    expect(invoke).toHaveBeenCalledWith('extension_site_permissions_revoke', { extensionId: 'ext-1', patterns: ['https://example.com/*'] });
  });

  it('revokes all extension site permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await revokeAllExtensionSitePermissions('ext-1');

    expect(invoke).toHaveBeenCalledWith('extension_site_permissions_revoke_all', {
      extensionId: 'ext-1',
    });
  });

  it('resolves extension host install', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resolveExtensionHostInstall('req-1', true);

    expect(invoke).toHaveBeenCalledWith('extension_host_install_resolve', { requestId: 'req-1', granted: true });
  });

  it('sets confirm host permissions on install', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setConfirmHostPermissionsOnInstall(true);

    expect(invoke).toHaveBeenCalledWith('extension_set_confirm_host_permissions', { confirm: true });
  });

  it('resolves browser site permission', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await resolveBrowserSitePermission('req-1', true);

    expect(invoke).toHaveBeenCalledWith('browser_site_permission_resolve', { requestId: 'req-1', granted: true });
  });

  it('lists browser site permissions', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockPermissions = [{ origin: 'https://example.com', kind: 'camera', granted: true }];
    vi.mocked(invoke).mockResolvedValue(mockPermissions);

    const permissions = await listBrowserSitePermissions();

    expect(permissions).toEqual(mockPermissions);
    expect(invoke).toHaveBeenCalledWith('browser_site_permissions_list');
  });

  it('revokes browser site permission', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await revokeBrowserSitePermission('https://example.com', ['camera']);

    expect(invoke).toHaveBeenCalledWith('browser_site_permissions_revoke', { origin: 'https://example.com', kinds: ['camera'] });
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('API error'));

    await expect(listExtensions()).rejects.toThrow('API error');
  });
});
