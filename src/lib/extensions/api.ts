/**
 * Exodus Browser — Web Extension host API (chrome.storage.local subset).
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
export {
  exodusAllamaAvailable,
  getExodusAllamaShim,
  type ExodusAllamaShim,
} from '$lib/extensions/exodusAllama';
import type { ExtensionInfo, StoreExtensionEntry } from '$lib/extensions/types';
import type { ExtensionTabSync } from '$lib/extensions/syncTabs';

/** List installed extensions. */
export async function listExtensions(): Promise<ExtensionInfo[]> {
  if (!isTauri()) {
    return [];
  }
  return invoke<ExtensionInfo[]>('extension_list');
}

/** Enable or disable an extension. */
export async function setExtensionEnabled(extensionId: string, enabled: boolean): Promise<void> {
  await invoke('extension_set_enabled', { extensionId, enabled });
}

/** Install unpacked extension from a folder path. */
export async function installExtensionFolder(folderPath: string): Promise<ExtensionInfo> {
  return invoke<ExtensionInfo>('extension_install_folder', { folderPath });
}

/** Uninstall extension by id. */
export async function uninstallExtension(extensionId: string): Promise<void> {
  await invoke('extension_uninstall', { extensionId });
}

/** Rescan dev + user extension directories. */
export async function rescanExtensions(): Promise<number> {
  return invoke<number>('extension_rescan');
}

/** Sync UI tabs for chrome.tabs.query in content scripts. */
export async function syncExtensionTabs(tabs: ExtensionTabSync[]): Promise<void> {
  await invoke('extension_sync_tabs', { tabs });
}

/** Query tab registry from the host (chrome.tabs.query subset). */
export async function queryExtensionTabs(
  query: Record<string, unknown> = {},
): Promise<ExtensionTabSync[]> {
  return invoke<ExtensionTabSync[]>('extension_tabs_query', { query });
}

/** Update tab properties (chrome.tabs.update). */
export async function updateExtensionTab(
  tabId: number,
  updateProperties: Record<string, unknown>,
): Promise<ExtensionTabSync> {
  return invoke<ExtensionTabSync>('extension_tabs_update', { tabId, updateProperties });
}

/** Remove tabs (chrome.tabs.remove). */
export async function removeExtensionTabs(tabIds: number[]): Promise<void> {
  await invoke('extension_tabs_remove', { tabIds });
}

/** Reload tab (chrome.tabs.reload). */
export async function reloadExtensionTab(tabId: number): Promise<void> {
  await invoke('extension_tabs_reload', { tabId });
}

/** chrome.storage.local.get */
export async function extensionStorageGet(
  extensionId: string,
  keys?: string[] | null,
): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('extension_storage_get', {
    extensionId,
    keys: keys ?? null,
  });
}

/** chrome.storage.local.set */
export async function extensionStorageSet(
  extensionId: string,
  items: Record<string, unknown>,
): Promise<void> {
  await invoke('extension_storage_set', { extensionId, items });
}

/** chrome.storage.local.remove */
export async function extensionStorageRemove(
  extensionId: string,
  keys: string[],
): Promise<void> {
  await invoke('extension_storage_remove', { extensionId, keys });
}

/** chrome.storage.local.clear */
export async function extensionStorageClear(extensionId: string): Promise<void> {
  await invoke('extension_storage_clear', { extensionId });
}

/** Install extension from `.crx` or `.zip` package path. */
export async function installExtensionCrx(packagePath: string): Promise<ExtensionInfo> {
  return invoke<ExtensionInfo>('extension_install_crx', { packagePath });
}

/** List bundled dev store extensions. */
export async function listStoreExtensions(): Promise<StoreExtensionEntry[]> {
  return invoke<StoreExtensionEntry[]>('extension_store_list');
}

/** Get extension popup URL (for action popup). */
export async function extensionPopupUrl(
  extensionId: string,
): Promise<string | null> {
  return invoke<string | null>('extension_popup_url', { extensionId });
}

/** Get extension manifest (chrome.runtime.getManifest). */
export async function extensionGetManifest(
  extensionId: string,
): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('extension_get_manifest', { extensionId });
}

/** Emit onInstalled event for an extension. */
export async function extensionEmitInstalledEvent(
  extensionId: string,
  reason: 'install' | 'update' | 'browser_update',
): Promise<void> {
  await invoke('extension_emit_installed_event', { extensionId, reason });
}

/** Check if extension has a permission (chrome.permissions.contains). */
export async function extensionPermissionsContains(
  extensionId: string,
  permission: string,
): Promise<boolean> {
  return invoke<boolean>('extension_permissions_contains', { extensionId, permission });
}

/** Get all granted permissions (chrome.permissions.getAll). */
export async function extensionPermissionsGetAll(
  extensionId: string,
): Promise<string[]> {
  return invoke<string[]>('extension_permissions_get_all', { extensionId });
}

/** Request additional permissions (chrome.permissions.request). */
export async function extensionPermissionsRequest(
  extensionId: string,
  permissions: string[],
): Promise<boolean> {
  return invoke<boolean>('extension_permissions_request', { extensionId, permissions });
}

/** Resolve a UI permission prompt (`extension_permissions_resolve`). */
export async function resolveExtensionPermission(
  requestId: string,
  granted: boolean,
): Promise<void> {
  await invoke('extension_permissions_resolve', { requestId, granted });
}

/** Fetch remote extension catalog when `extension_store_url` is configured. */
export async function fetchRemoteStoreExtensions(): Promise<StoreExtensionEntry[]> {
  return invoke<StoreExtensionEntry[]>('extension_store_fetch_remote');
}

/** Whether extension has host permission for URL. */
export async function validateExtensionHostAccess(
  extensionId: string,
  url: string,
): Promise<boolean> {
  return invoke<boolean>('extension_validate_host_access', { extensionId, url });
}

/** Granted host patterns for an extension. */
export async function listExtensionSitePermissions(extensionId: string): Promise<string[]> {
  return invoke<string[]>('extension_site_permissions_list', { extensionId });
}

/** Revoke granted host patterns for an extension. */
export async function revokeExtensionSitePermissions(
  extensionId: string,
  patterns: string[],
): Promise<void> {
  await invoke('extension_site_permissions_revoke', { extensionId, patterns });
}

/** Revoke all granted host patterns for an extension. */
export async function revokeAllExtensionSitePermissions(extensionId: string): Promise<void> {
  await invoke('extension_site_permissions_revoke_all', { extensionId });
}

/** Resolve install-time host_permissions prompt. */
export async function resolveExtensionHostInstall(
  requestId: string,
  granted: boolean,
): Promise<void> {
  await invoke('extension_host_install_resolve', { requestId, granted });
}

/** Toggle whether install auto-grants manifest host_permissions. */
export async function setConfirmHostPermissionsOnInstall(confirm: boolean): Promise<void> {
  await invoke('extension_set_confirm_host_permissions', { confirm });
}

/** Resolve a browser site permission prompt (camera / mic / geolocation). */
export async function resolveBrowserSitePermission(
  requestId: string,
  granted: boolean,
): Promise<void> {
  await invoke('browser_site_permission_resolve', { requestId, granted });
}

/** Stored per-origin browser permission (camera / mic / geolocation). */
export type BrowserSitePermissionEntry = {
  origin: string;
  kind: string;
  granted: boolean;
};

/** List saved browser site permission decisions. */
export async function listBrowserSitePermissions(): Promise<BrowserSitePermissionEntry[]> {
  return invoke<BrowserSitePermissionEntry[]>('browser_site_permissions_list');
}

/** Revoke browser site permission(s); omit kinds to clear the whole origin. */
export async function revokeBrowserSitePermission(
  origin: string,
  kinds?: string[],
): Promise<void> {
  await invoke('browser_site_permissions_revoke', {
    origin,
    kinds: kinds ?? null,
  });
}
