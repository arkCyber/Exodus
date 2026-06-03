/**
 * Color Blind Mode API for Exodus Browser
 * Provides color blindness accessibility features
 */

import { invoke } from '@tauri-apps/api/core';

export type ColorBlindType = 'none' | 'protanopia' | 'deuteranopia' | 'tritanopia' | 'achromatopsia';

export interface ColorBlindSettings {
  enabled: boolean;
  color_blind_type: ColorBlindType;
  intensity: number;
}

/**
 * Enable color blind mode
 */
export async function enableColorBlind(): Promise<void> {
  return invoke('enable_color_blind');
}

/**
 * Disable color blind mode
 */
export async function disableColorBlind(): Promise<void> {
  return invoke('disable_color_blind');
}

/**
 * Check if color blind mode is enabled
 */
export async function isColorBlindEnabled(): Promise<boolean> {
  return invoke('is_color_blind_enabled');
}

/**
 * Set color blind type
 */
export async function setColorBlindType(colorBlindType: ColorBlindType): Promise<void> {
  return invoke('set_color_blind_type', { colorBlindType });
}

/**
 * Get color blind type
 */
export async function getColorBlindType(): Promise<ColorBlindType> {
  return invoke('get_color_blind_type');
}

/**
 * Set color blind intensity
 */
export async function setColorBlindIntensity(intensity: number): Promise<void> {
  return invoke('set_color_blind_intensity', { intensity });
}

/**
 * Get color blind intensity
 */
export async function getColorBlindIntensity(): Promise<number> {
  return invoke('get_color_blind_intensity');
}

/**
 * Get color blind settings
 */
export async function getColorBlindSettings(): Promise<ColorBlindSettings> {
  return invoke('get_color_blind_settings');
}
