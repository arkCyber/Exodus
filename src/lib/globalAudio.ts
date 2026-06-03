/**
 * Global Audio Control API for Exodus Browser
 * Provides global audio control for all tabs
 */

import { invoke } from '@tauri-apps/api/core';

export interface TabAudioState {
  label: string;
  muted: boolean;
  volume: number;
  is_playing: boolean;
}

export interface GlobalAudioSettings {
  global_muted: boolean;
  global_volume: number;
  show_volume_indicator: boolean;
}

/**
 * Mute all tabs
 */
export async function muteAllTabs(): Promise<void> {
  return invoke('mute_all_tabs');
}

/**
 * Unmute all tabs
 */
export async function unmuteAllTabs(): Promise<void> {
  return invoke('unmute_all_tabs');
}

/**
 * Toggle global mute
 */
export async function toggleGlobalMute(): Promise<void> {
  return invoke('toggle_global_mute');
}

/**
 * Check if globally muted
 */
export async function isGloballyMuted(): Promise<boolean> {
  return invoke('is_globally_muted');
}

/**
 * Set global volume
 */
export async function setGlobalVolume(volume: number): Promise<void> {
  return invoke('set_global_volume', { volume });
}

/**
 * Get global volume
 */
export async function getGlobalVolume(): Promise<number> {
  return invoke('get_global_volume');
}

/**
 * Register a tab
 */
export async function registerAudioTab(label: string): Promise<void> {
  return invoke('register_audio_tab', { label });
}

/**
 * Unregister a tab
 */
export async function unregisterAudioTab(label: string): Promise<void> {
  return invoke('unregister_audio_tab', { label });
}

/**
 * Set tab mute
 */
export async function setTabMute(label: string, muted: boolean): Promise<void> {
  return invoke('set_global_tab_mute', { label, muted });
}

/**
 * Set tab volume
 */
export async function setTabVolume(label: string, volume: number): Promise<void> {
  return invoke('set_tab_volume', { label, volume });
}

/**
 * Update tab playing state
 */
export async function updateTabPlaying(label: string, isPlaying: boolean): Promise<void> {
  return invoke('update_tab_playing', { label, isPlaying });
}

/**
 * Get all tab states
 */
export async function getAllAudioTabs(): Promise<TabAudioState[]> {
  return invoke('get_all_audio_tabs');
}

/**
 * Get playing tabs count
 */
export async function getPlayingTabsCount(): Promise<number> {
  return invoke('get_playing_tabs_count');
}

/**
 * Show/hide volume indicator
 */
export async function showVolumeIndicator(show: boolean): Promise<void> {
  return invoke('show_volume_indicator', { show });
}

/**
 * Get global audio settings
 */
export async function getGlobalAudioSettings(): Promise<GlobalAudioSettings> {
  return invoke('get_global_audio_settings');
}
