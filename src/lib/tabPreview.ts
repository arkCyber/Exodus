/**
 * Tab Preview API for Exodus Browser
 * Allows previewing tabs by capturing their content
 */

import { invoke } from '@tauri-apps/api/core';

export interface TabPreview {
  label: string;
  image_data: string; // Base64 encoded image
  timestamp: number;
  width: number;
  height: number;
}

/**
 * Register a tab preview
 */
export async function registerTabPreview(preview: TabPreview): Promise<void> {
  return invoke('register_tab_preview', { preview });
}

/**
 * Unregister a tab preview
 */
export async function unregisterTabPreview(label: string): Promise<void> {
  return invoke('unregister_tab_preview', { label });
}

/**
 * Get a tab preview
 */
export async function getTabPreview(label: string): Promise<TabPreview | null> {
  return invoke('get_tab_preview', { label });
}

/**
 * Get all tab previews
 */
export async function getAllTabPreviews(): Promise<TabPreview[]> {
  return invoke('get_all_tab_previews');
}

/**
 * Clear all tab previews
 */
export async function clearTabPreviews(): Promise<void> {
  return invoke('clear_tab_previews');
}
