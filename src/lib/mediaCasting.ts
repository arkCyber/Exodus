/**
 * Media Casting API for Exodus Browser
 * Provides casting support for Chromecast, AirPlay, and other devices
 */

import { invoke } from '@tauri-apps/api/core';

export type CastDeviceType = 'chromecast' | 'airplay' | 'dlna' | 'webrtc';
export type CastState = 'idle' | 'connecting' | 'connected' | 'casting' | 'disconnected' | 'error';

export interface CastDevice {
  id: string;
  name: string;
  device_type: CastDeviceType;
  available: boolean;
}

export interface MediaCastingSettings {
  enabled: boolean;
  auto_discover: boolean;
  show_cast_indicator: boolean;
}

/**
 * Discover cast devices
 */
export async function discoverCastDevices(): Promise<CastDevice[]> {
  return invoke('discover_cast_devices');
}

/**
 * Get cast devices
 */
export async function getCastDevices(): Promise<CastDevice[]> {
  return invoke('get_cast_devices');
}

/**
 * Start cast
 */
export async function startCast(deviceId: string, url: string): Promise<boolean> {
  return invoke('start_cast', { deviceId, url });
}

/**
 * Stop cast
 */
export async function stopCast(): Promise<void> {
  return invoke('stop_cast');
}

/**
 * Get cast state
 */
export async function getCastState(): Promise<CastState> {
  return invoke('get_cast_state');
}

/**
 * Get current cast device
 */
export async function getCurrentCastDevice(): Promise<CastDevice | null> {
  return invoke('get_current_cast_device');
}

/**
 * Enable media casting
 */
export async function enableMediaCasting(): Promise<void> {
  return invoke('enable_media_casting');
}

/**
 * Disable media casting
 */
export async function disableMediaCasting(): Promise<void> {
  return invoke('disable_media_casting');
}

/**
 * Check if media casting is enabled
 */
export async function isMediaCastingEnabled(): Promise<boolean> {
  return invoke('is_media_casting_enabled');
}

/**
 * Set auto discover
 */
export async function setCastAutoDiscover(enabled: boolean): Promise<void> {
  return invoke('set_cast_auto_discover', { enabled });
}

/**
 * Get media casting settings
 */
export async function getMediaCastingSettings(): Promise<MediaCastingSettings> {
  return invoke('get_media_casting_settings');
}
