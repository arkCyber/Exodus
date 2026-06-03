/**
 * Exodus Browser — tab grouping API (Chrome-style tab groups).
 */

import { invoke } from '@tauri-apps/api/core';

/** Tab group row from `get_all_tab_groups`. */
export type TabGroup = {
  id: string;
  title: string;
  color: string;
  tab_ids: string[];
  created_at: number;
  last_modified: number;
  collapsed: boolean;
};

/** CSS color for a group chip / tab stripe. */
export function tabGroupColorCss(color: string): string {
  switch (color.toLowerCase()) {
    case 'blue':
      return '#3b82f6';
    case 'red':
      return '#ef4444';
    case 'yellow':
      return '#eab308';
    case 'green':
      return '#22c55e';
    case 'pink':
      return '#ec4899';
    case 'purple':
      return '#a855f7';
    case 'cyan':
      return '#06b6d4';
    case 'orange':
      return '#f97316';
    default:
      return '#6b7280';
  }
}

/** List all tab groups. */
export async function listTabGroups(): Promise<TabGroup[]> {
  try {
    return await invoke<TabGroup[]>('get_all_tab_groups');
  } catch (error) {
    console.error('get_all_tab_groups failed:', error);
    return [];
  }
}

/** Create a tab group; returns group id. */
export async function createTabGroup(title: string, color: string): Promise<string> {
  return invoke<string>('create_tab_group', { title, color });
}

/** Add a tab to a group. */
export async function addTabToGroup(groupId: string, tabId: string): Promise<void> {
  await invoke('add_tab_to_group', { groupId, tabId });
}

/** Remove a tab from its group. */
export async function removeTabFromGroup(tabId: string): Promise<void> {
  await invoke('remove_tab_from_group', { tabId });
}

/** Collapse a tab group in the tab strip. */
export async function collapseTabGroup(groupId: string): Promise<void> {
  await invoke('collapse_tab_group', { id: groupId });
}

/** Expand a collapsed tab group. */
export async function expandTabGroup(groupId: string): Promise<void> {
  await invoke('expand_tab_group', { id: groupId });
}

/** Update group title and/or color. */
export async function updateTabGroup(
  groupId: string,
  title?: string,
  color?: string,
): Promise<void> {
  await invoke('update_tab_group', {
    id: groupId,
    title: title ?? null,
    color: color ?? null,
  });
}

/** Delete a tab group (tabs stay open, ungrouped). */
export async function deleteTabGroup(groupId: string): Promise<void> {
  await invoke('delete_tab_group', { id: groupId });
}

/** Preset colors for new / edited groups. */
export const TAB_GROUP_COLORS = [
  'grey',
  'blue',
  'red',
  'yellow',
  'green',
  'pink',
  'purple',
  'cyan',
  'orange',
] as const;

/** Find the group containing a tab id. */
export function groupForTab(groups: TabGroup[], tabId: string): TabGroup | undefined {
  return groups.find((g) => g.tab_ids.includes(tabId));
}

/** Order tabs: pinned, then by group order, then ungrouped. Respects collapsed groups. */
export function sortTabsWithGroups(
  tabs: Array<{ id: string; pinned?: boolean }>,
  groups: TabGroup[],
  activeTabId: string,
): string[] {
  const pinned = tabs.filter((t) => t.pinned).map((t) => t.id);
  const unpinned = tabs.filter((t) => !t.pinned);
  const unpinnedIds = new Set(unpinned.map((t) => t.id));
  const ordered: string[] = [...pinned];
  const placed = new Set<string>(pinned);

  for (const g of groups) {
    if (g.collapsed) {
      if (!g.tab_ids.includes(activeTabId)) {
        for (const tid of g.tab_ids) {
          if (unpinnedIds.has(tid)) {
            placed.add(tid);
          }
        }
        continue;
      }
      if (unpinnedIds.has(activeTabId) && g.tab_ids.includes(activeTabId)) {
        ordered.push(activeTabId);
        placed.add(activeTabId);
      }
      for (const tid of g.tab_ids) {
        if (tid !== activeTabId && unpinnedIds.has(tid)) {
          placed.add(tid);
        }
      }
      continue;
    }
    for (const tid of g.tab_ids) {
      if (unpinnedIds.has(tid) && !placed.has(tid)) {
        ordered.push(tid);
        placed.add(tid);
      }
    }
  }
  for (const t of unpinned) {
    if (!placed.has(t.id)) {
      ordered.push(t.id);
    }
  }
  return ordered;
}
