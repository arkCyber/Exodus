/**
 * Exodus Browser — helpers for RAG indexed memory lists.
 */

import type { HistoryGroup } from '$lib/historyGroups';
import { groupHistoryByDate } from '$lib/historyGroups';
import type { IndexedPage } from '$lib/browserTypes';

/** Group indexed pages by Today / Yesterday / date (same labels as visit history). */
export function groupIndexedByDate(pages: IndexedPage[]): HistoryGroup[] {
  return groupHistoryByDate(pages);
}
