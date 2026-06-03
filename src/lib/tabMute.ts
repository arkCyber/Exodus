/**
 * Tab Mute API for Exodus Browser
 * Allows muting/unmuting audio in individual tabs
 */

import { invoke } from '@tauri-apps/api/core';

export interface TabMuteState {
  label: string;
  is_muted: boolean;
  audio_playing: boolean;
}

/**
 * Register a tab for mute tracking
 */
export async function registerMuteTab(label: string): Promise<void> {
  return invoke('register_mute_tab', { label });
}

/**
 * Unregister a tab from mute tracking
 */
export async function unregisterMuteTab(label: string): Promise<void> {
  return invoke('unregister_mute_tab', { label });
}

/**
 * Set mute state for a tab
 */
export async function setTabMute(label: string, isMuted: boolean): Promise<void> {
  return invoke('set_tab_mute', { label, isMuted });
}

/**
 * Get mute state for a tab
 */
export async function getTabMuteState(label: string): Promise<boolean | null> {
  return invoke('get_tab_mute_state', { label });
}

/**
 * Set audio playing state for a tab
 */
export async function setTabAudioPlaying(label: string, audioPlaying: boolean): Promise<void> {
  return invoke('set_tab_audio_playing', { label, audioPlaying });
}

/**
 * Get all tabs with audio playing
 */
export async function getAudioPlayingTabs(): Promise<TabMuteState[]> {
  return invoke('get_audio_playing_tabs');
}

/**
 * Get all tab mute states
 */
export async function getAllTabMuteStates(): Promise<TabMuteState[]> {
  return invoke('get_all_tab_mute_states');
}
