/**
 * Exodus Browser — Vue 3 composable for Web Extension host integration.
 * Bridges Rust extension runtime events to the browser shell (tabs, prompts, pump).
 */

import { ref, onUnmounted, type Ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { logPerf, perfAsync, perfStart, perfEnd } from '@/lib/perfLog';
import { listen } from '@tauri-apps/api/event';
import { syncExtensionTabs } from '@/lib/extensions/syncTabs';
import { ensureExtensionBackgrounds } from '@/lib/extensions/backgroundHosts';
import { listenExtensionTabOps, type ExtensionTabOp } from '@/lib/extensions/tabOps';
import {
  flushExtensionTab,
  pumpExtensionRuntime,
  listenExtensionTabCreates,
  listenExtensionPermissionRequests,
  listenExtensionHostInstallRequests,
  listenExtensionNotifications,
  listenExtensionHostDenied,
  type ExtensionPermissionRequestEvent,
  type ExtensionHostInstallRequestEvent,
  type ExtensionTabCreateRequest,
} from '@/lib/extensions/extensionEvents';
import type { TabCreateAck } from '@/lib/extensions/types';
import type { BrowserTab } from '@/lib/browserTypes';
import { canUseNativeWebview, tabWebviewLabel } from '@/lib/exodusBrowser';

/** Minimal tab shape for chrome.tabs registry sync. */
export type ExtensionTabLike = {
  id: string;
  title: string;
  url: string;
};

export type UseExtensionsOptions = {
  getTabs: () => ExtensionTabLike[];
  getActiveTabId: () => string | null;
  contentHost: Ref<HTMLElement | undefined>;
  onTabOps: (ops: ExtensionTabOp[]) => void | Promise<void>;
  onTabCreates: (requests: ExtensionTabCreateRequest[]) => Promise<TabCreateAck[]>;
  onStatus?: (message: string) => void;
};

/** Enqueue a prompt when one is already visible. */
function enqueuePrompt<T>(
  active: T | null,
  queue: T[],
  item: T,
): { active: T | null; queue: T[] } {
  if (!active) return { active: item, queue };
  return { active, queue: [...queue, item] };
}

/**
 * Wire extension runtime listeners, tab registry sync, and permission prompts.
 */
export function useExtensions(options: UseExtensionsOptions) {
  const useNativeWebview = canUseNativeWebview();
  const permRequest = ref<ExtensionPermissionRequestEvent | null>(null);
  const permQueue = ref<ExtensionPermissionRequestEvent[]>([]);
  const hostInstallRequest = ref<ExtensionHostInstallRequestEvent | null>(null);
  const hostInstallQueue = ref<ExtensionHostInstallRequestEvent[]>([]);

  const unlisteners: Array<() => void> = [];
  /** Single-flight lazy init — avoids startup WebView storm (macOS busy cursor). */
  let backgroundsPromise: Promise<void> | null = null;

  /** Push open tabs to the Rust chrome.tabs registry. */
  function syncRegistry(): void {
    const activeId = options.getActiveTabId();
    if (!activeId) return;
    void syncExtensionTabs(options.getTabs() as BrowserTab[], activeId);
  }

  /** Active tab webview label for flush/pump. */
  function activeLabel(): string | undefined {
    const id = options.getActiveTabId();
    return id ? tabWebviewLabel(id) : undefined;
  }

  /**
   * Create extension background WebViews only when needed (not at startup).
   * requestIdleCallback fires when idle (~few seconds), which caused the busy cursor.
   */
  function ensureBackgroundsLazy(): Promise<void> {
    const host = options.contentHost.value;
    if (!useNativeWebview || !host) {
      return Promise.resolve();
    }
    if (!backgroundsPromise) {
      logPerf('extensions.ensureBackgrounds: lazy start');
      backgroundsPromise = perfAsync('extensions.ensureBackgrounds', () =>
        ensureExtensionBackgrounds(host),
      ).catch((error) => {
        backgroundsPromise = null;
        throw error;
      });
    }
    return backgroundsPromise;
  }

  /** Pump background + content-script runtime queues. */
  async function pump(): Promise<void> {
    await ensureBackgroundsLazy();
    await pumpExtensionRuntime(activeLabel());
  }

  function dismissPermPrompt(): void {
    permRequest.value = permQueue.value.shift() ?? null;
  }

  function dismissHostInstallPrompt(): void {
    hostInstallRequest.value = hostInstallQueue.value.shift() ?? null;
  }

  /** Create off-screen background service worker webviews (explicit / lazy). */
  async function initBackgrounds(): Promise<void> {
    await ensureBackgroundsLazy();
  }

  /** Register Tauri event listeners (idempotent per mount). */
  async function setup(): Promise<void> {
    if (!isTauri()) return;

    perfStart('extensions.setup');
    syncRegistry();

    unlisteners.push(await listenExtensionTabOps((ops) => options.onTabOps(ops)));

    unlisteners.push(
      await listenExtensionPermissionRequests((req) => {
        const next = enqueuePrompt(permRequest.value, permQueue.value, req);
        permRequest.value = next.active;
        permQueue.value = next.queue;
      }),
    );

    unlisteners.push(
      await listenExtensionHostInstallRequests((req) => {
        const next = enqueuePrompt(hostInstallRequest.value, hostInstallQueue.value, req);
        hostInstallRequest.value = next.active;
        hostInstallQueue.value = next.queue;
      }),
    );

    unlisteners.push(
      await listenExtensionHostDenied((ev) => {
        options.onStatus?.(`Extension ${ev.extensionId} blocked: ${ev.url}`);
      }),
    );

    unlisteners.push(
      await listenExtensionNotifications((note) => {
        const title = note.title ?? 'Extension';
        const body = note.message ?? '';
        if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
          new Notification(title, { body });
        } else {
          options.onStatus?.(`${title}: ${body}`);
        }
      }),
    );

    unlisteners.push(
      await listen<string>('exodus-extension-open-popup', (event) => {
        void invoke('extension_open_popup_window', { extensionId: event.payload }).catch((error) => {
          console.error('extension_open_popup_window failed:', error);
        });
      }),
    );

    unlisteners.push(
      await listenExtensionTabCreates(async (requests) => options.onTabCreates(requests)),
    );

    // Background WebViews are created on first pump() / initBackgrounds() — not on idle timer.
    logPerf('extensions.setup: backgrounds deferred (lazy)');
    perfEnd('extensions.setup');
  }

  function teardown(): void {
    for (const u of unlisteners) u();
    unlisteners.length = 0;
  }

  onUnmounted(teardown);

  return {
    useNativeWebview,
    permRequest,
    hostInstallRequest,
    syncRegistry,
    flushTab: flushExtensionTab,
    pump,
    activeLabel,
    initBackgrounds,
    setup,
    teardown,
    dismissPermPrompt,
    dismissHostInstallPrompt,
    ensureBackgroundsLazy,
  };
}
