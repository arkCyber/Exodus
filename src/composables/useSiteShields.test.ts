/**
 * Exodus Browser — useSiteShields unit tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useSiteShields } from './useSiteShields';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: () => true,
}));

vi.mock('$lib/browserIntegrations', () => ({
  loadTrackingProtectionSettings: vi.fn(async () => ({ enabled: true })),
}));

describe('useSiteShields', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('shieldsEnabled reflects tracking and site override', async () => {
    const shields = useSiteShields({ onStatus: vi.fn() });
    await shields.loadTrackingProtection();
    shields.siteAllowTrackers.value = false;
    expect(shields.shieldsEnabled()).toBe(true);
    shields.siteAllowTrackers.value = true;
    expect(shields.shieldsEnabled()).toBe(false);
  });

  it('toggleSiteShieldAllowTrackers calls invoke', async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const shields = useSiteShields({ onStatus: vi.fn(), reloadActiveTab: vi.fn() });
    await shields.toggleSiteShieldAllowTrackers('https://example.com/path');
    expect(invoke).toHaveBeenCalledWith('set_site_shield_override', {
      host: 'example.com',
      allowTrackers: true,
    });
  });
});
