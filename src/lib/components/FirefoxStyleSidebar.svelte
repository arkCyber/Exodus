<script lang="ts">
  /**
   * Firefox-style sidebar with icon bar and content panel
   */
  import type { SidebarPanel } from '$lib/browserTypes';
  import SidebarIconBar from './SidebarIconBar.svelte';
  import SidebarContentPanel from './SidebarContentPanel.svelte';

  export let open: boolean;
  export let sidebarPanel: SidebarPanel;
  export let onClose: () => void;
  export let onOpenPanel: (panel: SidebarPanel) => void;

  let collapsed = false;
  let contentWidth = 320;

  function toggleCollapse() {
    collapsed = !collapsed;
  }

  function handleIconClick(panel: SidebarPanel) {
    if (sidebarPanel === panel && !collapsed) {
      collapsed = true;
    } else {
      collapsed = false;
      onOpenPanel(panel);
    }
  }
</script>

{#if open}
  <aside class="firefox-sidebar" class:collapsed>
    <SidebarIconBar
      {sidebarPanel}
      {collapsed}
      onIconClick={handleIconClick}
      onToggleCollapse={toggleCollapse}
      onClose={onClose}
    />
    {#if !collapsed}
      <SidebarContentPanel
        {sidebarPanel}
        {contentWidth}
      />
    {/if}
  </aside>
{/if}

<style>
  .firefox-sidebar {
    display: flex;
    background: #1c1c1c;
    border-left: 1px solid #3a3a3a;
    height: 100%;
    transition: width 0.2s ease;
  }

  .firefox-sidebar.collapsed {
    width: 48px;
  }

  .firefox-sidebar:not(.collapsed) {
    width: calc(48px + 320px);
  }
</style>
