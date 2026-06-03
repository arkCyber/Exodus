/**
 * extensionToolbarIcon — toolbar icon URL helpers.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import {
  extensionIconLetter,
  extensionIconUrlCandidates,
  probeExtensionIconUrl,
} from './extensionToolbarIcon';
import type { ExtensionInfo } from '@/lib/extensions/types';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  convertFileSrc: (path: string) => `asset://${path}`,
}));

describe('extensionToolbarIcon', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('builds icon letter fallback', () => {
    expect(extensionIconLetter('hello')).toBe('H');
    expect(extensionIconLetter('')).toBe('?');
  });

  it('builds candidate icon URLs from extension path', () => {
    const ext = {
      id: 'ext-1',
      name: 'Hello',
      path: '/tmp/ext-hello',
      enabled: true,
    } as ExtensionInfo;
    const urls = extensionIconUrlCandidates(ext);
    expect(urls[0]).toContain('icons/icon16.png');
    expect(urls.some((u) => u.includes('/tmp/ext-hello/'))).toBe(true);
  });

  it('probes image load success and failure', async () => {
    const original = global.Image;
    class MockImage {
      onload: (() => void) | null = null;
      onerror: (() => void) | null = null;
      set src(_value: string) {
        this.onload?.();
      }
    }
    global.Image = MockImage as unknown as typeof Image;
    await expect(probeExtensionIconUrl('asset://icon.png')).resolves.toBe(true);
    global.Image = original;
  });
});
