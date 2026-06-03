/**
 * Exodus Browser — load/save app config (Tauri ExodusConfig + privacy settings).
 */

import { ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { ExodusConfigDto } from '@/lib/browserSettings';
import { readShowBookmarkBar, writeShowBookmarkBar } from '@/lib/browserSettings';

const defaultConfig = (): ExodusConfigDto => ({
  ai_port: 11435,
  ai_model: 'exodus-default',
  embedding_model: 'nomic-embed-text',
  homepage_url: 'https://duckduckgo.com',
  search_engine_url: 'https://duckduckgo.com/?q={query}',
  status_clear_ms: 4000,
  spawn_sidecar: false,
  spawn_allama: true,
  confirm_host_permissions_on_install: true,
});

type BrowserConfigStore = ReturnType<typeof createBrowserConfigStore>;

let sharedConfig: BrowserConfigStore | null = null;

function createBrowserConfigStore() {
  const loading = ref(false);
  const aiPort = ref(11435);
  const aiModel = ref('exodus-default');
  const embeddingModel = ref('nomic-embed-text');
  const homepageUrl = ref('https://duckduckgo.com');
  const searchEngineUrl = ref('https://duckduckgo.com/?q={query}');
  const statusClearMs = ref(4000);
  const spawnSidecar = ref(false);
  const spawnAllama = ref(true);
  const httpsOnly = ref(false);
  const privateMode = ref(false);
  const blockPopups = ref(true);
  const sessionRestore = ref(true);
  const showBookmarkBar = ref(readShowBookmarkBar());

  /** Load config from Tauri backend. */
  async function load(): Promise<void> {
    if (!isTauri()) return;
    loading.value = true;
    try {
      const cfg = await invoke<ExodusConfigDto>('get_ai_config');
      aiPort.value = cfg.ai_port ?? defaultConfig().ai_port;
      aiModel.value = cfg.ai_model ?? defaultConfig().ai_model;
      embeddingModel.value = cfg.embedding_model ?? defaultConfig().embedding_model;
      homepageUrl.value = cfg.homepage_url ?? defaultConfig().homepage_url;
      searchEngineUrl.value = cfg.search_engine_url ?? defaultConfig().search_engine_url;
      statusClearMs.value = cfg.status_clear_ms ?? defaultConfig().status_clear_ms;
      spawnSidecar.value = cfg.spawn_sidecar ?? false;
      spawnAllama.value = cfg.spawn_allama ?? true;

      const [https, priv, popups, restore] = await invoke<[boolean, boolean, boolean, boolean]>(
        'get_privacy_settings',
      );
      httpsOnly.value = https;
      privateMode.value = priv;
      blockPopups.value = popups;
      sessionRestore.value = restore;
    } catch (error) {
      console.error('useBrowserConfig.load failed:', error);
    } finally {
      loading.value = false;
    }
  }

  /** Persist AI / browser URLs to ExodusConfig. */
  async function saveAiConfig(): Promise<void> {
    if (!isTauri()) return;
    await invoke('set_ai_config', {
      aiPort: aiPort.value,
      aiModel: aiModel.value,
      embeddingModel: embeddingModel.value,
      homepageUrl: homepageUrl.value,
      searchEngineUrl: searchEngineUrl.value,
      statusClearMs: statusClearMs.value,
      spawnSidecar: spawnSidecar.value,
      spawnAllama: spawnAllama.value,
    });
  }

  /** Persist privacy toggles. */
  async function savePrivacySettings(): Promise<void> {
    if (!isTauri()) return;
    await invoke('set_privacy_settings', {
      httpsOnly: httpsOnly.value,
      privateMode: privateMode.value,
      blockPopups: blockPopups.value,
      sessionRestore: sessionRestore.value,
    });
    if (!sessionRestore.value || privateMode.value) {
      try {
        await invoke('clear_session');
      } catch (error) {
        console.error('clear_session failed:', error);
      }
    }
  }

  /** Save AI + privacy and bookmark bar preference. */
  async function saveAll(): Promise<void> {
    writeShowBookmarkBar(showBookmarkBar.value);
    await saveAiConfig();
    await savePrivacySettings();
  }

  return {
    loading,
    aiPort,
    aiModel,
    embeddingModel,
    homepageUrl,
    searchEngineUrl,
    statusClearMs,
    spawnSidecar,
    spawnAllama,
    httpsOnly,
    privateMode,
    blockPopups,
    sessionRestore,
    showBookmarkBar,
    load,
    saveAll,
    saveAiConfig,
    savePrivacySettings,
  };
}

/**
 * Reactive browser + AI config shared by the shell and settings modal.
 */
export function useBrowserConfig(): BrowserConfigStore {
  if (!sharedConfig) {
    sharedConfig = createBrowserConfigStore();
  }
  return sharedConfig;
}
