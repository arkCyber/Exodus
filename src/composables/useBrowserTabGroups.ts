/**
 * Exodus Browser — tab group state and actions (Chrome-style groups).
 */
import { ref, computed } from 'vue';
import {
  addTabToGroup,
  collapseTabGroup,
  createTabGroup,
  deleteTabGroup,
  expandTabGroup,
  listTabGroups,
  removeTabFromGroup,
  sortTabsWithGroups,
  updateTabGroup,
  TAB_GROUP_COLORS,
  type TabGroup,
  type TabGroupEditOffer,
} from '@/lib/tabGroups';

export type TabLike = { id: string; title: string; url: string; pinned?: boolean };

export type UseBrowserTabGroupsOptions = {
  getTabs: () => TabLike[];
  getActiveTabId: () => string | null;
  onStatus: (message: string) => void;
};

/**
 * Tab groups list, sorted tab order, and context-menu handlers.
 */
export function useBrowserTabGroups(options: UseBrowserTabGroupsOptions) {
  const tabGroups = ref<TabGroup[]>([]);
  const tabContextMenu = ref<{ tabId: string; x: number; y: number } | null>(null);
  const tabGroupEditOffer = ref<TabGroupEditOffer | null>(null);
  const tabGroupEditBusy = ref(false);
  const tabGroupDeleteOffer = ref<{ groupId: string; title: string } | null>(null);
  const tabGroupDeleteBusy = ref(false);

  const sortedTabIds = computed(() =>
    sortTabsWithGroups(options.getTabs(), tabGroups.value, options.getActiveTabId() ?? ''),
  );

  const sortedTabs = computed((): TabLike[] => {
    const tabs = options.getTabs();
    const byId = new Map(tabs.map((t) => [t.id, t]));
    return sortedTabIds.value.map((id) => byId.get(id)).filter((t): t is TabLike => !!t);
  });

  async function loadTabGroups(): Promise<void> {
    tabGroups.value = await listTabGroups();
  }

  function openTabContextMenu(e: MouseEvent, tabId: string): void {
    e.preventDefault();
    tabContextMenu.value = { tabId, x: e.clientX, y: e.clientY };
  }

  function closeTabContextMenu(): void {
    tabContextMenu.value = null;
  }

  async function newTabGroupFromTab(tabId: string): Promise<void> {
    const tab = options.getTabs().find((t) => t.id === tabId);
    const title = (tab?.title || 'Group').slice(0, 28);
    try {
      const groupId = await createTabGroup(title, 'blue');
      await addTabToGroup(groupId, tabId);
      await loadTabGroups();
      options.onStatus(`Tab group "${title}" created`);
    } catch (error) {
      console.error('newTabGroupFromTab failed:', error);
      options.onStatus('Failed to create tab group');
    }
    closeTabContextMenu();
  }

  async function addTabToExistingGroup(tabId: string, groupId: string): Promise<void> {
    try {
      await addTabToGroup(groupId, tabId);
      await loadTabGroups();
      options.onStatus('Tab added to group');
    } catch (error) {
      console.error('addTabToExistingGroup failed:', error);
      options.onStatus('Failed to add tab to group');
    }
    closeTabContextMenu();
  }

  async function removeTabGroupMembership(tabId: string): Promise<void> {
    try {
      await removeTabFromGroup(tabId);
      await loadTabGroups();
    } catch (error) {
      console.error('removeTabFromGroup failed:', error);
    }
    closeTabContextMenu();
  }

  function renameTabGroupPrompt(groupId: string): void {
    const group = tabGroups.value.find((g) => g.id === groupId);
    if (!group) return;
    tabGroupEditOffer.value = { groupId, title: group.title, color: group.color };
    closeTabContextMenu();
  }

  async function saveTabGroupEdit(title: string, color: string): Promise<void> {
    if (!tabGroupEditOffer.value) return;
    tabGroupEditBusy.value = true;
    const { groupId } = tabGroupEditOffer.value;
    try {
      await updateTabGroup(groupId, title, color);
      await loadTabGroups();
      options.onStatus('Tab group updated');
      tabGroupEditOffer.value = null;
    } catch (error) {
      console.error('saveTabGroupEdit failed:', error);
      options.onStatus('Failed to update group');
    } finally {
      tabGroupEditBusy.value = false;
    }
  }

  function cancelTabGroupEdit(): void {
    tabGroupEditOffer.value = null;
  }

  async function cycleTabGroupColor(groupId: string): Promise<void> {
    const group = tabGroups.value.find((g) => g.id === groupId);
    if (!group) return;
    const idx = TAB_GROUP_COLORS.findIndex((c) => c === group.color.toLowerCase());
    const next = TAB_GROUP_COLORS[(idx + 1) % TAB_GROUP_COLORS.length];
    try {
      await updateTabGroup(groupId, undefined, next);
      await loadTabGroups();
    } catch (error) {
      console.error('cycleTabGroupColor failed:', error);
    }
    closeTabContextMenu();
  }

  function deleteTabGroupById(groupId: string): void {
    const group = tabGroups.value.find((g) => g.id === groupId);
    if (!group) return;
    tabGroupDeleteOffer.value = { groupId, title: group.title };
    closeTabContextMenu();
  }

  async function confirmTabGroupDelete(): Promise<void> {
    if (!tabGroupDeleteOffer.value) return;
    tabGroupDeleteBusy.value = true;
    const { groupId } = tabGroupDeleteOffer.value;
    try {
      await deleteTabGroup(groupId);
      await loadTabGroups();
      options.onStatus('Tab group deleted');
      tabGroupDeleteOffer.value = null;
    } catch (error) {
      console.error('confirmTabGroupDelete failed:', error);
      options.onStatus('Failed to delete group');
    } finally {
      tabGroupDeleteBusy.value = false;
    }
  }

  function cancelTabGroupDelete(): void {
    tabGroupDeleteOffer.value = null;
  }

  async function toggleTabGroupCollapse(groupId: string, collapsed: boolean): Promise<void> {
    try {
      if (collapsed) {
        await collapseTabGroup(groupId);
      } else {
        await expandTabGroup(groupId);
      }
      await loadTabGroups();
    } catch (error) {
      console.error('toggleTabGroupCollapse failed:', error);
    }
    closeTabContextMenu();
  }

  const tabGroupDeleteTitle = computed(() => tabGroupDeleteOffer.value?.title ?? null);

  return {
    tabGroups,
    tabContextMenu,
    tabGroupEditOffer,
    tabGroupEditBusy,
    tabGroupDeleteOffer,
    tabGroupDeleteBusy,
    tabGroupDeleteTitle,
    sortedTabIds,
    sortedTabs,
    loadTabGroups,
    openTabContextMenu,
    closeTabContextMenu,
    newTabGroupFromTab,
    addTabToExistingGroup,
    removeTabGroupMembership,
    renameTabGroupPrompt,
    saveTabGroupEdit,
    cancelTabGroupEdit,
    cycleTabGroupColor,
    deleteTabGroupById,
    confirmTabGroupDelete,
    cancelTabGroupDelete,
    toggleTabGroupCollapse,
  };
}
