/**
 * Unit tests for persisted download loader.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';
import { loadPersistedDownloads } from './downloadsPersist';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('loadPersistedDownloads', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('maps backend rows to DownloadRecord', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue([
      {
        id: 'a1',
        url: 'https://example.com/f.bin',
        filename: 'f.bin',
        path: '/tmp/f.bin',
        status: 'completed',
        progress: 100,
        received: 10,
        total: 10,
      },
    ]);
    const rows = await loadPersistedDownloads();
    expect(rows).toHaveLength(1);
    expect(rows[0].status).toBe('completed');
    expect(rows[0].path).toBe('/tmp/f.bin');
  });
});
