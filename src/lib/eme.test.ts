/**
 * Exodus Browser — EME API Tests
 * 
 * Tests for the Encrypted Media Extensions API implementation
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  isTypeSupported,
  requestMediaKeySystemAccess,
  MediaKeys,
  MediaKeySession,
  setLicenseServerUrl,
  getLicenseServerUrl,
  acquireLicense,
  isDrmEnabled,
  enableDrm,
  disableDrm,
  WIDEVINE_KEY_SYSTEM,
  DrmSessionState,
} from './eme';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => true),
}));

describe('EME API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('isTypeSupported', () => {
    it('should support Widevine key system', () => {
      expect(isTypeSupported(WIDEVINE_KEY_SYSTEM)).toBe(true);
    });

    it('should not support unsupported key systems', () => {
      expect(isTypeSupported('com.fake.drm')).toBe(false);
    });

    it('should support cenc init data type', () => {
      expect(isTypeSupported(WIDEVINE_KEY_SYSTEM, 'cenc')).toBe(true);
    });

    it('should support keyids init data type', () => {
      expect(isTypeSupported(WIDEVINE_KEY_SYSTEM, 'keyids')).toBe(true);
    });

    it('should not support unsupported init data types', () => {
      expect(isTypeSupported(WIDEVINE_KEY_SYSTEM, 'webm')).toBe(false);
    });
  });

  describe('requestMediaKeySystemAccess', () => {
    it('should return MediaKeys for supported key system', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(true);

      const mediaKeys = await requestMediaKeySystemAccess(WIDEVINE_KEY_SYSTEM, []);
      expect(mediaKeys).toBeInstanceOf(MediaKeys);
    });

    it('should reject unsupported key system', async () => {
      await expect(
        requestMediaKeySystemAccess('com.fake.drm', [])
      ).rejects.toThrow('NotSupportedError');
    });

    it('should reject when CDM is not available', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(false);

      await expect(
        requestMediaKeySystemAccess(WIDEVINE_KEY_SYSTEM, [])
      ).rejects.toThrow('NotSupportedError');
    });
  });

  describe('MediaKeys', () => {
    it('should create MediaKeySession', () => {
      const mediaKeys = new MediaKeys(WIDEVINE_KEY_SYSTEM);
      const session = mediaKeys.createSession();
      expect(session).toBeInstanceOf(MediaKeySession);
    });

    it('should create session with type', () => {
      const mediaKeys = new MediaKeys(WIDEVINE_KEY_SYSTEM);
      const session = mediaKeys.createSession('temporary');
      expect(session).toBeInstanceOf(MediaKeySession);
    });

    it('should reject invalid session type', () => {
      const mediaKeys = new MediaKeys(WIDEVINE_KEY_SYSTEM);
      expect(() => mediaKeys.createSession('invalid' as any)).toThrow('NotSupportedError');
    });

    it('should set server certificate', async () => {
      const mediaKeys = new MediaKeys(WIDEVINE_KEY_SYSTEM);
      const result = await mediaKeys.setServerCertificate(new ArrayBuffer(0));
      expect(result).toBe(false);
    });
  });

  describe('MediaKeySession', () => {
    let session: MediaKeySession;

    beforeEach(() => {
      session = new MediaKeySession(WIDEVINE_KEY_SYSTEM);
    });

    it('should generate key request', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue({
        sessionId: 'test-session',
        initDataType: 'cenc',
        initData: [],
        keySystem: WIDEVINE_KEY_SYSTEM,
      });

      const initData = new ArrayBuffer(10);
      await session.generateRequest('cenc', initData);
      expect(session.sessionId).toBeDefined();
    });

    it('should reject generateRequest when closed', async () => {
      await session.close();
      const initData = new ArrayBuffer(10);
      await expect(
        session.generateRequest('cenc', initData)
      ).rejects.toThrow('InvalidStateError');
    });

    it('should update key', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      const response = new ArrayBuffer(10);
      await session.update(response);
      // Should not throw
    });

    it('should reject update when closed', async () => {
      await session.close();
      const response = new ArrayBuffer(10);
      await expect(session.update(response)).rejects.toThrow('InvalidStateError');
    });

    it('should close session', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      await session.close();
      // Should not throw
    });

    it('should close session only once', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      await session.close();
      await session.close();
      // Should not throw
    });

    it('should remove session', async () => {
      await session.remove();
      // Should not throw
    });

    it('should return null expiration', () => {
      expect(session.expiration).toBeNull();
    });

    it('should return key statuses', () => {
      const statuses = session.getKeyStatuses();
      expect(statuses).toBeInstanceOf(Map);
    });

    it('should dispatch message event', (done) => {
      session.onmessage = (event) => {
        expect(event.messageType).toBe('license-request');
        done();
      };

      // Simulate message event
      session.dispatchEvent(
        new CustomEvent('message', {
          detail: {
            messageType: 'license-request',
            message: new ArrayBuffer(0),
          },
        })
      );
    });

    it('should dispatch keystatuseschange event', (done) => {
      session.onkeystatuseschange = () => {
        done();
      };

      session.dispatchEvent(new Event('keystatuseschange'));
    });
  });

  describe('License Server', () => {
    it('should set license server URL', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      await setLicenseServerUrl('https://example.com/license');
      expect(invoke).toHaveBeenCalledWith('drm_set_license_server_url', {
        url: 'https://example.com/license',
      });
    });

    it('should get license server URL', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue('https://example.com/license');

      const url = await getLicenseServerUrl();
      expect(url).toBe('https://example.com/license');
    });

    it('should acquire license', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue([1, 2, 3, 4]);

      const sessionId = 'test-session';
      const challenge = new Uint8Array([1, 2, 3]).buffer;
      const license = await acquireLicense(sessionId, challenge);

      expect(license).toBeInstanceOf(ArrayBuffer);
      expect(license.byteLength).toBe(4);
    });
  });

  describe('DRM Control', () => {
    it('should check if DRM is enabled', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(true);

      const enabled = await isDrmEnabled();
      expect(enabled).toBe(true);
    });

    it('should enable DRM', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      await enableDrm();
      expect(invoke).toHaveBeenCalledWith('drm_enable');
    });

    it('should disable DRM', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue(undefined);

      await disableDrm();
      expect(invoke).toHaveBeenCalledWith('drm_disable');
    });
  });

  describe('Non-Tauri Environment', () => {
    it('should handle non-Tauri environment gracefully', async () => {
      const { isTauri } = await import('@tauri-apps/api/core');
      vi.mocked(isTauri).mockReturnValue(false);

      const enabled = await isDrmEnabled();
      expect(enabled).toBe(false);
    });

    it('should skip license server URL set in non-Tauri', async () => {
      const { isTauri } = await import('@tauri-apps/api/core');
      vi.mocked(isTauri).mockReturnValue(false);

      await setLicenseServerUrl('https://example.com/license');
      // Should not throw
    });

    it('should return null for license server URL in non-Tauri', async () => {
      const { isTauri } = await import('@tauri-apps/api/core');
      vi.mocked(isTauri).mockReturnValue(false);

      const url = await getLicenseServerUrl();
      expect(url).toBeNull();
    });

    it('should return empty license in non-Tauri', async () => {
      const { isTauri } = await import('@tauri-apps/api/core');
      vi.mocked(isTauri).mockReturnValue(false);

      const license = await acquireLicense('test-session', new ArrayBuffer(0));
      expect(license.byteLength).toBe(0);
    });
  });

  describe('Constants', () => {
    it('should have correct Widevine key system', () => {
      expect(WIDEVINE_KEY_SYSTEM).toBe('com.widevine.alpha');
    });

    it('should have correct session state values', () => {
      expect(DrmSessionState.Idle).toBe('idle');
      expect(DrmSessionState.Created).toBe('created');
      expect(DrmSessionState.KeyReady).toBe('keyReady');
      expect(DrmSessionState.Closed).toBe('closed');
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty init data', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue({
        sessionId: 'test-session',
        initDataType: 'cenc',
        initData: [],
        keySystem: WIDEVINE_KEY_SYSTEM,
      });

      const session = new MediaKeySession(WIDEVINE_KEY_SYSTEM);
      await session.generateRequest('cenc', new ArrayBuffer(0));
      expect(session.sessionId).toBeDefined();
    });

    it('should handle large init data', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue({
        sessionId: 'test-session',
        initDataType: 'cenc',
        initData: [],
        keySystem: WIDEVINE_KEY_SYSTEM,
      });

      const session = new MediaKeySession(WIDEVINE_KEY_SYSTEM);
      const largeData = new ArrayBuffer(1024 * 1024); // 1MB
      await session.generateRequest('cenc', largeData);
      expect(session.sessionId).toBeDefined();
    });

    it('should handle multiple sessions', async () => {
      const { invoke } = await import('@tauri-apps/api/core');
      vi.mocked(invoke).mockResolvedValue({
        sessionId: 'test-session',
        initDataType: 'cenc',
        initData: [],
        keySystem: WIDEVINE_KEY_SYSTEM,
      });

      const mediaKeys = new MediaKeys(WIDEVINE_KEY_SYSTEM);
      const session1 = mediaKeys.createSession();
      const session2 = mediaKeys.createSession();

      await session1.generateRequest('cenc', new ArrayBuffer(10));
      await session2.generateRequest('cenc', new ArrayBuffer(10));

      expect(session1.sessionId).toBeDefined();
      expect(session2.sessionId).toBeDefined();
    });
  });
});
