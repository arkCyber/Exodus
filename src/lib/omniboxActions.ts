/**
 * Omnibox Quick Actions API for Exodus Browser
 * Provides quick actions like calculator, unit conversion from the address bar
 */

import { invoke } from '@tauri-apps/api/core';

export type OmniboxActionType = 
  | { type: 'Calculator'; expression: string; result: string }
  | { type: 'UnitConversion'; from: string; to: string; value: number; result: number }
  | { type: 'Search'; query: string; engine: string }
  | { type: 'None' };

export interface OmnActionResult {
  action: OmniboxActionType;
  display_text: string;
  url?: string;
}

/**
 * Enable omnibox actions
 */
export async function enableOmniboxActions(): Promise<void> {
  return invoke('enable_omnibox_actions');
}

/**
 * Disable omnibox actions
 */
export async function disableOmniboxActions(): Promise<void> {
  return invoke('disable_omnibox_actions');
}

/**
 * Check if omnibox actions are enabled
 */
export async function isOmniboxActionsEnabled(): Promise<boolean> {
  return invoke('is_omnibox_actions_enabled');
}

/**
 * Parse omnibox input and return action if applicable
 */
export async function parseOmniboxInput(input: string): Promise<OmnActionResult | null> {
  return invoke('parse_omnibox_input', { input });
}
