/**
 * Unit tests for privacy stats helpers.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('$lib/tauri', () => ({
  canInvokeTauri: vi.fn(() => true),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('fetchPrivacyStats', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns null when Tauri IPC is not ready', async () => {
    const { canInvokeTauri } = await import('$lib/tauri');
    vi.mocked(canInvokeTauri).mockReturnValue(false);
    const { fetchPrivacyStats } = await import('./privacyStats');
    await expect(fetchPrivacyStats()).resolves.toBeNull();
    const { invoke } = await import('@tauri-apps/api/core');
    expect(invoke).not.toHaveBeenCalled();
  });

  it('invokes get_privacy_stats when IPC is ready', async () => {
    const { canInvokeTauri } = await import('$lib/tauri');
    vi.mocked(canInvokeTauri).mockReturnValue(true);
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue({
      trackers_blocked: 3,
      malicious_sites_blocked: 0,
      cookies_blocked: 1,
      fingerprinting_blocked: 2,
    });
    const { fetchPrivacyStats } = await import('./privacyStats');
    const stats = await fetchPrivacyStats();
    expect(invoke).toHaveBeenCalledWith('get_privacy_stats');
    expect(stats?.trackers_blocked).toBe(3);
  });
});
