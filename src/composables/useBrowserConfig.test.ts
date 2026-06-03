/**
 * Exodus Browser — useBrowserConfig composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useBrowserConfig } from './useBrowserConfig';

let tauriRuntime = true;

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => tauriRuntime,
}));

vi.mock('$lib/browserSettings', () => ({
  readShowBookmarkBar: () => true,
  writeShowBookmarkBar: vi.fn(),
}));

describe('useBrowserConfig', () => {
  beforeEach(() => {
    tauriRuntime = true;
    vi.clearAllMocks();
  });

  it('initializes with default values', () => {
    const { aiPort, aiModel, homepageUrl, searchEngineUrl, loading } = useBrowserConfig();

    expect(aiPort.value).toBe(11435);
    expect(aiModel.value).toBe('exodus-default');
    expect(homepageUrl.value).toBe('https://duckduckgo.com');
    expect(searchEngineUrl.value).toBe('https://duckduckgo.com/?q={query}');
    expect(loading.value).toBe(false);
  });

  it('loads config from Tauri', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockConfig = {
      ai_port: 11435,
      ai_model: 'test-model',
      embedding_model: 'test-embedding',
      homepage_url: 'https://test.com',
      search_engine_url: 'https://test.com/search?q={query}',
      status_clear_ms: 5000,
      spawn_sidecar: true,
      spawn_allama: false,
    };
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockConfig)
      .mockResolvedValueOnce([true, false, true, false]);

    const { load, aiModel, homepageUrl, httpsOnly, privateMode } = useBrowserConfig();

    await load();

    expect(aiModel.value).toBe('test-model');
    expect(homepageUrl.value).toBe('https://test.com');
    expect(httpsOnly.value).toBe(true);
    expect(privateMode.value).toBe(false);
  });

  it('handles load errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Load failed'));

    const { load, loading } = useBrowserConfig();

    await load();

    expect(loading.value).toBe(false);
  });

  it('does not load when not in Tauri', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    tauriRuntime = false;

    const { load: configLoad } = useBrowserConfig();

    await configLoad();

    expect(invoke).not.toHaveBeenCalled();
  });

  it('saves AI config', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { saveAiConfig, aiPort, aiModel } = useBrowserConfig();
    aiPort.value = 12345;
    aiModel.value = 'new-model';

    await saveAiConfig();

    expect(invoke).toHaveBeenCalledWith('set_ai_config', {
      aiPort: 12345,
      aiModel: 'new-model',
      embeddingModel: 'nomic-embed-text',
      homepageUrl: 'https://duckduckgo.com',
      searchEngineUrl: 'https://duckduckgo.com/?q={query}',
      statusClearMs: 4000,
      spawnSidecar: false,
      spawnAllama: true,
    });
  });

  it('saves privacy settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { savePrivacySettings, httpsOnly, privateMode } = useBrowserConfig();
    httpsOnly.value = true;
    privateMode.value = true;

    await savePrivacySettings();

    expect(invoke).toHaveBeenCalledWith('set_privacy_settings', {
      httpsOnly: true,
      privateMode: true,
      blockPopups: true,
      sessionRestore: true,
    });
  });

  it('clears session when session restore is disabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { savePrivacySettings, sessionRestore } = useBrowserConfig();
    sessionRestore.value = false;

    await savePrivacySettings();

    expect(invoke).toHaveBeenCalledWith('clear_session');
  });

  it('clears session when private mode is enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { savePrivacySettings, privateMode } = useBrowserConfig();
    privateMode.value = true;

    await savePrivacySettings();

    expect(invoke).toHaveBeenCalledWith('clear_session');
  });

  it('saves all settings', async () => {
    const { writeShowBookmarkBar } = await import('$lib/browserSettings');
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    const { saveAll, showBookmarkBar } = useBrowserConfig();
    showBookmarkBar.value = false;

    await saveAll();

    expect(writeShowBookmarkBar).toHaveBeenCalledWith(false);
    expect(invoke).toHaveBeenCalledWith('set_ai_config', expect.any(Object));
    expect(invoke).toHaveBeenCalledWith('set_privacy_settings', expect.any(Object));
  });

  it('handles clear session errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke)
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error('Clear failed'));

    const { savePrivacySettings, sessionRestore } = useBrowserConfig();
    sessionRestore.value = false;

    await expect(savePrivacySettings()).resolves.not.toThrow();
  });
});
