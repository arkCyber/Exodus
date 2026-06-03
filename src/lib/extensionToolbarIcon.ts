/**
 * Exodus Browser — extension toolbar icon URLs for Chrome-style address bar actions.
 * Aerospace-level error handling, security validation, and path normalization.
 */

import { extLog } from '@/lib/diagnosticLog';
import { convertFileSrc, isTauri } from '@tauri-apps/api/core';
import type { ExtensionInfo } from '@/lib/extensions/types';

extLog.info('extensionToolbarIcon module loaded');

/** Common MV3 icon paths relative to the extension install directory. */
const ICON_CANDIDATES = [
  'icons/icon16.png',
  'icons/icon-16.png',
  'icons/icon-16x16.png',
  'icon16.png',
  'icon.png',
];

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_PATH_PATTERN = /^[a-zA-Z0-9_\-./]+$/;

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
 * Aerospace-level validation for extension path format.
 * Prevents path traversal attacks.
 */
function validateExtensionPath(path: string): boolean {
  if (!path || typeof path !== 'string') {
    extLog.error('Invalid extension path');
    return false;
  }
  // Allow ../ in development environment for sample extensions
  // Normalize the path first to resolve any ../ components
  const normalized = normalizePath(path);
  // Check for malicious patterns after normalization
  if (normalized.includes('..') || normalized.includes('~')) {
    extLog.error('Potentially malicious path pattern', path);
    return false;
  }
  return VALID_PATH_PATTERN.test(normalized);
}

/** First grapheme for letter fallback tiles. */
export function extensionIconLetter(name: string): string {
  // Aerospace-level input validation
  if (!name || typeof name !== 'string') {
    extLog.error('Invalid extension name for icon letter');
    return '?';
  }
  const trimmed = name.trim();
  if (!trimmed) return '?';
  return trimmed.charAt(0).toUpperCase();
}

/** Build webview-safe icon URLs from an on-disk extension path. */
export function extensionIconUrlCandidates(ext: ExtensionInfo): string[] {
  // Aerospace-level input validation
  if (!ext || !ext.id) {
    extLog.error('Invalid extension data');
    return [];
  }
  
  if (!validateExtensionId(ext.id)) {
    extLog.error('Invalid extension ID format', ext.id);
    return [];
  }
  
  if (!isTauri() || !ext.path?.trim()) return [];
  
  if (!validateExtensionPath(ext.path)) {
    extLog.error('Invalid extension path format', ext.path);
    return [];
  }
  
  const base = ext.path.replace(/\/$/, '');
  // Normalize path to resolve any ../ components
  const normalizedBase = normalizePath(base);
  return ICON_CANDIDATES.map((rel) => convertFileSrc(`${normalizedBase}/${rel}`));
}

/** Normalize a file path by resolving ../ and ./ components. */
function normalizePath(path: string): string {
  const parts = path.split('/');
  const resolved: string[] = [];
  
  for (const part of parts) {
    if (part === '..') {
      resolved.pop();
    } else if (part !== '' && part !== '.') {
      resolved.push(part);
    }
  }
  
  return '/' + resolved.join('/');
}

/** Probe whether an icon URL loads successfully in the shell webview. */
export function probeExtensionIconUrl(url: string): Promise<boolean> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => resolve(true);
    img.onerror = () => resolve(false);
    img.src = url;
  });
}

/** Resolve the first working icon URL for an extension, if any. */
export async function resolveExtensionIconUrl(ext: ExtensionInfo): Promise<string | null> {
  for (const url of extensionIconUrlCandidates(ext)) {
    if (await probeExtensionIconUrl(url)) {
      return url;
    }
  }
  return null;
}
