/**
 * Exodus Browser — host integration for chrome.contextMenus.
 * Aerospace-level error handling, security validation, and input validation.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_MENU_ITEM_ID_PATTERN = /^[a-zA-Z0-9_\-:]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+/;
const VALID_HOST_CONTEXT_PATTERN = /^[a-zA-Z0-9_-]+$/;

/**
 * Aerospace-level validation for extension ID format.
 */
function validateExtensionId(extensionId: string): boolean {
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ContextMenus] Invalid extension ID');
    return false;
  }
  return VALID_EXTENSION_ID_PATTERN.test(extensionId);
}

/**
 * Aerospace-level validation for menu item ID format.
 */
function validateMenuItemId(menuItemId: string): boolean {
  if (!menuItemId || typeof menuItemId !== 'string') {
    console.error('[ContextMenus] Invalid menu item ID');
    return false;
  }
  return VALID_MENU_ITEM_ID_PATTERN.test(menuItemId);
}

/**
 * Aerospace-level validation for URL format.
 */
function validateUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    console.error('[ContextMenus] Invalid URL');
    return false;
  }
  return VALID_URL_PATTERN.test(url);
}

/**
 * Aerospace-level validation for host context format.
 */
function validateHostContext(hostContext: string): boolean {
  if (!hostContext || typeof hostContext !== 'string') {
    console.error('[ContextMenus] Invalid host context');
    return false;
  }
  return VALID_HOST_CONTEXT_PATTERN.test(hostContext);
}

/** Context menu item from extension registry. */
export type ExtensionContextMenuItem = {
  id: string;
  title: string;
  contexts: string[];
  parentId?: string | null;
  enabled: boolean;
  visible: boolean;
  itemType: string;
  checked?: boolean;
};

/** Row shown in the host context menu. */
export type HostContextMenuEntry = {
  extensionId: string;
  extensionName: string;
  item: ExtensionContextMenuItem;
};

/** List extension context menu items for a page URL and host context (e.g. `page`). */
export async function listExtensionContextMenus(
  pageUrl: string,
  hostContext = 'page',
): Promise<HostContextMenuEntry[]> {
  if (!isTauri()) return [];
  
  // Aerospace-level input validation
  if (!validateUrl(pageUrl)) {
    console.error('[ContextMenus] Invalid pageUrl for listExtensionContextMenus:', pageUrl);
    return [];
  }
  
  if (!validateHostContext(hostContext)) {
    console.error('[ContextMenus] Invalid hostContext for listExtensionContextMenus:', hostContext);
    return [];
  }
  
  try {
    return await invoke<HostContextMenuEntry[]>('extension_context_menus_list_host', {
      pageUrl,
      hostContext,
    });
  } catch (error) {
    console.error('extension_context_menus_list_host failed:', error);
    return [];
  }
}

/** Notify extension background that a context menu item was clicked. */
export async function fireExtensionContextMenuClick(
  extensionId: string,
  menuItemId: string,
  pageUrl: string,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ContextMenus] Invalid extensionId for fireExtensionContextMenuClick:', extensionId);
    return;
  }
  
  if (!validateMenuItemId(menuItemId)) {
    console.error('[ContextMenus] Invalid menuItemId for fireExtensionContextMenuClick:', menuItemId);
    return;
  }
  
  if (!validateUrl(pageUrl)) {
    console.error('[ContextMenus] Invalid pageUrl for fireExtensionContextMenuClick:', pageUrl);
    return;
  }
  
  try {
    await invoke('extension_context_menu_clicked', {
      extensionId,
      menuItemId,
      pageUrl,
    });
  } catch (error) {
    console.error('extension_context_menu_clicked failed:', error);
  }
}

/** Context menu creation options. */
export type CreateContextMenuOptions = {
  id: string;
  title?: string;
  contexts?: string[];
  parentId?: string;
  enabled?: boolean;
  visible?: boolean;
  checked?: boolean;
  type?: 'normal' | 'checkbox' | 'radio' | 'separator';
  icons?: { '16': string; '32': string; '48': string; '128': string };
};

/** chrome.contextMenus.create - Create a context menu item. */
export async function createContextMenu(
  extensionId: string,
  options: CreateContextMenuOptions,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ContextMenus] Invalid extensionId for createContextMenu');
    return;
  }
  
  if (!options || typeof options !== 'object') {
    console.error('[ContextMenus] Invalid options for createContextMenu');
    return;
  }
  
  if (!options.id || !validateMenuItemId(options.id)) {
    console.error('[ContextMenus] Invalid id in options for createContextMenu');
    return;
  }
  
  // Validate parentId if provided
  if (options.parentId && !validateMenuItemId(options.parentId)) {
    console.error('[ContextMenus] Invalid parentId in options for createContextMenu');
    return;
  }
  
  // Validate contexts if provided
  if (options.contexts && !Array.isArray(options.contexts)) {
    console.error('[ContextMenus] Invalid contexts in options for createContextMenu');
    return;
  }
  
  try {
    await invoke('extension_context_menus_create', { extensionId, options });
  } catch (error) {
    console.error('[ContextMenus] extension_context_menus_create failed:', error);
    throw error;
  }
}

/** Context menu update options. */
export type UpdateContextMenuOptions = {
  title?: string;
  contexts?: string[];
  enabled?: boolean;
  visible?: boolean;
  checked?: boolean;
  icons?: { '16': string; '32': string; '48': string; '128': string };
};

/** chrome.contextMenus.update - Update a context menu item. */
export async function updateContextMenu(
  extensionId: string,
  menuItemId: string,
  options: UpdateContextMenuOptions,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ContextMenus] Invalid extensionId for updateContextMenu');
    return;
  }
  
  if (!validateMenuItemId(menuItemId)) {
    console.error('[ContextMenus] Invalid menuItemId for updateContextMenu');
    return;
  }
  
  if (!options || typeof options !== 'object') {
    console.error('[ContextMenus] Invalid options for updateContextMenu');
    return;
  }
  
  // Validate contexts if provided
  if (options.contexts && !Array.isArray(options.contexts)) {
    console.error('[ContextMenus] Invalid contexts in options for updateContextMenu');
    return;
  }
  
  try {
    await invoke('extension_context_menus_update', { extensionId, menuItemId, options });
  } catch (error) {
    console.error('[ContextMenus] extension_context_menus_update failed:', error);
    throw error;
  }
}

/** chrome.contextMenus.remove - Remove a context menu item. */
export async function removeContextMenu(
  extensionId: string,
  menuItemId: string,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ContextMenus] Invalid extensionId for removeContextMenu');
    return;
  }
  
  if (!validateMenuItemId(menuItemId)) {
    console.error('[ContextMenus] Invalid menuItemId for removeContextMenu');
    return;
  }
  
  try {
    await invoke('extension_context_menus_remove', { extensionId, menuItemId });
  } catch (error) {
    console.error('[ContextMenus] extension_context_menus_remove failed:', error);
    throw error;
  }
}

/** chrome.contextMenus.removeAll - Remove all context menu items. */
export async function removeAllContextMenus(
  extensionId: string,
): Promise<void> {
  if (!isTauri()) return;
  
  // Aerospace-level input validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ContextMenus] Invalid extensionId for removeAllContextMenus');
    return;
  }
  
  try {
    await invoke('extension_context_menus_remove_all', { extensionId });
  } catch (error) {
    console.error('[ContextMenus] extension_context_menus_remove_all failed:', error);
    throw error;
  }
}
