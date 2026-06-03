/**
 * Exodus Browser — horizontal tab strip rules aligned with Google Chrome.
 */

/**
 * Whether the tab close affordance (×) should be visible.
 * Chrome shows × on unpinned tabs (including new-tab page); pinned tabs have no close.
 */
export function canShowTabClose(
  _tabCount: number,
  tab: { pinned?: boolean },
): boolean {
  return !tab.pinned;
}
