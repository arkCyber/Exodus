/**
 * Exodus Browser — shared BrowserTabBar event handlers (top strip + sidebar vertical tabs).
 */
import type { useBrowserTabGroups } from '@/composables/useBrowserTabGroups';

export type TabBarHandlersOptions = {
  activateTab: (id: string) => void | Promise<void>;
  createNewTab: () => void | Promise<void>;
  closeTab: (id: string, force?: boolean) => void | Promise<void>;
  toggleTabPin: (id: string) => void;
  duplicateTab: (id: string) => void | Promise<void>;
  reorderTabs: (fromId: string, toId: string) => void;
  tabGroups: ReturnType<typeof useBrowserTabGroups>;
};

/** v-on map for BrowserTabBar (top strip + sidebar vertical tabs). */
export type TabBarHandlerMap = {
  switchTab: (id: string) => void;
  newTab: () => void;
  closeTab: (id: string, force?: boolean) => void;
  tabMouseDown: (e: MouseEvent, id: string) => void;
  tabContextMenu: (e: MouseEvent, id: string) => void;
  closeContextMenu: () => void;
  togglePin: (id: string) => void;
  duplicateTab: (id: string) => void;
  reorderTabs: (fromId: string, toId: string) => void;
  newTabGroup: (id: string) => void;
  addTabToGroup: (tabId: string, groupId: string) => void;
  removeTabFromGroup: (id: string) => void;
  toggleGroupCollapse: (groupId: string, collapsed: boolean) => void;
  renameTabGroup: (groupId: string) => void;
  cycleTabGroupColor: (groupId: string) => void;
  deleteTabGroup: (groupId: string) => void;
};

/**
 * Build v-on listeners for BrowserTabBar (Firefox / Chrome tab context menu).
 */
export function buildTabBarHandlers(opts: TabBarHandlersOptions): TabBarHandlerMap {
  const tg = opts.tabGroups;
  return {
    switchTab: (id: string) => void opts.activateTab(id),
    newTab: () => void opts.createNewTab(),
    closeTab: (id: string, force?: boolean) => void opts.closeTab(id, force),
    tabMouseDown: (e: MouseEvent, id: string) => {
      if (e.button === 1) {
        e.preventDefault();
        void opts.closeTab(id, true);
      }
    },
    tabContextMenu: (e: MouseEvent, id: string) => tg.openTabContextMenu(e, id),
    closeContextMenu: () => tg.closeTabContextMenu(),
    togglePin: (id: string) => opts.toggleTabPin(id),
    duplicateTab: (id: string) => void opts.duplicateTab(id),
    reorderTabs: (fromId: string, toId: string) => opts.reorderTabs(fromId, toId),
    newTabGroup: (id: string) => void tg.newTabGroupFromTab(id),
    addTabToGroup: (tabId: string, groupId: string) => void tg.addTabToExistingGroup(tabId, groupId),
    removeTabFromGroup: (id: string) => void tg.removeTabGroupMembership(id),
    toggleGroupCollapse: (groupId: string, collapsed: boolean) =>
      void tg.toggleTabGroupCollapse(groupId, collapsed),
    renameTabGroup: (groupId: string) => tg.renameTabGroupPrompt(groupId),
    cycleTabGroupColor: (groupId: string) => void tg.cycleTabGroupColor(groupId),
    deleteTabGroup: (groupId: string) => tg.deleteTabGroupById(groupId),
  };
}
