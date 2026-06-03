<script lang="ts">
  /**
   * Exodus Browser — edit tab group name and color.
   */
  import { TAB_GROUP_COLORS, tabGroupColorCss } from '$lib/tabGroups';

  export type TabGroupEditOffer = {
    groupId: string;
    title: string;
    color: string;
  };

  type Props = {
    offer: TabGroupEditOffer | null;
    busy?: boolean;
    onSave: (title: string, color: string) => void | Promise<void>;
    onCancel: () => void;
  };

  let { offer, busy = false, onSave, onCancel }: Props = $props();

  let titleInput = $state('');
  let colorInput = $state('blue');

  $effect(() => {
    if (offer) {
      titleInput = offer.title;
      colorInput = offer.color.toLowerCase();
    }
  });
</script>

{#if offer}
  <button type="button" class="prompt-backdrop" aria-label="Cancel" onclick={onCancel}></button>
  <div class="prompt-dialog" role="dialog" aria-labelledby="tg-edit-title">
    <h3 id="tg-edit-title">Edit tab group</h3>
    <label class="field">
      <span>Name</span>
      <input type="text" bind:value={titleInput} />
    </label>
    <p class="color-label">Color</p>
    <div class="color-row">
      {#each TAB_GROUP_COLORS as c (c)}
        <button
          type="button"
          class="color-swatch"
          class:selected={colorInput === c}
          style="--swatch: {tabGroupColorCss(c)}"
          title={c}
          onclick={() => (colorInput = c)}
        ></button>
      {/each}
    </div>
    <div class="prompt-actions">
      <button type="button" class="btn secondary" disabled={busy} onclick={onCancel}>Cancel</button>
      <button
        type="button"
        class="btn primary"
        disabled={busy || !titleInput.trim()}
        onclick={() => void onSave(titleInput.trim(), colorInput)}
      >
        {busy ? 'Saving…' : 'Save'}
      </button>
    </div>
  </div>
{/if}

<style>
  .prompt-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10001;
    background: rgba(0, 0, 0, 0.5);
    border: none;
    cursor: default;
  }

  .prompt-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 10002;
    width: min(360px, 92vw);
    background: #2d2d2d;
    border: 1px solid #505050;
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.55);
  }

  .prompt-dialog h3 {
    margin: 0 0 16px;
    font-size: 18px;
    color: #f0f0f0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 14px;
    font-size: 12px;
    color: #aaa;
  }

  .field input {
    padding: 8px 10px;
    border-radius: 6px;
    border: 1px solid #505050;
    background: #1e1e1e;
    color: #e0e0e0;
    font-size: 14px;
  }

  .color-label {
    margin: 0 0 8px;
    font-size: 12px;
    color: #aaa;
  }

  .color-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
  }

  .color-swatch {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
    background: var(--swatch);
    cursor: pointer;
    padding: 0;
  }

  .color-swatch.selected {
    border-color: #fff;
    box-shadow: 0 0 0 2px #6366f1;
  }

  .prompt-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 14px;
  }

  .btn.secondary {
    background: #404040;
    color: #e0e0e0;
  }

  .btn.primary {
    background: #6366f1;
    color: #fff;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
