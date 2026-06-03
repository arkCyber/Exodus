/**
 * Exodus Browser — settings auto-save composable tests.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ref, nextTick } from 'vue';
import {
  useChromeSettingsAutoSave,
  SETTINGS_TEXT_AUTOSAVE_MS,
  SETTINGS_PRIVACY_AUTOSAVE_MS,
} from './useChromeSettingsAutoSave';

vi.mock('@/lib/browserSettings', () => ({
  writeShowBookmarkBar: vi.fn(),
}));

import { writeShowBookmarkBar } from '@/lib/browserSettings';

function mockConfig() {
  return {
    showBookmarkBar: ref(true),
    httpsOnly: ref(false),
    privateMode: ref(false),
    blockPopups: ref(true),
    sessionRestore: ref(true),
    homepageUrl: ref('https://example.com'),
    searchEngineUrl: ref('https://example.com?q={query}'),
    spawnAllama: ref(true),
    aiPort: ref(11435),
    aiModel: ref('test-model'),
    savePrivacySettings: vi.fn(async () => {}),
    saveAiConfig: vi.fn(async () => {}),
  } as ReturnType<typeof import('./useBrowserConfig').useBrowserConfig>;
}

describe('useChromeSettingsAutoSave', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('persists bookmark bar immediately without blocking queue', async () => {
    const cfg = mockConfig();
    const onSaved = vi.fn();
    const { beginHydration, endHydration, startAutoSaveWatchers, stopAutoSaveWatchers, autoSaveState } =
      useChromeSettingsAutoSave({ config: cfg, onSaved });
    beginHydration();
    startAutoSaveWatchers();
    endHydration();
    cfg.showBookmarkBar.value = false;
    await nextTick();
    expect(writeShowBookmarkBar).toHaveBeenCalledWith(false);
    await vi.waitFor(() => expect(onSaved).toHaveBeenCalled(), { timeout: 1000 });
    expect(autoSaveState.value).toBe('saved');
    stopAutoSaveWatchers();
  });

  it('debounces privacy toggles into one save', async () => {
    vi.useFakeTimers();
    const cfg = mockConfig();
    const { beginHydration, endHydration, startAutoSaveWatchers, stopAutoSaveWatchers } =
      useChromeSettingsAutoSave({ config: cfg });
    beginHydration();
    startAutoSaveWatchers();
    endHydration();
    cfg.httpsOnly.value = true;
    cfg.privateMode.value = true;
    await nextTick();
    expect(cfg.savePrivacySettings).not.toHaveBeenCalled();
    vi.advanceTimersByTime(SETTINGS_PRIVACY_AUTOSAVE_MS + 50);
    await nextTick();
    await vi.waitFor(() => expect(cfg.savePrivacySettings).toHaveBeenCalledTimes(1));
    stopAutoSaveWatchers();
  });

  it('debounces homepage URL text changes', async () => {
    vi.useFakeTimers();
    const cfg = mockConfig();
    const { beginHydration, endHydration, startAutoSaveWatchers, stopAutoSaveWatchers } =
      useChromeSettingsAutoSave({ config: cfg });
    beginHydration();
    startAutoSaveWatchers();
    endHydration();
    cfg.homepageUrl.value = 'https://duckduckgo.com';
    await nextTick();
    expect(cfg.saveAiConfig).not.toHaveBeenCalled();
    vi.advanceTimersByTime(SETTINGS_TEXT_AUTOSAVE_MS + 50);
    await nextTick();
    await vi.waitFor(() => expect(cfg.saveAiConfig).toHaveBeenCalledTimes(1));
    stopAutoSaveWatchers();
  });

  it('uses reactive savedLabel from getter', async () => {
    vi.useFakeTimers();
    const cfg = mockConfig();
    const label = ref('Saved EN');
    const onStatus = vi.fn();
    const { beginHydration, endHydration, startAutoSaveWatchers, stopAutoSaveWatchers } =
      useChromeSettingsAutoSave({
        config: cfg,
        savedLabel: () => label.value,
        onStatus,
      });
    beginHydration();
    startAutoSaveWatchers();
    endHydration();
    cfg.httpsOnly.value = true;
    await nextTick();
    vi.advanceTimersByTime(SETTINGS_PRIVACY_AUTOSAVE_MS + 50);
    await vi.waitFor(() => expect(onStatus).toHaveBeenCalledWith('Saved EN'));
    label.value = 'Saved ZH';
    cfg.privateMode.value = true;
    await nextTick();
    vi.advanceTimersByTime(SETTINGS_PRIVACY_AUTOSAVE_MS + 50);
    await vi.waitFor(() => expect(onStatus).toHaveBeenCalledWith('Saved ZH'));
    stopAutoSaveWatchers();
    vi.useRealTimers();
  });
});
