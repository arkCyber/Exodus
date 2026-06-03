/**
 * Exodus Browser — find-in-page (native webview + iframe fallback).
 */
import { type Ref } from 'vue';
import { evalTabReturning, findInTab, tabWebviewLabel } from '@/lib/exodusBrowser';

export type UseFindInPageOptions = {
  useNativeWebview: Ref<boolean>;
  getActiveTabId: () => string | null;
  getContentDocument: () => Document | null | undefined;
  findQuery: Ref<string>;
  findResults: Ref<number>;
  currentFindIndex: Ref<number>;
};

/**
 * Count matches and step through find results in the active tab.
 */
export function useFindInPage(options: UseFindInPageOptions) {
  function activeLabel(): string {
    const id = options.getActiveTabId();
    return id ? tabWebviewLabel(id) : '';
  }

  /** Count occurrences of the find query in page text. */
  async function countFindMatches(): Promise<number> {
    const q = options.findQuery.value.trim();
    if (!q) return 0;
    const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    try {
      if (options.useNativeWebview.value) {
        const script = `(function() {
          const content = document.body.innerText || '';
          const m = content.match(new RegExp(${JSON.stringify(escaped)}, 'gi'));
          return m ? m.length : 0;
        })()`;
        const raw = await evalTabReturning(activeLabel(), script);
        const count = typeof raw === 'string' ? JSON.parse(raw) : raw;
        return Number(count) || 0;
      }
      const content = options.getContentDocument()?.body?.innerText || '';
      const matches = content.match(new RegExp(escaped, 'gi'));
      return matches?.length || 0;
    } catch (error) {
      console.error('countFindMatches failed:', error);
      return 0;
    }
  }

  /** Highlight next/previous match; updates find result index. */
  async function findInPage(direction: 'next' | 'prev' = 'next'): Promise<void> {
    const q = options.findQuery.value.trim();
    if (!q) return;
    const forward = direction === 'next';
    try {
      options.findResults.value = await countFindMatches();
      let found = false;
      if (options.useNativeWebview.value) {
        found = await findInTab(activeLabel(), q, forward);
      } else {
        const doc = options.getContentDocument();
        const win = doc?.defaultView as
          | (Window & { find: (s: string, a: boolean, b: boolean, c: boolean) => boolean })
          | null
          | undefined;
        if (win?.find) {
          found = win.find(q, false, !forward, true);
        }
      }
      if (found) {
        if (forward) {
          options.currentFindIndex.value =
            options.findResults.value > 0
              ? (options.currentFindIndex.value % options.findResults.value) + 1
              : options.currentFindIndex.value + 1;
        } else {
          options.currentFindIndex.value =
            options.findResults.value > 0
              ? options.currentFindIndex.value <= 1
                ? options.findResults.value
                : options.currentFindIndex.value - 1
              : Math.max(1, options.currentFindIndex.value - 1);
        }
      } else if (options.findResults.value === 0) {
        options.currentFindIndex.value = 0;
      }
    } catch (error) {
      console.error('findInPage failed:', error);
    }
  }

  /** Debounced recount when the query changes. */
  async function onFindQueryInput(): Promise<void> {
    options.currentFindIndex.value = 0;
    options.findResults.value = await countFindMatches();
  }

  return {
    countFindMatches,
    findInPage,
    onFindQueryInput,
  };
}
