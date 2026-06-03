/**
 * Reading Progress Tracking API for Exodus Browser
 * Tracks scroll position and reading progress for web pages
 */

import { invoke } from '@tauri-apps/api/core';

export interface ReadingProgress {
  url: string;
  title: string;
  scroll_position: number;
  scroll_y: number;
  total_height: number;
  last_read_time: number;
  is_completed: boolean;
}

export interface ReadingProgressSettings {
  enabled: boolean;
  auto_save: boolean;
  save_interval_seconds: number;
  show_progress_indicator: boolean;
  mark_completed_threshold: number;
}

/**
 * Update reading progress
 */
export async function updateReadingProgress(
  url: string,
  title: string,
  scrollPosition: number,
  scrollY: number,
  totalHeight: number
): Promise<void> {
  return invoke('update_reading_progress', { url, title, scrollPosition, scrollY, totalHeight });
}

/**
 * Get reading progress for a URL
 */
export async function getReadingProgress(url: string): Promise<ReadingProgress | null> {
  return invoke('get_reading_progress', { url });
}

/**
 * Get all reading progress
 */
export async function getAllReadingProgress(): Promise<ReadingProgress[]> {
  return invoke('get_all_reading_progress');
}

/**
 * Get completed articles
 */
export async function getCompletedReading(): Promise<ReadingProgress[]> {
  return invoke('get_completed_reading');
}

/**
 * Get in-progress articles
 */
export async function getInProgressReading(): Promise<ReadingProgress[]> {
  return invoke('get_in_progress_reading');
}

/**
 * Mark article as completed
 */
export async function markReadingCompleted(url: string): Promise<void> {
  return invoke('mark_reading_completed', { url });
}

/**
 * Reset progress for a URL
 */
export async function resetReadingProgress(url: string): Promise<void> {
  return invoke('reset_reading_progress', { url });
}

/**
 * Delete progress for a URL
 */
export async function deleteReadingProgress(url: string): Promise<void> {
  return invoke('delete_reading_progress', { url });
}

/**
 * Clear all progress
 */
export async function clearReadingProgress(): Promise<void> {
  return invoke('clear_reading_progress');
}

/**
 * Enable reading progress tracking
 */
export async function enableReadingProgress(): Promise<void> {
  return invoke('enable_reading_progress');
}

/**
 * Disable reading progress tracking
 */
export async function disableReadingProgress(): Promise<void> {
  return invoke('disable_reading_progress');
}

/**
 * Check if reading progress is enabled
 */
export async function isReadingProgressEnabled(): Promise<boolean> {
  return invoke('is_reading_progress_enabled');
}

/**
 * Set auto-save
 */
export async function setReadingProgressAutoSave(enabled: boolean): Promise<void> {
  return invoke('set_reading_progress_auto_save', { enabled });
}

/**
 * Set save interval
 */
export async function setReadingProgressInterval(seconds: number): Promise<void> {
  return invoke('set_reading_progress_interval', { seconds });
}

/**
 * Set show indicator
 */
export async function setReadingProgressIndicator(show: boolean): Promise<void> {
  return invoke('set_reading_progress_indicator', { show });
}

/**
 * Set completed threshold
 */
export async function setReadingProgressThreshold(threshold: number): Promise<void> {
  return invoke('set_reading_progress_threshold', { threshold });
}

/**
 * Get reading progress settings
 */
export async function getReadingProgressSettings(): Promise<ReadingProgressSettings> {
  return invoke('get_reading_progress_settings');
}
