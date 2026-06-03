/**
 * Exodus Browser — load persisted downloads from the Rust download manager.
 */

import { invoke } from '@tauri-apps/api/core';
import type { DownloadRecord } from '$lib/browserTypes';

type PersistedDownloadRow = {
  id: string;
  url: string;
  filename: string;
  path?: string;
  status: DownloadRecord['status'];
  progress: number;
  received: number;
  total: number;
};

/**
 * Load download history from disk (survives app restart).
 */
export async function loadPersistedDownloads(): Promise<DownloadRecord[]> {
  try {
    const rows = await invoke<PersistedDownloadRow[]>('list_persisted_downloads');
    return rows.map((row) => ({
      id: row.id,
      url: row.url,
      filename: row.filename,
      path: row.path,
      status: row.status,
      progress: row.progress ?? 0,
      received: row.received ?? 0,
      total: row.total ?? 0,
    }));
  } catch (error) {
    console.error('loadPersistedDownloads failed:', error);
    return [];
  }
}
