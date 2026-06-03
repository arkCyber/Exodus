/**
 * Exodus Browser — tab strip drag-reorder helpers (Chrome-style pinned / unpinned).
 */

export type TabOrderItem = { id: string; pinned?: boolean };

/**
 * Move a tab before another tab in display order (pinned tabs only among pinned).
 */
export function reorderTabsById<T extends TabOrderItem>(
  tabs: T[],
  fromId: string,
  toId: string,
): T[] {
  if (fromId === toId) return tabs;
  const from = tabs.find((t) => t.id === fromId);
  const to = tabs.find((t) => t.id === toId);
  if (!from || !to) return tabs;
  if (!!from.pinned !== !!to.pinned) return tabs;

  const order = tabs.map((t) => t.id);
  const fromIdx = order.indexOf(fromId);
  const toIdx = order.indexOf(toId);
  if (fromIdx < 0 || toIdx < 0) return tabs;

  order.splice(fromIdx, 1);
  order.splice(toIdx, 0, fromId);

  const byId = new Map(tabs.map((t) => [t.id, t]));
  return order.map((id) => byId.get(id)).filter((t): t is T => !!t);
}
