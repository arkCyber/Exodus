/**
 * Exodus Browser — Chrome-style extension toolbar display rules.
 * Only enabled extensions pinned to the toolbar appear in the address bar strip.
 * Aerospace-level error handling and input validation.
 */

import { logStartup } from '@/lib/startupLog';
import { extLog } from '@/lib/diagnosticLog';

logStartup('extensionToolbarDisplay module loaded');

import type { ExtensionInfo } from '@/lib/extensions/types';

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;

/**
 * Aerospace-level validation for extension ID format.
 */
function validateExtensionId(extensionId: string): boolean {
  if (!extensionId || typeof extensionId !== 'string') {
    extLog.error('Invalid extension ID');
    return false;
  }
  return VALID_EXTENSION_ID_PATTERN.test(extensionId);
}

/**
 * Aerospace-level validation for extension data structure.
 */
function validateExtensionInfo(ext: ExtensionInfo): boolean {
  if (!ext || typeof ext !== 'object') {
    extLog.error('Invalid extension data');
    return false;
  }
  if (!ext.id || typeof ext.id !== 'string') {
    extLog.error('Invalid extension ID in extension data');
    return false;
  }
  if (!ext.name || typeof ext.name !== 'string') {
    extLog.error('Invalid extension name in extension data');
    return false;
  }
  return true;
}

/** Whether an extension should appear in the address bar toolbar (Chrome: pinned + enabled). */
export function isPinnedToolbarExtension(ext: ExtensionInfo): boolean {
  // Aerospace-level input validation
  if (!validateExtensionInfo(ext)) {
    return false;
  }
  
  if (!validateExtensionId(ext.id)) {
    extLog.error('Invalid extension ID format', ext.id);
    return false;
  }
  
  return ext.enabled && ext.pinned !== false;
}

/** Filter and sort extensions for the toolbar display module (alphabetical, like Chrome). */
export function pinnedToolbarExtensions(list: ExtensionInfo[]): ExtensionInfo[] {
  // Aerospace-level input validation
  if (!list || !Array.isArray(list)) {
    extLog.error('Invalid extension list');
    return [];
  }
  
  // Validate each extension before processing
  const validExtensions: ExtensionInfo[] = [];
  for (const ext of list) {
    if (validateExtensionInfo(ext) && validateExtensionId(ext.id)) {
      validExtensions.push(ext);
    } else {
      extLog.error('Skipping invalid extension in list');
    }
  }
  
  return validExtensions.filter(isPinnedToolbarExtension).sort((a, b) => a.name.localeCompare(b.name));
}

/** Tooltip for a pinned toolbar extension action button. */
export function pinnedToolbarActionTitle(
  ext: ExtensionInfo,
  options?: { nativePopups?: boolean },
): string {
  // Aerospace-level input validation
  if (!validateExtensionInfo(ext)) {
    return 'Extension (invalid)';
  }
  
  if (!validateExtensionId(ext.id)) {
    extLog.error('Invalid extension ID format in title', ext.id);
    return 'Extension (invalid)';
  }
  
  const nativePopups = options?.nativePopups ?? true;
  if (ext.actionPopup && !nativePopups) {
    return `${ext.name} (requires Tauri webview)`;
  }
  if (ext.actionPopup) return ext.name;
  return `${ext.name} — open Extensions`;
}

/** Summary label for the toolbar display module (accessibility / debugging). */
export function pinnedToolbarModuleLabel(count: number): string {
  // Aerospace-level input validation
  if (typeof count !== 'number' || count < 0 || !Number.isInteger(count)) {
    extLog.error('Invalid count for module label', count);
    return 'Extensions (invalid count)';
  }
  
  if (count === 0) return 'Extensions (none pinned)';
  if (count === 1) return 'Extensions (1 pinned)';
  return `Extensions (${count} pinned)`;
}
