<!--
  Exodus Browser — vertical tabs inside Firefox-style sidebar.
-->
<template>
  <div class="sidebar-vtabs">
    <BrowserTabBar
      :tabs="tabs"
      :active-tab-id="activeTabId"
      :sorted-tabs="sortedTabs"
      :tab-context-menu="tabContextMenu"
      :tab-groups="tabGroups"
      vertical
      vertical-in-sidebar
      :vertical-width="stripWidth"
      :vertical-right="false"
      v-on="tabBarHandlers"
    />
    <p class="muted sidebar-vtabs-hint">
      Vertical tabs are in the sidebar. Change in <strong>Customize sidebar</strong> (gear).
    </p>
  </div>
</template>

<script setup lang="ts">
import BrowserTabBar from '@/components/BrowserTabBar.vue';
import type { BrowserTab } from '$lib/browserTypes';

withDefaults(
  defineProps<{
    tabs: BrowserTab[];
    activeTabId: string | null;
    sortedTabs?: BrowserTab[];
    stripWidth?: number;
    tabContextMenu?: { tabId: string; x: number; y: number } | null;
    tabGroups?: import('$lib/tabGroups').TabGroup[];
    tabBarHandlers?: import('@/composables/useBrowserTabBarHandlers').TabBarHandlerMap;
  }>(),
  {
    sortedTabs: () => [],
    stripWidth: 280,
    tabContextMenu: null,
    tabGroups: () => [],
    tabBarHandlers: undefined,
  },
);

/** Events forwarded via tabBarHandlers when provided by parent. */
defineEmits<{
  switchTab: [id: string];
  newTab: [];
  closeTab: [id: string, force?: boolean];
}>();
</script>

<style scoped>
.sidebar-vtabs {
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
}

.sidebar-vtabs :deep(.tab-bar.vertical) {
  flex: 1;
  min-height: 120px;
  max-height: none;
  border: none;
  background: transparent;
}

.sidebar-vtabs-hint {
  margin: 8px 0 0;
  font-size: 11px;
  padding: 0 4px;
}
</style>
