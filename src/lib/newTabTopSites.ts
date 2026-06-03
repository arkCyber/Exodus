/**
 * Exodus Browser — derive Brave/Chrome-style top sites from browsing history.
 */

import type { QuickLink } from '$lib/browserTypes';
import type { ManagedHistoryEntry } from '$lib/historyManager';

/**
 * Build top-site chips from recent history (dedupe by hostname, prefer high visit count).
 */
export function buildTopSitesFromHistory(
  entries: ManagedHistoryEntry[],
  limit = 8,
): QuickLink[] {
  const byHost = new Map<string, { title: string; url: string; score: number }>();

  for (const entry of entries) {
    if (!entry.url.startsWith('http://') && !entry.url.startsWith('https://')) continue;
    let host: string;
    try {
      host = new URL(entry.url).hostname;
    } catch {
      continue;
    }
    if (!host || host === 'localhost') continue;
    const score = (entry.visit_count ?? 1) + (entry.last_visit ?? 0) / 1e12;
    const prev = byHost.get(host);
    if (!prev || score > prev.score) {
      byHost.set(host, {
        title: entry.title?.trim() || host,
        url: entry.url,
        score,
      });
    }
  }

  return [...byHost.values()]
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map((row) => ({ title: row.title, url: row.url }));
}
