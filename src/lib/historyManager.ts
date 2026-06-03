/**
 * Exodus Browser — full browsing history API (history_manager backend).
 */

import { invoke } from '@tauri-apps/api/core';
import type { HistoryPage } from '$lib/browserTypes';

/** Single history entry from the history manager store. */
export type ManagedHistoryEntry = {
  id: string;
  url: string;
  title: string;
  visit_time: number;
  visit_count: number;
  last_visit: number;
  favicon?: string | null;
  referrer?: string | null;
  transition_type: string;
};

/** History manager settings. */
export type HistoryManagerSettings = {
  enabled: boolean;
  remember_browsing: boolean;
  remember_downloads: boolean;
  remember_form_data: boolean;
  retention_days: number;
  clear_on_exit: boolean;
  allow_incognito: boolean;
};

/** Record a page visit in the history manager. */
export async function addManagedHistoryEntry(url: string, title: string): Promise<void> {
  try {
    await invoke('add_history_entry', { url, title });
  } catch (error) {
    console.error('add_history_entry failed:', error);
  }
}

/** Recent history entries (newest first). */
export async function getRecentManagedHistory(limit = 50): Promise<ManagedHistoryEntry[]> {
  try {
    return await invoke<ManagedHistoryEntry[]>('get_recent_history', { limit });
  } catch (error) {
    console.error('get_recent_history failed:', error);
    return [];
  }
}

/** Search history by URL/title substring. */
export async function searchManagedHistory(query: string): Promise<ManagedHistoryEntry[]> {
  try {
    return await invoke<ManagedHistoryEntry[]>('search_history', { query });
  } catch (error) {
    console.error('search_history failed:', error);
    return [];
  }
}

/** Remove one history entry by id. */
export async function removeManagedHistoryEntry(id: string): Promise<void> {
  await invoke('remove_history_entry', { id });
}

/** Remove all entries for a domain. */
export async function removeManagedHistoryByDomain(domain: string): Promise<void> {
  await invoke('remove_history_by_domain', { domain });
}

/** Clear the full managed history store. */
export async function clearAllManagedHistory(): Promise<void> {
  await invoke('clear_all_history');
}

/** Load history manager settings. */
export async function loadHistoryManagerSettings(): Promise<HistoryManagerSettings> {
  return invoke<HistoryManagerSettings>('get_history_settings');
}

/** Persist history manager settings. */
export async function saveHistoryManagerSettings(
  settings: HistoryManagerSettings,
): Promise<void> {
  await invoke('update_history_settings', { settings });
}

/** Aggregate stats (total entries, domains, etc.). */
export async function getManagedHistoryStats(): Promise<Record<string, number>> {
  try {
    return await invoke<Record<string, number>>('get_history_stats');
  } catch (error) {
    console.error('get_history_stats failed:', error);
    return {};
  }
}

/** Visit rows from RAG visit store. */
export async function fetchVisitHistory(): Promise<HistoryPage[]> {
  try {
    const visits = await invoke<
      Array<{
        id: string;
        url: string;
        title: string;
        timestamp: string;
        visit_count: number;
      }>
    >('get_visit_history');
    return visits.map((v) => ({
      id: v.id,
      url: v.url,
      title: v.title,
      timestamp: v.timestamp,
      visit_count: v.visit_count,
    }));
  } catch (error) {
    console.error('get_visit_history failed:', error);
    return [];
  }
}

/** Merge RAG visits with history_manager entries (dedupe by URL, newest wins). */
export function mergeBrowsingHistoryLists(
  visits: HistoryPage[],
  managed: ManagedHistoryEntry[],
): HistoryPage[] {
  const byUrl = new Map<string, HistoryPage>();

  for (const v of visits) {
    byUrl.set(v.url, { ...v });
  }

  for (const m of managed) {
    const ts = new Date(m.last_visit * 1000).toISOString();
    const existing = byUrl.get(m.url);
    if (existing) {
      const newer = ts > existing.timestamp ? ts : existing.timestamp;
      byUrl.set(m.url, {
        id: existing.id || m.id,
        url: m.url,
        title: m.title || existing.title,
        timestamp: newer,
        visit_count: Math.max(existing.visit_count ?? 1, m.visit_count),
      });
    } else {
      byUrl.set(m.url, {
        id: m.id,
        url: m.url,
        title: m.title,
        timestamp: ts,
        visit_count: m.visit_count,
      });
    }
  }

  return [...byUrl.values()].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
  );
}

/** Load unified browsing history for sidebar and omnibox seeding. */
export async function loadMergedBrowsingHistory(limit = 200): Promise<HistoryPage[]> {
  const [visits, managed] = await Promise.all([
    fetchVisitHistory(),
    getRecentManagedHistory(limit),
  ]);
  return mergeBrowsingHistoryLists(visits, managed);
}

/** Search both history stores and merge results. */
export async function searchMergedBrowsingHistory(query: string): Promise<HistoryPage[]> {
  const q = query.trim();
  if (!q) return loadMergedBrowsingHistory();
  try {
    const visits = await invoke<
      Array<{
        id: string;
        url: string;
        title: string;
        timestamp: string;
        visit_count: number;
      }>
    >('search_visits', { query: q });
    const visitPages: HistoryPage[] = visits.map((v) => ({
      id: v.id,
      url: v.url,
      title: v.title,
      timestamp: v.timestamp,
      visit_count: v.visit_count,
    }));
    const managed = await searchManagedHistory(q);
    return mergeBrowsingHistoryLists(visitPages, managed);
  } catch (error) {
    console.error('searchMergedBrowsingHistory failed:', error);
    return [];
  }
}
