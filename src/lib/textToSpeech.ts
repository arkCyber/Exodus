/**
 * Text-to-Speech API for Exodus Browser
 * Provides speech synthesis for web content
 */

import { invoke } from '@tauri-apps/api/core';

export interface TtsVoice {
  name: string;
  language: string;
  gender: string;
}

export interface TtsResult {
  success: boolean;
  error?: string;
}

export interface TtsSettings {
  enabled: boolean;
  voice: string;
  rate: number;
  pitch: number;
  volume: number;
}

/**
 * Check if TTS is available
 */
export async function isTtsAvailable(): Promise<boolean> {
  return invoke('is_tts_available');
}

/**
 * Enable TTS
 */
export async function enableTts(): Promise<void> {
  return invoke('enable_tts');
}

/**
 * Disable TTS
 */
export async function disableTts(): Promise<void> {
  return invoke('disable_tts');
}

/**
 * Check if TTS is enabled
 */
export async function isTtsEnabled(): Promise<boolean> {
  return invoke('is_tts_enabled');
}

/**
 * Speak text
 */
export async function ttsSpeak(text: string): Promise<TtsResult> {
  return invoke('tts_speak', { text });
}

/**
 * Stop speaking
 */
export async function ttsStop(): Promise<TtsResult> {
  return invoke('tts_stop');
}

/**
 * Get available voices
 */
export async function ttsGetVoices(): Promise<TtsVoice[]> {
  return invoke('tts_get_voices');
}

/**
 * Set voice
 */
export async function ttsSetVoice(voice: string): Promise<void> {
  return invoke('tts_set_voice', { voice });
}

/**
 * Set rate
 */
export async function ttsSetRate(rate: number): Promise<void> {
  return invoke('tts_set_rate', { rate });
}

/**
 * Set pitch
 */
export async function ttsSetPitch(pitch: number): Promise<void> {
  return invoke('tts_set_pitch', { pitch });
}

/**
 * Set volume
 */
export async function ttsSetVolume(volume: number): Promise<void> {
  return invoke('tts_set_volume', { volume });
}

/**
 * Get TTS settings
 */
export async function ttsGetSettings(): Promise<TtsSettings> {
  return invoke('tts_get_settings');
}
