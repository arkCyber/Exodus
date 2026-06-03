/**
 * Exodus Browser — auto-save hooks for Chrome settings (per-field persistence).
 */

import { logStartup } from '@/lib/startupLog';
import { ref, watch, toValue, type MaybeRefOrGetter, type WatchStopHandle } from 'vue';
import { writeShowBookmarkBar } from '@/lib/browserSettings';
import type { useBrowserConfig } from '@/composables/useBrowserConfig';

logStartup('useChromeSettingsAutoSave module loaded');

/** Debounce delay for text fields (homepage, search URL). */
export const SETTINGS_TEXT_AUTOSAVE_MS = 600;

/** Debounce delay for privacy toggles (coalesce rapid checkbox changes). */
export const SETTINGS_PRIVACY_AUTOSAVE_MS = 400;

/** UI state for the settings footer status indicator. */
export type SettingsAutoSaveState = 'idle' | 'saving' | 'saved' | 'error';

export type BrowserConfigStore = ReturnType<typeof useBrowserConfig>;

export type UseChromeSettingsAutoSaveOptions = {
  config: BrowserConfigStore;
  /** Called after a successful persist (refresh shell UI). Debounced internally. */
  onSaved?: () => void;
  /** Optional status line in the browser chrome (skipped for instant local saves). */
  onStatus?: (message: string) => void;
  /** Localized "saved" label for status messages (reactive when ref/computed/getter). */
  savedLabel?: MaybeRefOrGetter<string>;
};

/**
 * Watch browser config refs and persist each change (checkboxes immediately, URLs debounced).
 * Saves are serialized to avoid overlapping Tauri invokes (macOS busy cursor).
 */
export function useChromeSettingsAutoSave(options: UseChromeSettingsAutoSaveOptions) {
  const { config, onSaved, onStatus, savedLabel } = options;

  function resolveSavedLabel(): string {
    return toValue(savedLabel) ?? 'Settings saved';
  }

  const autoSaveState = ref<SettingsAutoSaveState>('idle');
  let hydrating = true;
  let textDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let privacyDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let savedIdleTimer: ReturnType<typeof setTimeout> | null = null;
  let savedEmitTimer: ReturnType<typeof setTimeout> | null = null;
  let saveChain: Promise<void> = Promise.resolve();
  const stopHandles: WatchStopHandle[] = [];

  function setState(next: SettingsAutoSaveState): void {
    autoSaveState.value = next;
  }

  function scheduleSavedEmit(): void {
    if (!onSaved) return;
    if (savedEmitTimer) clearTimeout(savedEmitTimer);
    savedEmitTimer = setTimeout(() => {
      savedEmitTimer = null;
      onSaved();
    }, 350);
  }

  function flashSaved(opts?: { status?: boolean }): void {
    setState('saved');
    scheduleSavedEmit();
    if (opts?.status !== false) {
      onStatus?.(resolveSavedLabel());
    }
    if (savedIdleTimer) clearTimeout(savedIdleTimer);
    savedIdleTimer = setTimeout(() => {
      if (autoSaveState.value === 'saved') setState('idle');
    }, 2000);
  }

  function enqueuePersist(run: () => Promise<void>, opts?: { showSaving?: boolean }): void {
    if (opts?.showSaving !== false) {
      setState('saving');
    }
    saveChain = saveChain
      .then(async () => {
        await run();
        flashSaved({ status: opts?.showSaving !== false });
      })
      .catch((error) => {
        console.error('settings auto-save failed:', error);
        setState('error');
      });
  }

  function persistBookmarkBar(): void {
    try {
      writeShowBookmarkBar(config.showBookmarkBar.value);
      flashSaved({ status: false });
    } catch (error) {
      console.error('persistBookmarkBar failed:', error);
      setState('error');
    }
  }

  function schedulePrivacyPersist(): void {
    if (hydrating) return;
    if (privacyDebounceTimer) clearTimeout(privacyDebounceTimer);
    privacyDebounceTimer = setTimeout(() => {
      privacyDebounceTimer = null;
      enqueuePersist(() => config.savePrivacySettings());
    }, SETTINGS_PRIVACY_AUTOSAVE_MS);
  }

  function scheduleTextPersist(): void {
    if (hydrating) return;
    if (textDebounceTimer) clearTimeout(textDebounceTimer);
    textDebounceTimer = setTimeout(() => {
      textDebounceTimer = null;
      enqueuePersist(() => config.saveAiConfig());
    }, SETTINGS_TEXT_AUTOSAVE_MS);
  }

  function scheduleAiPersist(): void {
    if (hydrating) return;
    enqueuePersist(() => config.saveAiConfig());
  }

  function guardWatch<T>(source: () => T, cb: () => void): void {
    stopHandles.push(
      watch(source, () => {
        if (hydrating) return;
        cb();
      }),
    );
  }

  /** Register watchers; call once after mount. */
  function startAutoSaveWatchers(): void {
    guardWatch(() => config.showBookmarkBar.value, () => persistBookmarkBar());
    guardWatch(() => config.httpsOnly.value, schedulePrivacyPersist);
    guardWatch(() => config.privateMode.value, schedulePrivacyPersist);
    guardWatch(() => config.blockPopups.value, schedulePrivacyPersist);
    guardWatch(() => config.sessionRestore.value, schedulePrivacyPersist);
    guardWatch(() => config.spawnAllama.value, scheduleAiPersist);
    guardWatch(() => config.aiPort.value, scheduleAiPersist);
    guardWatch(() => config.aiModel.value, scheduleAiPersist);
    guardWatch(() => config.homepageUrl.value, scheduleTextPersist);
    guardWatch(() => config.searchEngineUrl.value, scheduleTextPersist);
  }

  /** Suppress auto-save while loading config from backend. */
  function beginHydration(): void {
    hydrating = true;
  }

  /** Enable auto-save after initial load completes. */
  function endHydration(): void {
    hydrating = false;
  }

  function stopAutoSaveWatchers(): void {
    if (textDebounceTimer) clearTimeout(textDebounceTimer);
    if (privacyDebounceTimer) clearTimeout(privacyDebounceTimer);
    if (savedIdleTimer) clearTimeout(savedIdleTimer);
    if (savedEmitTimer) clearTimeout(savedEmitTimer);
    saveChain = Promise.resolve();
    for (const stop of stopHandles) stop();
    stopHandles.length = 0;
  }

  return {
    autoSaveState,
    beginHydration,
    endHydration,
    startAutoSaveWatchers,
    stopAutoSaveWatchers,
    persistBookmarkBar,
    schedulePrivacyPersist,
    scheduleAiPersist,
  };
}
