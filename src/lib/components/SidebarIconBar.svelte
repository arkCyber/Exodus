<script lang="ts">
  /**
   * Firefox-style sidebar icon bar using Lucide icons
   */
  import type { SidebarPanel } from '$lib/browserTypes';
  import { Bot, Clock, Bookmark, Network, ChevronRight, ChevronLeft, X } from '@lucide/svelte';

  export let sidebarPanel: SidebarPanel;
  export let collapsed: boolean;
  export let onIconClick: (panel: SidebarPanel) => void;
  export let onToggleCollapse: () => void;
  export let onClose: () => void;

  const panels = [
    { id: 'ai' as SidebarPanel, icon: Bot, title: 'AI Chat' },
    { id: 'memory' as SidebarPanel, icon: Clock, title: 'History & Memory' },
    { id: 'bookmarks' as SidebarPanel, icon: Bookmark, title: 'Bookmarks' },
    { id: 'p2p' as SidebarPanel, icon: Network, title: 'P2P CDN' },
    { id: 'pocket' as SidebarPanel, icon: Bookmark, title: 'Pocket' },
  ];
</script>

<div class="icon-bar">
  <div class="icon-list">
    {#each panels as panel (panel.id)}
      <button
        type="button"
        class="icon-button"
        class:active={sidebarPanel === panel.id}
        title={panel.title}
        on:click={() => onIconClick(panel.id)}
      >
        <svelte:component this={panel.icon} size={20} />
      </button>
    {/each}
  </div>
  <div class="icon-footer">
    <button
      type="button"
      class="icon-button"
      title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      on:click={onToggleCollapse}
    >
      {#if collapsed}
        <ChevronRight size={20} />
      {:else}
        <ChevronLeft size={20} />
      {/if}
    </button>
    <button
      type="button"
      class="icon-button close-btn"
      title="Close sidebar"
      on:click={onClose}
    >
      <X size={20} />
    </button>
  </div>
</div>

<style>
  .icon-bar {
    width: 48px;
    background: #18181b;
    border-right: 1px solid #3f3f46;
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .icon-list {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 8px 4px;
    gap: 4px;
  }

  .icon-button {
    width: 40px;
    height: 40px;
    border: none;
    background: transparent;
    color: #a1a1aa;
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .icon-button:hover {
    background: #27272a;
    color: #e4e4e7;
  }

  .icon-button.active {
    background: #6366f1;
    color: #fff;
  }

  .icon-button.active:hover {
    background: #4f46e5;
  }

  .icon-footer {
    display: flex;
    flex-direction: column;
    padding: 8px 4px;
    gap: 4px;
    border-top: 1px solid #3f3f46;
  }

  .close-btn:hover {
    color: #ef4444;
    background: #27272a;
  }
</style>
