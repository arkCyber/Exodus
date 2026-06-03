/**
 * Exodus Browser — tests for extension background host bootstrap.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

vi.mock('@tauri-apps/api/dpi', () => ({
  LogicalPosition: class {
    constructor(
      public x: number,
      public y: number,
    ) {}
  },
  LogicalSize: class {
    constructor(
      public w: number,
      public h: number,
    ) {}
  },
}));

vi.mock('@tauri-apps/api/webview', () => ({
  Webview: {
    getByLabel: vi.fn(async () => null),
  },
}));

import { ensureExtensionBackgrounds } from './backgroundHosts';

describe('ensureExtensionBackgrounds', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue([]);
  });

  it('loads specs and skips create when empty', async () => {
    const el = {
      getBoundingClientRect: () => ({ left: 0, top: 0, width: 800, height: 600 }),
    } as HTMLElement;
    await ensureExtensionBackgrounds(el);
    expect(invokeMock).toHaveBeenCalledWith('extension_background_specs');
  });
});
