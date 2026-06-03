<script lang="ts">
  /**
   * Exodus Browser — modal shell for bookmarks / history / downloads panels.
   */
  import type { Snippet } from 'svelte';

  type Props = {
    open: boolean;
    title: string;
    onClose: () => void;
    children: Snippet;
  };

  let { open, title, onClose, children }: Props = $props();
</script>

{#if open}
  <div
    class="panel-modal exodus-panel-modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="panel-title"
    tabindex="-1"
    onclick={(e) => e.target === e.currentTarget && onClose()}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  >
    <div>
      <div class="panel-header">
        <h2 id="panel-title">{title}</h2>
        <button type="button" class="close-btn" onclick={onClose} aria-label="Close">×</button>
      </div>
      <div class="panel-content">
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .panel-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .panel-modal > div {
    background: #292a2d;
    border: 1px solid #5f6368;
    border-radius: 8px;
    width: 600px;
    max-width: 90vw;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid #5f6368;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 500;
    color: #e8eaed;
  }

  .panel-content {
    padding: 24px;
    overflow-y: auto;
  }
</style>
