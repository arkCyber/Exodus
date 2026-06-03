/**
 * Voice Control API for Exodus Browser
 * Provides voice command navigation and control
 */

import { invoke } from '@tauri-apps/api/core';

export interface VoiceCommand {
  id: string;
  phrase: string;
  action: string;
  enabled: boolean;
}

export interface VoiceControlSettings {
  enabled: boolean;
  continuous_listening: boolean;
  wake_word: string;
  confidence_threshold: number;
}

/**
 * Start voice listening
 */
export async function startVoiceListening(): Promise<void> {
  return invoke('start_voice_listening');
}

/**
 * Stop voice listening
 */
export async function stopVoiceListening(): Promise<void> {
  return invoke('stop_voice_listening');
}

/**
 * Check if voice is listening
 */
export async function isVoiceListening(): Promise<boolean> {
  return invoke('is_voice_listening');
}

/**
 * Process voice command
 */
export async function processVoiceCommand(text: string): Promise<VoiceCommand | null> {
  return invoke('process_voice_command', { text });
}

/**
 * Add voice command
 */
export async function addVoiceCommand(phrase: string, action: string): Promise<string> {
  return invoke('add_voice_command', { phrase, action });
}

/**
 * Remove voice command
 */
export async function removeVoiceCommand(id: string): Promise<void> {
  return invoke('remove_voice_command', { id });
}

/**
 * Enable voice command
 */
export async function enableVoiceCommand(id: string): Promise<void> {
  return invoke('enable_voice_command', { id });
}

/**
 * Disable voice command
 */
export async function disableVoiceCommand(id: string): Promise<void> {
  return invoke('disable_voice_command', { id });
}

/**
 * Get voice commands
 */
export async function getVoiceCommands(): Promise<VoiceCommand[]> {
  return invoke('get_voice_commands');
}

/**
 * Enable voice control
 */
export async function enableVoiceControl(): Promise<void> {
  return invoke('enable_voice_control');
}

/**
 * Disable voice control
 */
export async function disableVoiceControl(): Promise<void> {
  return invoke('disable_voice_control');
}

/**
 * Check if voice control is enabled
 */
export async function isVoiceControlEnabled(): Promise<boolean> {
  return invoke('is_voice_control_enabled');
}

/**
 * Set wake word
 */
export async function setVoiceControlWakeWord(wakeWord: string): Promise<void> {
  return invoke('set_voice_control_wake_word', { wakeWord });
}

/**
 * Get voice control settings
 */
export async function getVoiceControlSettings(): Promise<VoiceControlSettings> {
  return invoke('get_voice_control_settings');
}
