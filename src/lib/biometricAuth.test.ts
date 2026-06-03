/**
 * Exodus Browser — biometric authentication API tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  isBiometricAvailable,
  enableBiometric,
  disableBiometric,
  isBiometricEnabled,
  authenticateBiometric,
  setBiometricRequirePasswords,
  setBiometricRequireSensitive,
  setBiometricAutoLockTimeout,
  getBiometricSettings,
} from './biometricAuth';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('biometricAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('checks if biometric is available', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const available = await isBiometricAvailable();

    expect(available).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_biometric_available');
  });

  it('enables biometric authentication', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await enableBiometric();

    expect(invoke).toHaveBeenCalledWith('enable_biometric');
  });

  it('disables biometric authentication', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await disableBiometric();

    expect(invoke).toHaveBeenCalledWith('disable_biometric');
  });

  it('checks if biometric is enabled', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);

    const enabled = await isBiometricEnabled();

    expect(enabled).toBe(true);
    expect(invoke).toHaveBeenCalledWith('is_biometric_enabled');
  });

  it('authenticates with biometric', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResult = { success: true };
    vi.mocked(invoke).mockResolvedValue(mockResult);

    const result = await authenticateBiometric('Authenticate to access passwords');

    expect(result).toEqual(mockResult);
    expect(invoke).toHaveBeenCalledWith('authenticate_biometric', { reason: 'Authenticate to access passwords' });
  });

  it('returns error on failed biometric authentication', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResult = { success: false, error: 'Authentication failed' };
    vi.mocked(invoke).mockResolvedValue(mockResult);

    const result = await authenticateBiometric('Authenticate');

    expect(result.success).toBe(false);
    expect(result.error).toBe('Authentication failed');
  });

  it('sets biometric requirement for passwords', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setBiometricRequirePasswords(true);

    expect(invoke).toHaveBeenCalledWith('set_biometric_require_passwords', { require: true });
  });

  it('sets biometric requirement for sensitive data', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setBiometricRequireSensitive(false);

    expect(invoke).toHaveBeenCalledWith('set_biometric_require_sensitive', { require: false });
  });

  it('sets auto-lock timeout', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await setBiometricAutoLockTimeout(5);

    expect(invoke).toHaveBeenCalledWith('set_biometric_auto_lock_timeout', { timeoutMinutes: 5 });
  });

  it('gets biometric settings', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockSettings = {
      enabled: true,
      require_for_passwords: true,
      require_for_sensitive_data: false,
      auto_lock_timeout_minutes: 5,
    };
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const settings = await getBiometricSettings();

    expect(settings).toEqual(mockSettings);
    expect(invoke).toHaveBeenCalledWith('get_biometric_settings');
  });

  it('handles errors gracefully', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValue(new Error('Biometric not available'));

    await expect(isBiometricAvailable()).rejects.toThrow('Biometric not available');
  });
});
