/**
 * Biometric Authentication API for Exodus Browser
 * Provides Face ID, Touch ID, and Windows Hello support
 */

import { invoke } from '@tauri-apps/api/core';

export interface BiometricResult {
  success: boolean;
  error?: string;
}

export interface BiometricSettings {
  enabled: boolean;
  require_for_passwords: boolean;
  require_for_sensitive_data: boolean;
  auto_lock_timeout_minutes: number;
}

/**
 * Check if biometric authentication is available
 */
export async function isBiometricAvailable(): Promise<boolean> {
  return invoke('is_biometric_available');
}

/**
 * Enable biometric authentication
 */
export async function enableBiometric(): Promise<void> {
  return invoke('enable_biometric');
}

/**
 * Disable biometric authentication
 */
export async function disableBiometric(): Promise<void> {
  return invoke('disable_biometric');
}

/**
 * Check if biometric authentication is enabled
 */
export async function isBiometricEnabled(): Promise<boolean> {
  return invoke('is_biometric_enabled');
}

/**
 * Request biometric authentication
 */
export async function authenticateBiometric(reason: string): Promise<BiometricResult> {
  return invoke('authenticate_biometric', { reason });
}

/**
 * Set requirement for passwords
 */
export async function setBiometricRequirePasswords(require: boolean): Promise<void> {
  return invoke('set_biometric_require_passwords', { require });
}

/**
 * Set requirement for sensitive data
 */
export async function setBiometricRequireSensitive(require: boolean): Promise<void> {
  return invoke('set_biometric_require_sensitive', { require });
}

/**
 * Set auto-lock timeout
 */
export async function setBiometricAutoLockTimeout(timeoutMinutes: number): Promise<void> {
  return invoke('set_biometric_auto_lock_timeout', { timeoutMinutes });
}

/**
 * Get biometric authentication settings
 */
export async function getBiometricSettings(): Promise<BiometricSettings> {
  return invoke('get_biometric_settings');
}
