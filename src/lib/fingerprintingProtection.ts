/**
 * Fingerprinting Protection API for Exodus Browser
 * Protects against browser fingerprinting techniques
 */

import { invoke } from '@tauri-apps/api/core';

export interface FingerprintingSettings {
  enabled: boolean;
  block_canvas_fingerprinting: boolean;
  block_webgl_fingerprinting: boolean;
  block_audio_fingerprinting: boolean;
  block_font_fingerprinting: boolean;
  block_screen_fingerprinting: boolean;
  block_timezone_fingerprinting: boolean;
  block_language_fingerprinting: boolean;
  randomize_user_agent: boolean;
}

/**
 * Enable fingerprinting protection
 */
export async function enableFingerprintingProtection(): Promise<void> {
  return invoke('enable_fingerprinting_protection');
}

/**
 * Disable fingerprinting protection
 */
export async function disableFingerprintingProtection(): Promise<void> {
  return invoke('disable_fingerprinting_protection');
}

/**
 * Check if fingerprinting protection is enabled
 */
export async function isFingerprintingProtectionEnabled(): Promise<boolean> {
  return invoke('is_fingerprinting_protection_enabled');
}

/**
 * Set canvas fingerprinting protection
 */
export async function setCanvasFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_canvas_fingerprinting_protection', { enabled });
}

/**
 * Set WebGL fingerprinting protection
 */
export async function setWebglFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_webgl_fingerprinting_protection', { enabled });
}

/**
 * Set audio fingerprinting protection
 */
export async function setAudioFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_audio_fingerprinting_protection', { enabled });
}

/**
 * Set font fingerprinting protection
 */
export async function setFontFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_font_fingerprinting_protection', { enabled });
}

/**
 * Set screen fingerprinting protection
 */
export async function setScreenFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_screen_fingerprinting_protection', { enabled });
}

/**
 * Set timezone fingerprinting protection
 */
export async function setTimezoneFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_timezone_fingerprinting_protection', { enabled });
}

/**
 * Set language fingerprinting protection
 */
export async function setLanguageFingerprintingProtection(enabled: boolean): Promise<void> {
  return invoke('set_language_fingerprinting_protection', { enabled });
}

/**
 * Set user agent randomization
 */
export async function setRandomizeUserAgent(enabled: boolean): Promise<void> {
  return invoke('set_randomize_user_agent', { enabled });
}

/**
 * Get fingerprinting protection settings
 */
export async function getFingerprintingProtectionSettings(): Promise<FingerprintingSettings> {
  return invoke('get_fingerprinting_protection_settings');
}
