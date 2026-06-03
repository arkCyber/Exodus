/**
 * Exodus Browser — Tauri download manager integration (progress events + invoke).
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DownloadRecord } from '@/lib/browserTypes';
import { loadPersistedDownloads } from '@/lib/downloadsPersist';

export type UseBrowserDownloadsOptions = {
  onStatus?: (message: string) => void;
};

/**
 * Reactive download list wired to Rust `download_url` and `exodus-download-*` events.
 */
export function useBrowserDownloads(options: UseBrowserDownloadsOptions = {}) {
  const downloads = ref<DownloadRecord[]>([]);
  const showDownloadsPanel = ref(false);

  const activeDownloadsCount = computed(() =>
    downloads.value.filter((d) => d.status === 'downloading' || d.status === 'pending').length,
  );

  let unlistenProgress: UnlistenFn | undefined;
  let unlistenDone: UnlistenFn | undefined;
  let unlistenError: UnlistenFn | undefined;
  let unlistenRequested: UnlistenFn | undefined;

  function openDownloadsPanel(): void {
    showDownloadsPanel.value = true;
  }

  function closeDownloadsPanel(): void {
    showDownloadsPanel.value = false;
  }

  function clearDownloads(): void {
    downloads.value = [];
  }

  /**
   * Start downloading a URL via the Rust download manager.
   */
  async function startDownload(url: string, filename?: string): Promise<void> {
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      options.onStatus?.('Cannot download this URL');
      return;
    }
    const id = crypto.randomUUID().slice(0, 8);
    const record: DownloadRecord = {
      id,
      url,
      filename: filename || url.split('/').pop()?.split('?')[0] || 'download',
      status: 'pending',
      progress: 0,
      received: 0,
      total: 0,
    };
    downloads.value = [record, ...downloads.value];
    showDownloadsPanel.value = true;
    try {
      downloads.value = downloads.value.map((d) =>
        d.id === id ? { ...d, status: 'downloading' as const } : d,
      );
      await invoke('download_url', { id, url, filename: filename ?? null });
    } catch (error) {
      downloads.value = downloads.value.map((d) =>
        d.id === id ? { ...d, status: 'failed' as const } : d,
      );
      const msg = error instanceof Error ? error.message : String(error);
      options.onStatus?.(`Download failed: ${msg}`);
    }
  }

  async function openDownloadsDir(): Promise<void> {
    try {
      await invoke('open_downloads_folder');
    } catch (error) {
      console.error('Open downloads folder failed:', error);
      options.onStatus?.('Could not open downloads folder');
    }
  }

  async function openDownloadFile(path: string): Promise<void> {
    try {
      await invoke('open_download', { path });
    } catch (error) {
      console.error('open_download failed:', error);
      options.onStatus?.('Could not open file');
    }
  }

  async function revealDownloadFile(path: string): Promise<void> {
    try {
      await invoke('reveal_download', { path });
    } catch (error) {
      console.error('reveal_download failed:', error);
      options.onStatus?.('Could not reveal file');
    }
  }

  async function setupListeners(): Promise<void> {
    if (!isTauri()) return;
    const persisted = await loadPersistedDownloads();
    if (persisted.length > 0) {
      downloads.value = persisted;
    }

    unlistenProgress = await listen<{
      id: string;
      url: string;
      filename: string;
      received: number;
      total: number;
      progress: number;
    }>('exodus-download-progress', (e) => {
      const p = e.payload;
      downloads.value = downloads.value.map((d) =>
        d.id === p.id
          ? {
              ...d,
              status: 'downloading',
              filename: p.filename,
              received: p.received,
              total: p.total,
              progress: p.progress,
            }
          : d,
      );
    });

    unlistenDone = await listen<{ id: string; path: string; filename: string }>(
      'exodus-download-done',
      (e) => {
        const p = e.payload;
        downloads.value = downloads.value.map((d) =>
          d.id === p.id
            ? { ...d, status: 'completed', progress: 100, path: p.path, filename: p.filename }
            : d,
        );
        options.onStatus?.(`Downloaded: ${p.filename}`);
      },
    );

    unlistenError = await listen<{ id: string; message: string }>('exodus-download-error', (e) => {
      const { id, message } = e.payload;
      downloads.value = downloads.value.map((d) =>
        d.id === id ? { ...d, status: 'failed' as const } : d,
      );
      options.onStatus?.(`Download error: ${message}`);
    });

    unlistenRequested = await listen<{ label: string; url: string }>(
      'exodus-download-requested',
      (e) => {
        void startDownload(e.payload.url);
      },
    );
  }

  function teardownListeners(): void {
    unlistenProgress?.();
    unlistenDone?.();
    unlistenError?.();
    unlistenRequested?.();
    unlistenProgress = undefined;
    unlistenDone = undefined;
    unlistenError = undefined;
    unlistenRequested = undefined;
  }

  onMounted(() => {
    void setupListeners();
  });

  onUnmounted(() => {
    teardownListeners();
  });

  return {
    downloads,
    showDownloadsPanel,
    activeDownloadsCount,
    openDownloadsPanel,
    closeDownloadsPanel,
    clearDownloads,
    startDownload,
    openDownloadsDir,
    openDownloadFile,
    revealDownloadFile,
    setupListeners,
    teardownListeners,
  };
}
