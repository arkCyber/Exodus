/**
 * Tab Stacking API for Exodus Browser
 * Provides nested tab organization (tab groups within tab groups)
 */

import { invoke } from '@tauri-apps/api/core';

export interface TabStack {
  stack_id: string;
  parent_stack_id?: string;
  label: string;
  color: string;
  collapsed: boolean;
  tab_count: number;
}

export interface TabStackSettings {
  enabled: boolean;
  auto_group_by_domain: boolean;
  show_stack_badges: boolean;
  collapse_on_navigate_away: boolean;
}

/**
 * Create a stack
 */
export async function createTabStack(
  stackId: string,
  label: string,
  color: string,
  parentStackId?: string
): Promise<void> {
  return invoke('create_tab_stack', { stackId, label, color, parentStackId });
}

/**
 * Delete a stack
 */
export async function deleteTabStack(stackId: string): Promise<void> {
  return invoke('delete_tab_stack', { stackId });
}

/**
 * Add tab to stack
 */
export async function addTabToStack(tabLabel: string, stackId: string): Promise<void> {
  return invoke('add_tab_to_stack', { tabLabel, stackId });
}

/**
 * Remove tab from stack
 */
export async function removeTabFromStack(tabLabel: string): Promise<void> {
  return invoke('remove_tab_from_stack', { tabLabel });
}

/**
 * Move tab to stack
 */
export async function moveTabToStack(tabLabel: string, newStackId: string): Promise<void> {
  return invoke('move_tab_to_stack', { tabLabel, newStackId });
}

/**
 * Collapse stack
 */
export async function collapseTabStack(stackId: string): Promise<void> {
  return invoke('collapse_tab_stack', { stackId });
}

/**
 * Expand stack
 */
export async function expandTabStack(stackId: string): Promise<void> {
  return invoke('expand_tab_stack', { stackId });
}

/**
 * Toggle stack collapse
 */
export async function toggleTabStackCollapse(stackId: string): Promise<void> {
  return invoke('toggle_tab_stack_collapse', { stackId });
}

/**
 * Rename stack
 */
export async function renameTabStack(stackId: string, newLabel: string): Promise<void> {
  return invoke('rename_tab_stack', { stackId, newLabel });
}

/**
 * Change stack color
 */
export async function changeTabStackColor(stackId: string, newColor: string): Promise<void> {
  return invoke('change_tab_stack_color', { stackId, newColor });
}

/**
 * Get tab stack
 */
export async function getTabStack(tabLabel: string): Promise<TabStack | null> {
  return invoke('get_tab_stack', { tabLabel });
}

/**
 * Get all stacks
 */
export async function getAllTabStacks(): Promise<TabStack[]> {
  return invoke('get_all_tab_stacks');
}

/**
 * Get stack tabs
 */
export async function getStackTabs(stackId: string): Promise<string[]> {
  return invoke('get_stack_tabs', { stackId });
}

/**
 * Enable tab stacking
 */
export async function enableTabStacking(): Promise<void> {
  return invoke('enable_tab_stacking');
}

/**
 * Disable tab stacking
 */
export async function disableTabStacking(): Promise<void> {
  return invoke('disable_tab_stacking');
}

/**
 * Check if tab stacking is enabled
 */
export async function isTabStackingEnabled(): Promise<boolean> {
  return invoke('is_tab_stacking_enabled');
}

/**
 * Set auto group by domain
 */
export async function setTabStackingAutoGroup(enabled: boolean): Promise<void> {
  return invoke('set_tab_stacking_auto_group', { enabled });
}

/**
 * Get tab stacking settings
 */
export async function getTabStackingSettings(): Promise<TabStackSettings> {
  return invoke('get_tab_stacking_settings');
}
