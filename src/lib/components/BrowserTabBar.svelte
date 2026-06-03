<script lang="ts">
  /**
   * Exodus Browser — tab strip with pin, close, and context menu.
   */
  import type { BrowserTab } from '$lib/browserTypes';
  import { faviconUrlFor } from '$lib/favicon';
  import { groupForTab, tabGroupColorCss, type TabGroup } from '$lib/tabGroups';

  type TabContextMenu = { tabId: string; x: number; y: number };

  type Props = {
    tabs: BrowserTab[];
    activeTabId: string;
    tabBarEl?: HTMLDivElement;
    tabContextMenu: TabContextMenu | null;
    sortedTabs: BrowserTab[];
    onSwitchTab: (id: string) => void;
    onNewTab: () => void;
    onCloseTab: (id: string, force?: boolean) => void;
    onTabMouseDown: (e: MouseEvent, id: string) => void;
    onTabContextMenu: (e: MouseEvent, id: string) => void;
    onCloseContextMenu: () => void;
    onTogglePin: (id: string) => void;
    onDuplicateTab: (id: string) => void;
    tabGroups?: TabGroup[];
    onNewTabGroup?: (tabId: string) => void;
    onAddTabToGroup?: (tabId: string, groupId: string) => void;
    onRemoveTabFromGroup?: (tabId: string) => void;
    onToggleGroupCollapse?: (groupId: string, collapsed: boolean) => void;
    onRenameTabGroup?: (groupId: string) => void;
    onCycleTabGroupColor?: (groupId: string) => void;
    onDeleteTabGroup?: (groupId: string) => void;
    /** Vertical tab strip along the left or right edge. */
    vertical?: boolean;
    verticalWidth?: number;
    verticalRight?: boolean;
  };

  let {
    tabs,
    activeTabId,
    tabBarEl = $bindable(undefined),
    tabContextMenu,
    sortedTabs,
    onSwitchTab,
    onNewTab,
    onCloseTab,
    onTabMouseDown,
    onTabContextMenu,
    onCloseContextMenu,
    onTogglePin,
    onDuplicateTab,
    tabGroups = [],
    onNewTabGroup,
    onAddTabToGroup,
    onRemoveTabFromGroup,
    onToggleGroupCollapse,
    onRenameTabGroup,
    onCycleTabGroupColor,
    onDeleteTabGroup,
    vertical = false,
    verticalWidth = 220,
    verticalRight = false,
  }: Props = $props();

  function tabGroup(tabId: string): TabGroup | undefined {
    return groupForTab(tabGroups, tabId);
  }
</script>

<div
  class="tab-bar"
  class:vertical
  class:vertical-right={vertical && verticalRight}
  style={vertical ? `--vt-width: ${verticalWidth}px` : undefined}
  bind:this={tabBarEl}
>
  {#each sortedTabs as tab (tab.id)}
    {@const grp = tabGroup(tab.id)}
    <button
      type="button"
      class="tab-item"
      class:active={tab.id === activeTabId}
      class:pinned={tab.pinned}
      class:has-group={!!grp}
      style={grp ? `--tab-group-color: ${tabGroupColorCss(grp.color)}` : undefined}
      onclick={() => onSwitchTab(tab.id)}
      onmousedown={(e) => onTabMouseDown(e, tab.id)}
      oncontextmenu={(e) => onTabContextMenu(e, tab.id)}
      title={tab.pinned ? 'Pinned · Right-click for menu' : 'Right-click for menu · Middle-click to close'}
    >
      {#if tab.pinned}
        <span class="tab-pin" aria-hidden="true">📌</span>
      {:else if faviconUrlFor(tab.url)}
        <img class="tab-favicon" src={tab.favicon ?? faviconUrlFor(tab.url)} alt="" width="16" height="16" />
      {/if}
      <span class="tab-title">{tab.title || 'New Tab'}</span>
      {#if tabs.length > 1 && !tab.pinned}
        <span
          class="tab-close"
          role="button"
          tabindex="0"
          onclick={(e) => {
            e.stopPropagation();
            onCloseTab(tab.id);
          }}
          onkeydown={(e) => e.key === 'Enter' && onCloseTab(tab.id)}
        >×</span>
      {/if}
    </button>
  {/each}
  <button type="button" class="tab-new" onclick={onNewTab} title="New tab (⌘T)">+</button>
</div>

{#if tabContextMenu}
  <button type="button" class="menu-backdrop" aria-label="Close" onclick={onCloseContextMenu}></button>
  <div class="tab-context-menu" style="left: {tabContextMenu.x}px; top: {tabContextMenu.y}px;">
    <button type="button" class="menu-item" onclick={() => onTogglePin(tabContextMenu.tabId)}>
      {tabs.find((t) => t.id === tabContextMenu?.tabId)?.pinned ? 'Unpin tab' : 'Pin tab'}
    </button>
    {#if onNewTabGroup}
      <button type="button" class="menu-item" onclick={() => onNewTabGroup(tabContextMenu.tabId)}>
        New tab group
      </button>
    {/if}
    {#if onAddTabToGroup && tabGroups.length > 0}
      {#each tabGroups as g (g.id)}
        <button
          type="button"
          class="menu-item menu-sub"
          onclick={() => onAddTabToGroup(tabContextMenu.tabId, g.id)}
        >
          Add to · {g.title}
        </button>
      {/each}
    {/if}
    {#if onRemoveTabFromGroup && groupForTab(tabGroups, tabContextMenu.tabId)}
      <button type="button" class="menu-item" onclick={() => onRemoveTabFromGroup(tabContextMenu.tabId)}>
        Remove from group
      </button>
      {#if onToggleGroupCollapse}
        {@const g = groupForTab(tabGroups, tabContextMenu.tabId)}
        {#if g}
          <button
            type="button"
            class="menu-item"
            onclick={() => onToggleGroupCollapse(g.id, !g.collapsed)}
          >
            {g.collapsed ? 'Expand group' : 'Collapse group'}
          </button>
          {#if onRenameTabGroup}
            <button type="button" class="menu-item" onclick={() => onRenameTabGroup(g.id)}>
              Rename group
            </button>
          {/if}
          {#if onCycleTabGroupColor}
            <button type="button" class="menu-item" onclick={() => onCycleTabGroupColor(g.id)}>
              Change group color
            </button>
          {/if}
          {#if onDeleteTabGroup}
            <button type="button" class="menu-item" onclick={() => onDeleteTabGroup(g.id)}>
              Delete group
            </button>
          {/if}
        {/if}
      {/if}
    {/if}
    <button
      type="button"
      class="menu-item"
      onclick={() => {
        onCloseContextMenu();
        onDuplicateTab(tabContextMenu.tabId);
      }}
    >
      Duplicate tab
    </button>
    <button
      type="button"
      class="menu-item"
      onclick={() => {
        onCloseContextMenu();
        onCloseTab(tabContextMenu.tabId, true);
      }}
    >
      Close tab
    </button>
  </div>
{/if}

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px 0;
    background: #252525;
    border-bottom: 1px solid #404040;
    overflow-x: auto;
    flex-shrink: 0;
  }

  .tab-bar.vertical {
    flex-direction: column;
    align-items: stretch;
    width: var(--vt-width, 220px);
    min-width: var(--vt-width, 220px);
    max-width: var(--vt-width, 220px);
    height: 100%;
    padding: 8px 6px;
    overflow-x: hidden;
    overflow-y: auto;
    border-bottom: none;
    border-right: 1px solid #404040;
    order: 0;
  }

  .tab-bar.vertical.vertical-right {
    border-right: none;
    border-left: 1px solid #404040;
    order: 2;
  }

  .tab-bar.vertical .tab-item {
    max-width: none;
    width: 100%;
    border-radius: 8px;
    border-bottom: 1px solid #404040;
    flex-shrink: 0;
  }

  .tab-bar.vertical .tab-item.active {
    max-width: none;
  }

  .tab-bar.vertical .tab-new {
    width: 100%;
    margin-top: 4px;
  }

  .tab-item {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    max-width: 200px;
    flex-shrink: 1;
    padding: 6px 10px;
    background: #333;
    border: 1px solid #404040;
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    color: #ccc;
    cursor: pointer;
    font-size: 12px;
  }

  .tab-item.active {
    background: #2d2d2d;
    color: #fff;
    border-color: #6366f1;
    max-width: 260px;
    flex-shrink: 0;
  }

  .tab-item:not(.active) {
    max-width: 160px;
  }

  .tab-item.pinned {
    min-width: 48px;
    max-width: 180px;
    flex-shrink: 0;
    background: #2a2a35;
  }

  .tab-item.has-group {
    border-left: 3px solid var(--tab-group-color, #6b7280);
    padding-left: 10px;
  }

  .menu-sub {
    padding-left: 20px;
    font-size: 12px;
  }

  .tab-pin {
    font-size: 12px;
    line-height: 1;
    flex-shrink: 0;
  }

  .tab-favicon {
    border-radius: 2px;
    flex-shrink: 0;
  }

  .tab-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tab-close {
    opacity: 0.7;
    padding: 0 4px;
  }

  .tab-close:hover {
    opacity: 1;
    color: #f87171;
  }

  .tab-new {
    background: transparent;
    border: 1px dashed #555;
    color: #aaa;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    flex-shrink: 0;
    font-size: 18px;
  }

  .menu-backdrop {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: transparent;
    border: none;
    cursor: default;
  }

  .tab-context-menu {
    position: fixed;
    z-index: 3000;
    min-width: 160px;
    background: #2d2d2d;
    border: 1px solid #555;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
    padding: 4px;
  }

  .menu-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: #eee;
    cursor: pointer;
    border-radius: 6px;
    font-size: 13px;
  }

  .menu-item:hover {
    background: #404040;
  }
</style>
