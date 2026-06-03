/**
 * Exodus Browser — group browsing history by calendar day labels.
 */

import type { HistoryPage } from '$lib/browserTypes';

export type HistoryGroup = {
  label: string;
  pages: HistoryPage[];
};

function startOfDay(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate());
}

function isSameDay(a: Date, b: Date): boolean {
  return startOfDay(a).getTime() === startOfDay(b).getTime();
}

/**
 * Group history pages under Today / Yesterday / locale date headings.
 */
export function groupHistoryByDate(pages: HistoryPage[]): HistoryGroup[] {
  const sorted = [...pages].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
  );

  const groups: HistoryGroup[] = [];
  const indexByLabel = new Map<string, number>();

  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  for (const page of sorted) {
    const d = new Date(page.timestamp);
    let label: string;
    if (isSameDay(d, today)) {
      label = 'Today';
    } else if (isSameDay(d, yesterday)) {
      label = 'Yesterday';
    } else {
      label = d.toLocaleDateString(undefined, {
        weekday: 'short',
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    }

    const idx = indexByLabel.get(label);
    if (idx === undefined) {
      indexByLabel.set(label, groups.length);
      groups.push({ label, pages: [page] });
    } else {
      groups[idx].pages.push(page);
    }
  }

  return groups;
}
