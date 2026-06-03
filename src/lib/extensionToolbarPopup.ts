/**
 * Exodus Browser — Chrome-style embedded extension action popups in the main window.
 * Uses child webviews (`exodus-ext-popup-{id}`) anchored below toolbar icons instead of
 * separate popup windows so the user can click outside or press Escape to return to the page.
 * Aerospace-level error handling, security validation, and state management.
 */

import { extLog } from '@/lib/diagnosticLog';
import { invoke } from '@tauri-apps/api/core';
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/dpi';
import { Webview } from '@tauri-apps/api/webview';
import { closeTabWebview, canUseNativeWebview } from '@/lib/exodusBrowser';

extLog.info('extensionToolbarPopup module loaded');

/** Chrome default action popup width (px). */
export const CHROME_POPUP_WIDTH = 352;

/** Chrome default action popup height (px). */
export const CHROME_POPUP_HEIGHT = 512;

/** Gap between toolbar icon and popup panel (px). */
const POPUP_ANCHOR_GAP = 4;

/** Minimum inset from viewport edges (px). */
const VIEWPORT_INSET = 8;

/** z-index for transparent click-capture backdrop. */
export const EXTENSION_POPUP_BACKDROP_Z = 9998;

/** z-index for popup host shell and native webview. */
export const EXTENSION_POPUP_HOST_Z = 9999;

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+|^extension:\/\/.+/;

let popupHostEl: HTMLDivElement | null = null;
let currentAnchor: HTMLElement | null = null;
let currentExtensionId: string | null = null;
let layoutListener: (() => void) | null = null;

/**
 * Aerospace-level validation for extension ID format.
 * Prevents injection attacks and ensures safe webview label generation.
 */
function validateExtensionId(extensionId: string): boolean {
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ExtensionToolbarPopup] Invalid extension ID');
    return false;
  }
  return VALID_EXTENSION_ID_PATTERN.test(extensionId);
}

/**
 * Aerospace-level validation for popup URL format.
 * Prevents malicious URL injection.
 */
function validatePopupUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    console.error('[ExtensionToolbarPopup] Invalid popup URL');
    return false;
  }
  return VALID_URL_PATTERN.test(url);
}

/** Stable child webview label for an extension action popup. */
export function toolbarPopupLabel(extensionId: string): string {
  // Aerospace-level security validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ExtensionToolbarPopup] Invalid extension ID for label generation:', extensionId);
    return 'exodus-ext-popup-invalid';
  }
  const safe = extensionId.replace(/[^a-zA-Z0-9_-]/g, '');
  return `exodus-ext-popup-${safe}`;
}

/** Compute popup shell rect anchored below (or above) a toolbar button. */
export function computeToolbarPopupRect(anchor: DOMRect): DOMRect {
  const width = CHROME_POPUP_WIDTH;
  const height = CHROME_POPUP_HEIGHT;

  let left = anchor.left + anchor.width / 2 - width / 2;
  left = Math.max(VIEWPORT_INSET, Math.min(left, window.innerWidth - width - VIEWPORT_INSET));

  let top = anchor.bottom + POPUP_ANCHOR_GAP;
  if (top + height > window.innerHeight - VIEWPORT_INSET) {
    const aboveTop = anchor.top - height - POPUP_ANCHOR_GAP;
    if (aboveTop >= VIEWPORT_INSET) {
      top = aboveTop;
    } else {
      top = Math.max(VIEWPORT_INSET, window.innerHeight - VIEWPORT_INSET - height);
    }
  }

  return new DOMRect(left, top, width, height);
}

/** Whether an embedded toolbar popup webview exists for the extension. */
export async function isToolbarExtensionPopupOpen(extensionId: string): Promise<boolean> {
  const wv = await Webview.getByLabel(toolbarPopupLabel(extensionId));
  return wv != null;
}

function ensurePopupHost(): HTMLDivElement {
  if (popupHostEl) return popupHostEl;

  const shell = document.createElement('div');
  shell.className = 'exodus-extension-popup-host';
  shell.style.position = 'fixed';
  shell.style.zIndex = String(EXTENSION_POPUP_HOST_Z);
  shell.style.pointerEvents = 'auto';
  shell.style.overflow = 'hidden';
  shell.style.borderRadius = '8px';
  shell.style.boxShadow = '0 4px 24px rgba(0, 0, 0, 0.18), 0 0 0 1px rgba(0, 0, 0, 0.08)';
  shell.style.background = '#fff';
  document.body.appendChild(shell);
  popupHostEl = shell;
  return shell;
}

function layoutPopupHost(anchor: HTMLElement): DOMRect {
  const shell = ensurePopupHost();
  const rect = computeToolbarPopupRect(anchor.getBoundingClientRect());
  shell.style.left = `${rect.left}px`;
  shell.style.top = `${rect.top}px`;
  shell.style.width = `${rect.width}px`;
  shell.style.height = `${rect.height}px`;
  return rect;
}

async function relayoutPopupWebview(): Promise<void> {
  if (!currentAnchor || !currentExtensionId) return;
  const rect = layoutPopupHost(currentAnchor);
  const label = toolbarPopupLabel(currentExtensionId);
  const wv = await Webview.getByLabel(label);
  if (!wv) return;
  try {
    await wv.setPosition(new LogicalPosition(rect.left, rect.top));
    await wv.setSize(new LogicalSize(rect.width, rect.height));
  } catch (error) {
    console.error('extension toolbar popup relayout failed:', error);
  }
}

function attachLayoutListeners(): void {
  detachLayoutListeners();
  layoutListener = () => {
    void relayoutPopupWebview();
  };
  window.addEventListener('resize', layoutListener);
  window.addEventListener('scroll', layoutListener, true);
}

function detachLayoutListeners(): void {
  if (!layoutListener) return;
  window.removeEventListener('resize', layoutListener);
  window.removeEventListener('scroll', layoutListener, true);
  layoutListener = null;
}

function removePopupHost(): void {
  if (popupHostEl) {
    popupHostEl.remove();
    popupHostEl = null;
  }
  currentAnchor = null;
  currentExtensionId = null;
  detachLayoutListeners();
}

/** Open an embedded extension action popup below a toolbar icon. */
export async function openToolbarExtensionPopup(options: {
  extensionId: string;
  popupUrl: string;
  anchor: HTMLElement;
}): Promise<Webview> {
  if (!canUseNativeWebview()) {
    throw new Error('Native webview required for extension toolbar popup');
  }

  const { extensionId, popupUrl, anchor } = options;
  
  // Aerospace-level security validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ExtensionToolbarPopup] Invalid extension ID for popup:', extensionId);
    throw new Error('Invalid extension ID');
  }
  
  if (!validatePopupUrl(popupUrl)) {
    console.error('[ExtensionToolbarPopup] Invalid popup URL:', popupUrl);
    throw new Error('Invalid popup URL');
  }
  
  // Aerospace-level input validation for anchor
  if (!anchor || !(anchor instanceof HTMLElement)) {
    console.error('[ExtensionToolbarPopup] Invalid anchor element');
    throw new Error('Invalid anchor element');
  }

  const label = toolbarPopupLabel(extensionId);

  const existing = await Webview.getByLabel(label);
  if (existing) {
    try {
      await existing.close();
    } catch (error) {
      console.error('close existing extension popup webview failed:', error);
    }
  }

  currentAnchor = anchor;
  currentExtensionId = extensionId;

  const rect = layoutPopupHost(anchor);

  try {
    await invoke('browser_create_tab', {
      label,
      url: popupUrl,
      x: rect.left,
      y: rect.top,
      width: Math.max(rect.width, 100),
      height: Math.max(rect.height, 100),
    });
  } catch (error) {
    removePopupHost();
    console.error('browser_create_tab for extension popup failed:', error);
    throw error;
  }

  const webview = await Webview.getByLabel(label);
  if (!webview) {
    removePopupHost();
    throw new Error(`Extension popup webview not found after create: ${label}`);
  }

  try {
    await webview.setPosition(new LogicalPosition(rect.left, rect.top));
    await webview.setSize(new LogicalSize(rect.width, rect.height));
    await webview.show();
  } catch (error) {
    console.error('extension popup webview show failed:', error);
    await closeToolbarExtensionPopup(extensionId);
    throw error;
  }

  attachLayoutListeners();
  return webview;
}

/** Close embedded extension popup and remove its host shell. */
export async function closeToolbarExtensionPopup(extensionId: string): Promise<void> {
  // Aerospace-level security validation
  if (!validateExtensionId(extensionId)) {
    console.error('[ExtensionToolbarPopup] Invalid extension ID for close:', extensionId);
    return;
  }
  
  const label = toolbarPopupLabel(extensionId);
  try {
    await closeTabWebview(label);
  } catch (error) {
    console.error('closeTabWebview for extension popup failed:', error);
  }
  removePopupHost();
}

/** Close any open embedded toolbar popup. */
export async function closeAnyToolbarExtensionPopup(): Promise<string | null> {
  const id = currentExtensionId;
  if (!id) return null;
  await closeToolbarExtensionPopup(id);
  return id;
}
