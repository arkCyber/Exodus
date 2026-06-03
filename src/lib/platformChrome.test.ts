/**
 * Unit tests — platformChrome helpers.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: vi.fn(() => false),
}));

describe('platformChrome', () => {
  beforeEach(() => {
    document.documentElement.className = '';
  });

  it('does not add macOS class when not in Tauri', async () => {
    const { isTauri } = await import('@tauri-apps/api/core');
    vi.mocked(isTauri).mockReturnValue(false);
    const { applyPlatformChromeClasses, isMacTauriOverlayTitlebar } = await import(
      './platformChrome'
    );
    expect(isMacTauriOverlayTitlebar()).toBe(false);
    applyPlatformChromeClasses();
    expect(document.documentElement.classList.contains('exodus-macos-overlay-titlebar')).toBe(
      false,
    );
  });

  it('adds macOS overlay class on Mac Tauri', async () => {
    const { isTauri } = await import('@tauri-apps/api/core');
    vi.mocked(isTauri).mockReturnValue(true);
    vi.stubGlobal('navigator', { userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X)' });
    const { applyPlatformChromeClasses, isMacTauriOverlayTitlebar } = await import(
      './platformChrome'
    );
    expect(isMacTauriOverlayTitlebar()).toBe(true);
    applyPlatformChromeClasses();
    expect(document.documentElement.classList.contains('exodus-macos-overlay-titlebar')).toBe(
      true,
    );
  });
});
