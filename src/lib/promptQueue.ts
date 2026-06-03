/**
 * Exodus Browser — FIFO helper for single-modal permission prompts.
 */

/** Enqueue an item; show immediately if no active prompt. */
export function enqueuePrompt<T>(active: T | null, queue: T[], item: T): { active: T; queue: T[] } {
  if (active === null) {
    return { active: item, queue };
  }
  return { active, queue: [...queue, item] };
}

/** Advance to the next queued prompt after the active one is resolved. */
export function advancePromptQueue<T>(queue: T[]): T | null {
  return queue.length > 0 ? queue.shift()! : null;
}
