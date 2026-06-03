/**
 * Exodus Browser — Singleton IM message sync (Tauri push + focus refresh + fallback poll).
 * One coordinator serves both full-width and sidebar ImMessenger instances.
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { isTauri } from '@tauri-apps/api/core';
import { IM_NEW_MESSAGE_EVENT } from '$lib/imChat';
import { bumpImStore } from '$lib/imStore';
import { logDebug } from '@/lib/logger';

export type ImMessageSyncHandlers = {
  pollActiveConversation: () => Promise<void>;
  syncContactPreviews: () => Promise<void>;
  refreshPresence: () => Promise<void>;
};

let started = false;
let handlers: ImMessageSyncHandlers | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;
let presenceTimer: ReturnType<typeof setInterval> | null = null;
let tauriUnlisten: UnlistenFn | null = null;

/** Long fallback when Rust sequence watcher and push events miss updates. */
const MESSAGE_POLL_FALLBACK_MS = 60_000;

/**
 * Register sync handlers and start listeners once for the whole app session.
 */
export function ensureImMessageSync(next: ImMessageSyncHandlers): void {
  handlers = next;
  if (started) return;
  started = true;

  window.addEventListener(IM_NEW_MESSAGE_EVENT, onWindowImEvent as EventListener);
  window.addEventListener('focus', onFocusRefresh);
  document.addEventListener('visibilitychange', onVisibilityRefresh);

  if (isTauri()) {
    void listen<{ groupId: string }>('exodus-im-new-message', (event) => {
      const groupId = event.payload?.groupId;
      if (groupId) void onRemoteMessage(groupId, 'tauri');
    }).then((unlisten) => {
      tauriUnlisten = unlisten;
    });
  }

  pollTimer = setInterval(() => void runFallbackPoll(), MESSAGE_POLL_FALLBACK_MS);
  presenceTimer = setInterval(() => void handlers?.refreshPresence(), 30_000);
  logDebug('imMessageSync', 'Started IM message sync coordinator');
}

async function onRemoteMessage(groupId: string, source: string): Promise<void> {
  logDebug('imMessageSync', 'New message push', { groupId, source });
  bumpImStore();
  await handlers?.pollActiveConversation();
  await handlers?.syncContactPreviews();
}

function onWindowImEvent(ev: Event): void {
  const groupId = (ev as CustomEvent<{ groupId: string }>).detail?.groupId;
  if (groupId) void onRemoteMessage(groupId, 'window');
}

function onFocusRefresh(): void {
  void runFallbackPoll();
}

function onVisibilityRefresh(): void {
  if (document.visibilityState === 'visible') void runFallbackPoll();
}

async function runFallbackPoll(): Promise<void> {
  await handlers?.pollActiveConversation();
  await handlers?.syncContactPreviews();
}

/** Tear down sync (tests only). */
export function resetImMessageSyncForTests(): void {
  window.removeEventListener(IM_NEW_MESSAGE_EVENT, onWindowImEvent as EventListener);
  window.removeEventListener('focus', onFocusRefresh);
  document.removeEventListener('visibilitychange', onVisibilityRefresh);
  if (pollTimer) clearInterval(pollTimer);
  if (presenceTimer) clearInterval(presenceTimer);
  pollTimer = null;
  presenceTimer = null;
  if (tauriUnlisten) void tauriUnlisten();
  tauriUnlisten = null;
  started = false;
  handlers = null;
}
