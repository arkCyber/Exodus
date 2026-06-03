/**
 * Voice Search API for Exodus Browser
 * Provides speech recognition for search queries
 */

import { invoke } from '@tauri-apps/api/core';

export interface VoiceSearchResult {
  success: boolean;
  text?: string;
  error?: string;
}

export interface VoiceSearchSettings {
  enabled: boolean;
  language: string;
  auto_submit: boolean;
}

/**
 * Check if voice search is available
 */
export async function isVoiceSearchAvailable(): Promise<boolean> {
  return invoke('is_voice_search_available');
}

/**
 * Enable voice search
 */
export async function enableVoiceSearch(): Promise<void> {
  return invoke('enable_voice_search');
}

/**
 * Disable voice search
 */
export async function disableVoiceSearch(): Promise<void> {
  return invoke('disable_voice_search');
}

/**
 * Check if voice search is enabled
 */
export async function isVoiceSearchEnabled(): Promise<boolean> {
  return invoke('is_voice_search_enabled');
}

/**
 * Start voice recognition
 */
export async function startVoiceRecognition(): Promise<VoiceSearchResult> {
  return invoke('start_voice_recognition');
}

/**
 * Set language
 */
export async function setVoiceSearchLanguage(language: string): Promise<void> {
  return invoke('set_voice_search_language', { language });
}

/**
 * Set auto-submit
 */
export async function setVoiceSearchAutoSubmit(autoSubmit: boolean): Promise<void> {
  return invoke('set_voice_search_auto_submit', { autoSubmit });
}

/**
 * Get voice search settings
 */
export async function getVoiceSearchSettings(): Promise<VoiceSearchSettings> {
  return invoke('get_voice_search_settings');
}
