import { describe, it, expect, vi, beforeEach } from 'vitest';
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock('$lib/downloadsPersist', () => ({
  loadPersistedDownloads: vi.fn(async () => []),
}));

import { invoke } from '@tauri-apps/api/core';
import { useBrowserDownloads } from './useBrowserDownloads';

describe('useBrowserDownloads', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('tracks active download count', async () => {
    const { downloads, activeDownloadsCount, setupListeners, teardownListeners } =
      useBrowserDownloads();
    await setupListeners();
    downloads.value = [
      {
        id: 'a',
        url: 'https://example.com/a.zip',
        filename: 'a.zip',
        status: 'downloading',
        progress: 50,
        received: 50,
        total: 100,
      },
      {
        id: 'b',
        url: 'https://example.com/b.zip',
        filename: 'b.zip',
        status: 'completed',
        progress: 100,
        received: 100,
        total: 100,
      },
    ];
    expect(activeDownloadsCount.value).toBe(1);
    teardownListeners();
  });

  it('starts download via invoke', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const { startDownload, downloads, setupListeners, teardownListeners } = useBrowserDownloads();
    await setupListeners();
    await startDownload('https://example.com/file.zip', 'file.zip');
    expect(downloads.value.length).toBe(1);
    expect(invoke).toHaveBeenCalledWith('download_url', expect.objectContaining({ url: 'https://example.com/file.zip' }));
    teardownListeners();
  });
});
