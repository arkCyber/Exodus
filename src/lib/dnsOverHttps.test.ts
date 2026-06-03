/**
 * Unit tests for DNS over HTTPS (DoH) API.
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

const invokeMock = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
  isTauri: () => true,
}));

import {
  enableDoh,
  disableDoh,
  isDohEnabled,
  setDohProvider,
  getActiveDohProvider,
  addDohProvider,
  removeDohProvider,
  setDohFallback,
  getDohProviders,
  getDohSettings,
} from './dnsOverHttps';

describe('dnsOverHttps', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  describe('enableDoh', () => {
    it('calls enable_doh command', async () => {
      await enableDoh();
      expect(invokeMock).toHaveBeenCalledWith('enable_doh');
    });
  });

  describe('disableDoh', () => {
    it('calls disable_doh command', async () => {
      await disableDoh();
      expect(invokeMock).toHaveBeenCalledWith('disable_doh');
    });
  });

  describe('isDohEnabled', () => {
    it('returns true when DoH is enabled', async () => {
      invokeMock.mockResolvedValue(true);
      const result = await isDohEnabled();
      expect(result).toBe(true);
      expect(invokeMock).toHaveBeenCalledWith('is_doh_enabled');
    });

    it('returns false when DoH is disabled', async () => {
      invokeMock.mockResolvedValue(false);
      const result = await isDohEnabled();
      expect(result).toBe(false);
    });
  });

  describe('setDohProvider', () => {
    it('calls set_doh_provider with provider name', async () => {
      await setDohProvider('Cloudflare');
      expect(invokeMock).toHaveBeenCalledWith('set_doh_provider', { providerName: 'Cloudflare' });
    });
  });

  describe('getActiveDohProvider', () => {
    it('returns active provider', async () => {
      const provider = { name: 'Cloudflare', url: 'https://dns.cloudflare.com/dns-query', enabled: true };
      invokeMock.mockResolvedValue(provider);
      const result = await getActiveDohProvider();
      expect(result).toEqual(provider);
      expect(invokeMock).toHaveBeenCalledWith('get_active_doh_provider');
    });

    it('returns null when no active provider', async () => {
      invokeMock.mockResolvedValue(null);
      const result = await getActiveDohProvider();
      expect(result).toBeNull();
    });
  });

  describe('addDohProvider', () => {
    it('calls add_doh_provider with name and url', async () => {
      await addDohProvider('Custom', 'https://custom-dns.example.com/dns-query');
      expect(invokeMock).toHaveBeenCalledWith('add_doh_provider', {
        name: 'Custom',
        url: 'https://custom-dns.example.com/dns-query',
      });
    });
  });

  describe('removeDohProvider', () => {
    it('calls remove_doh_provider with provider name', async () => {
      await removeDohProvider('Custom');
      expect(invokeMock).toHaveBeenCalledWith('remove_doh_provider', { name: 'Custom' });
    });
  });

  describe('setDohFallback', () => {
    it('calls set_doh_fallback with fallback flag', async () => {
      await setDohFallback(true);
      expect(invokeMock).toHaveBeenCalledWith('set_doh_fallback', { fallback: true });
    });

    it('calls set_doh_fallback with false', async () => {
      await setDohFallback(false);
      expect(invokeMock).toHaveBeenCalledWith('set_doh_fallback', { fallback: false });
    });
  });

  describe('getDohProviders', () => {
    it('returns list of providers', async () => {
      const providers = [
        { name: 'Cloudflare', url: 'https://dns.cloudflare.com/dns-query', enabled: true },
        { name: 'Google', url: 'https://dns.google/dns-query', enabled: false },
      ];
      invokeMock.mockResolvedValue(providers);
      const result = await getDohProviders();
      expect(result).toEqual(providers);
      expect(invokeMock).toHaveBeenCalledWith('get_doh_providers');
    });

    it('returns empty array when no providers', async () => {
      invokeMock.mockResolvedValue([]);
      const result = await getDohProviders();
      expect(result).toEqual([]);
    });
  });

  describe('getDohSettings', () => {
    it('returns DoH settings', async () => {
      const settings = {
        enabled: true,
        providers: [
          { name: 'Cloudflare', url: 'https://dns.cloudflare.com/dns-query', enabled: true },
        ],
        fallback_to_system: false,
      };
      invokeMock.mockResolvedValue(settings);
      const result = await getDohSettings();
      expect(result).toEqual(settings);
      expect(invokeMock).toHaveBeenCalledWith('get_doh_settings');
    });
  });
});
