/**
 * Exodus Browser — Text-to-Speech Client
 * Provides interface to TTS service for speech synthesis
 */

import { invoke } from '@tauri-apps/api/core';

/** TTS voice */
export type TtsVoice = {
  name: string;
  language: string;
  gender: string;
};

/** TTS result */
export type TtsResult = {
  success: boolean;
  error?: string;
};

/** TTS settings */
export type TtsSettings = {
  enabled: boolean;
  voice: string;
  rate: number;      // 0.1 to 10.0
  pitch: number;     // 0.0 to 2.0
  volume: number;    // 0.0 to 1.0
};

/** Speak text */
export async function speak(text: string): Promise<TtsResult> {
  return invoke<TtsResult>('tts_speak', { text });
}

/** Stop speaking */
export async function stopSpeaking(): Promise<TtsResult> {
  return invoke<TtsResult>('tts_stop');
}

/** Enable TTS */
export async function enableTts(): Promise<void> {
  return invoke<void>('enable_tts');
}

/** Disable TTS */
export async function disableTts(): Promise<void> {
  return invoke<void>('disable_tts');
}

/** Check if TTS is enabled */
export async function isTtsEnabled(): Promise<boolean> {
  return invoke<boolean>('is_tts_enabled');
}

/** Check if TTS is available */
export async function isTtsAvailable(): Promise<boolean> {
  return invoke<boolean>('is_tts_available');
}

/** Get TTS settings */
export async function getTtsSettings(): Promise<TtsSettings> {
  return invoke<TtsSettings>('get_tts_settings');
}

/** Update TTS settings */
export async function updateTtsSettings(settings: TtsSettings): Promise<void> {
  return invoke<void>('update_tts_settings', { settings });
}

/** Get available voices */
export async function getAvailableVoices(): Promise<TtsVoice[]> {
  return invoke<TtsVoice[]>('get_available_voices');
}
