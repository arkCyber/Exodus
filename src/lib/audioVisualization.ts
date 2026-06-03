/**
 * Audio Visualization API for Exodus Browser
 * Provides audio visualization features for media playback
 */

import { invoke } from '@tauri-apps/api/core';

export type VisualizationType = 'none' | 'bars' | 'waveform' | 'circular' | 'spectrum';

export interface AudioVisualizationSettings {
  enabled: boolean;
  visualization_type: VisualizationType;
  sensitivity: number;
  smoothing: number;
  color_scheme: string;
}

/**
 * Enable audio visualization
 */
export async function enableAudioVisualization(): Promise<void> {
  return invoke('enable_audio_visualization');
}

/**
 * Disable audio visualization
 */
export async function disableAudioVisualization(): Promise<void> {
  return invoke('disable_audio_visualization');
}

/**
 * Check if audio visualization is enabled
 */
export async function isAudioVisualizationEnabled(): Promise<boolean> {
  return invoke('is_audio_visualization_enabled');
}

/**
 * Set visualization type
 */
export async function setVisualizationType(vizType: VisualizationType): Promise<void> {
  return invoke('set_visualization_type', { vizType });
}

/**
 * Get visualization type
 */
export async function getVisualizationType(): Promise<VisualizationType> {
  return invoke('get_visualization_type');
}

/**
 * Set visualization sensitivity
 */
export async function setVisualizationSensitivity(sensitivity: number): Promise<void> {
  return invoke('set_visualization_sensitivity', { sensitivity });
}

/**
 * Get visualization sensitivity
 */
export async function getVisualizationSensitivity(): Promise<number> {
  return invoke('get_visualization_sensitivity');
}

/**
 * Set visualization smoothing
 */
export async function setVisualizationSmoothing(smoothing: number): Promise<void> {
  return invoke('set_visualization_smoothing', { smoothing });
}

/**
 * Get visualization smoothing
 */
export async function getVisualizationSmoothing(): Promise<number> {
  return invoke('get_visualization_smoothing');
}

/**
 * Set visualization color scheme
 */
export async function setVisualizationColorScheme(scheme: string): Promise<void> {
  return invoke('set_visualization_color_scheme', { scheme });
}

/**
 * Get visualization color scheme
 */
export async function getVisualizationColorScheme(): Promise<string> {
  return invoke('get_visualization_color_scheme');
}

/**
 * Get audio visualization settings
 */
export async function getAudioVisualizationSettings(): Promise<AudioVisualizationSettings> {
  return invoke('get_audio_visualization_settings');
}
