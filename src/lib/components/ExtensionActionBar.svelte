<script lang="ts">
  /**
   * Exodus Browser — extension toolbar action buttons (MV3 `action` / popup).
   */
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listExtensions } from '$lib/extensions/api';
  import type { ExtensionInfo } from '$lib/extensions/types';
  import { canUseNativeWebview } from '$lib/exodusBrowser';

  type Props = {
    contentHost: HTMLElement | undefined;
  };

  let { contentHost }: Props = $props();

  let extensions = $state<ExtensionInfo[]>([]);
  let openPopupId = $state<string | null>(null);

  async function refresh() {
    try {
      extensions = (await listExtensions()).filter((e) => e.enabled && e.actionPopup);
    } catch (error) {
      console.error('extension_list failed:', error);
    }
  }

  function popupLabel(extId: string): string {
    return `exodus-ext-popup-${extId}`;
  }

  async function closePopup() {
    await closePopupWindow();
  }

  /** Open extension action popup in a dedicated window (chrome.action). */
  async function openPopup(ext: ExtensionInfo) {
    if (!canUseNativeWebview()) return;
    if (openPopupId === ext.id) {
      await closePopup();
      return;
    }
    await closePopup();
    try {
      await invoke('extension_open_popup_window', { extensionId: ext.id });
      openPopupId = ext.id;
    } catch (error) {
      console.error('extension_open_popup_window failed:', error);
    }
  }

  async function closePopupWindow() {
    if (!openPopupId) return;
    try {
      await invoke('extension_close_popup_window', { extensionId: openPopupId });
    } catch {
      /* ignore */
    }
    openPopupId = null;
  }

  onMount(() => {
    void refresh();
  });
</script>

{#if extensions.length > 0}
  <div class="extension-action-bar" role="toolbar" aria-label="Extensions">
    {#each extensions as ext (ext.id)}
      <button
        type="button"
        class="extension-action-btn"
        class:active={openPopupId === ext.id}
        title={ext.name}
        onclick={() => void openPopup(ext)}
      >
        {ext.name.slice(0, 1).toUpperCase()}
      </button>
    {/each}
  </div>
{/if}

<style>
  .extension-action-bar {
    display: flex;
    gap: 4px;
    padding: 4px 8px;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle, #333);
    background: var(--chrome-bg, #1e1e1e);
  }

  .extension-action-btn {
    min-width: 28px;
    height: 28px;
    padding: 0 6px;
    border-radius: 6px;
    border: 1px solid #444;
    background: #2a2a2a;
    color: #ddd;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .extension-action-btn:hover {
    background: #383838;
  }

  .extension-action-btn.active {
    background: #4a6cf7;
    border-color: #6b8cff;
    color: #fff;
  }
</style>
