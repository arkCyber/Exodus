/**
 * Data Saver Mode API for Exodus Browser
 * Reduces data usage by compressing images and blocking media
 */

import { invoke } from '@tauri-apps/api/core';

export interface DataSaverSettings {
  enabled: boolean;
  block_images: boolean;
  block_videos: boolean;
  block_autoplay_media: boolean;
  compress_images: boolean;
  quality_level: number;
}

/**
 * Enable data saver
 */
export async function enableDataSaver(): Promise<void> {
  return invoke('enable_data_saver');
}

/**
 * Disable data saver
 */
export async function disableDataSaver(): Promise<void> {
  return invoke('disable_data_saver');
}

/**
 * Check if data saver is enabled
 */
export async function isDataSaverEnabled(): Promise<boolean> {
  return invoke('is_data_saver_enabled');
}

/**
 * Set block images
 */
export async function setDataSaverBlockImages(block: boolean): Promise<void> {
  return invoke('set_data_saver_block_images', { block });
}

/**
 * Set block videos
 */
export async function setDataSaverBlockVideos(block: boolean): Promise<void> {
  return invoke('set_data_saver_block_videos', { block });
}

/**
 * Set block autoplay media
 */
export async function setDataSaverBlockAutoplay(block: boolean): Promise<void> {
  return invoke('set_data_saver_block_autoplay', { block });
}

/**
 * Set compress images
 */
export async function setDataSaverCompressImages(compress: boolean): Promise<void> {
  return invoke('set_data_saver_compress_images', { compress });
}

/**
 * Set quality level
 */
export async function setDataSaverQualityLevel(level: number): Promise<void> {
  return invoke('set_data_saver_quality_level', { level });
}

/**
 * Record bytes saved
 */
export async function recordDataSaverBytes(bytes: number): Promise<void> {
  return invoke('record_data_saver_bytes', { bytes });
}

/**
 * Get bytes saved
 */
export async function getDataSaverBytesSaved(): Promise<number> {
  return invoke('get_data_saver_bytes_saved');
}

/**
 * Reset bytes saved
 */
export async function resetDataSaverBytes(): Promise<void> {
  return invoke('reset_data_saver_bytes');
}

/**
 * Get data saver settings
 */
export async function getDataSaverSettings(): Promise<DataSaverSettings> {
  return invoke('get_data_saver_settings');
}
