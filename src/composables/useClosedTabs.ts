/**
 * Exodus Browser — recently closed tab stack for ⌘⇧T restore.
 */
import { ref, computed } from 'vue';
import type { ClosedTabSnapshot } from '@/lib/browserTypes';

const DEFAULT_MAX = 25;

/**
 * FIFO stack of closed tabs (most recent first).
 */
export function useClosedTabs(maxSize = DEFAULT_MAX) {
  const closedTabs = ref<ClosedTabSnapshot[]>([]);

  const closedTabsCount = computed(() => closedTabs.value.length);

  /** Push a tab snapshot onto the restore stack. */
  function recordClosedTab(tab: ClosedTabSnapshot): void {
    closedTabs.value = [tab, ...closedTabs.value].slice(0, maxSize);
  }

  /** Remove and return the most recently closed tab, if any. */
  function popClosedTab(): ClosedTabSnapshot | undefined {
    const snap = closedTabs.value[0];
    if (!snap) return undefined;
    closedTabs.value = closedTabs.value.slice(1);
    return snap;
  }

  return {
    closedTabs,
    closedTabsCount,
    recordClosedTab,
    popClosedTab,
  };
}
