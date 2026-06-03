/**
 * Exodus Browser — Web Extension host events (tabs.create, runtime pump).
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { TabCreateAck } from '$lib/extensions/types';

/** Payload when an extension calls chrome.tabs.create. */
export type ExtensionTabCreateRequest = {
  requestId: string;
  url: string;
  active: boolean;
  sourceWebviewLabel?: string;
};

export type ExtensionTabsCreateEvent = {
  requests: ExtensionTabCreateRequest[];
};

/** Flush extension storage + runtime messages for a tab webview. */
export async function flushExtensionTab(webviewLabel: string): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke('browser_extension_flush_tab', { label: webviewLabel });
  } catch (error) {
    console.error('browser_extension_flush_tab failed:', error);
  }
}

/** Pump runtime message queues (active tab flush + background hosts). */
export async function pumpExtensionRuntime(activeWebviewLabel?: string): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke('extension_pump_runtime', {
      activeLabel: activeWebviewLabel ?? null,
    });
  } catch (error) {
    console.error('extension_pump_runtime failed:', error);
  }
}

/** Acknowledge completed tab creates to resolve chrome.tabs.create promises. */
export async function ackExtensionTabCreates(acks: TabCreateAck[]): Promise<void> {
  if (!isTauri() || acks.length === 0) return;
  try {
    await invoke('extension_tabs_create_ack', { acks });
  } catch (error) {
    console.error('extension_tabs_create_ack failed:', error);
  }
}

/**
 * Listen for extension-requested tab opens; caller creates tabs and sends acks.
 */
export function listenExtensionTabCreates(
  onCreate: (requests: ExtensionTabCreateRequest[]) => Promise<TabCreateAck[]>,
): Promise<UnlistenFn> {
  return listen<ExtensionTabsCreateEvent>('exodus-extension-tabs-create', async (event) => {
    const requests = event.payload.requests ?? [];
    if (requests.length === 0) return;
    try {
      const acks = await onCreate(requests);
      await ackExtensionTabCreates(acks);
    } catch (error) {
      console.error('exodus-extension-tabs-create handler failed:', error);
    }
  });
}

/** Permission prompt payload from `exodus-extension-permission-request`. */
export type ExtensionPermissionRequestEvent = {
  extensionId: string;
  extensionName: string;
  requestId: string;
  permissions: string[];
  sourceWebviewLabel?: string;
};

/** System notification payload from extension flush. */
export type ExtensionNotificationEvent = {
  extensionId: string;
  notificationId: string;
  title?: string;
  message?: string;
};

/**
 * Listen for extension permission requests (show UI, then resolve via invoke).
 */
export function listenExtensionPermissionRequests(
  onRequest: (req: ExtensionPermissionRequestEvent) => void,
): Promise<UnlistenFn> {
  return listen<ExtensionPermissionRequestEvent>(
    'exodus-extension-permission-request',
    (event) => {
      onRequest(event.payload);
    },
  );
}

/** Listen for extension-raised notifications (host may show OS toast). */
export function listenExtensionNotifications(
  onNotification: (note: ExtensionNotificationEvent) => void,
): Promise<UnlistenFn> {
  return listen<ExtensionNotificationEvent>('exodus-extension-notification', (event) => {
    onNotification(event.payload);
  });
}

/** Host permission denied for extension navigation. */
export type ExtensionHostDeniedEvent = {
  extensionId: string;
  url: string;
};

/** Listen when an extension tab update is blocked by host permissions. */
export function listenExtensionHostDenied(
  onDenied: (ev: ExtensionHostDeniedEvent) => void,
): Promise<UnlistenFn> {
  return listen<ExtensionHostDeniedEvent>('exodus-extension-host-denied', (event) => {
    onDenied(event.payload);
  });
}

/** Install-time host_permissions confirmation (`exodus-extension-host-install-request`). */
export type ExtensionHostInstallRequestEvent = {
  requestId: string;
  extensionId: string;
  extensionName: string;
  hostPermissions: string[];
};

/** Listen for install-time manifest host_permissions prompts. */
export function listenExtensionHostInstallRequests(
  onRequest: (req: ExtensionHostInstallRequestEvent) => void,
): Promise<UnlistenFn> {
  return listen<ExtensionHostInstallRequestEvent>(
    'exodus-extension-host-install-request',
    (event) => {
      onRequest(event.payload);
    },
  );
}

/** Browser site permission (camera / mic / geolocation) from page bridge. */
export type BrowserSitePermissionRequestEvent = {
  requestId: string;
  kind: string;
  origin: string;
  webviewLabel: string;
};

/** Listen for per-origin browser permission prompts. */
export function listenBrowserSitePermissionRequests(
  onRequest: (req: BrowserSitePermissionRequestEvent) => void,
): Promise<UnlistenFn> {
  return listen<BrowserSitePermissionRequestEvent>(
    'exodus-browser-site-permission-request',
    (event) => {
      onRequest(event.payload);
    },
  );
}
