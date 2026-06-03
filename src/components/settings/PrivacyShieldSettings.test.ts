/**
 * Exodus Browser — PrivacyShieldSettings tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import PrivacyShieldSettings from './PrivacyShieldSettings.vue';

vi.mock('$lib/browserIntegrations', () => ({
  loadSafeBrowsingSettings: vi.fn(async () => ({
    enabled: true,
    block_malware: true,
    block_phishing: true,
    block_unwanted_software: true,
    show_warnings: true,
    allow_proceed: true,
    list_url: null,
  })),
  loadTrackingProtectionSettings: vi.fn(async () => ({
    enabled: true,
    block_advertising: true,
    block_analytics: true,
    block_social: false,
    block_fingerprinting: true,
    block_cryptomining: true,
    block_tracking: true,
    subscription_url: null,
    subscription_refresh_hours: 24,
  })),
  loadEncryptedSyncSettings: vi.fn(async () => ({
    enabled: false,
    has_passphrase: false,
    last_sync_at: 0,
    sync_server_url: null,
    sync_token: null,
    device_id: null,
  })),
  saveSafeBrowsingSettings: vi.fn(),
  saveTrackingProtectionSettings: vi.fn(),
  refreshSafeBrowsingList: vi.fn(async () => 1),
  setTrackingSubscription: vi.fn(),
  setEncryptedSyncServer: vi.fn(),
  setEncryptedSyncPassphrase: vi.fn(),
  storeEncryptedBookmarkVault: vi.fn(),
  uploadEncryptedVault: vi.fn(),
  downloadEncryptedVault: vi.fn(),
}));

vi.mock('$lib/siteShields', () => ({
  refreshTrackerBlocklist: vi.fn(async () => 100),
}));

describe('PrivacyShieldSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('loads and shows tracking protection section', async () => {
    const wrapper = mount(PrivacyShieldSettings);
    await flushPromises();
    expect(wrapper.text()).toContain('Tracking protection');
    expect(wrapper.text()).toContain('Safe Browsing');
  });
});
