<script lang="ts">
  /**
   * Exodus Browser — confirm tab group deletion.
   */

  type Props = {
    groupTitle: string | null;
    busy?: boolean;
    onConfirm: () => void | Promise<void>;
    onCancel: () => void;
  };

  let { groupTitle, busy = false, onConfirm, onCancel }: Props = $props();
</script>

{#if groupTitle !== null}
  <button type="button" class="prompt-backdrop" aria-label="Cancel" onclick={onCancel}></button>
  <div class="prompt-dialog" role="alertdialog" aria-labelledby="tg-del-title">
    <h3 id="tg-del-title">Delete tab group?</h3>
    <p>Delete <strong>{groupTitle}</strong>? Open tabs will stay open.</p>
    <div class="prompt-actions">
      <button type="button" class="btn secondary" disabled={busy} onclick={onCancel}>Cancel</button>
      <button type="button" class="btn danger" disabled={busy} onclick={() => void onConfirm()}>
        {busy ? 'Deleting…' : 'Delete'}
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
  }

  .prompt-dialog h3 {
    margin: 0 0 12px;
    color: #f0f0f0;
  }

  .prompt-dialog p {
    margin: 0 0 16px;
    color: #ccc;
    font-size: 14px;
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

  .btn.danger {
    background: #dc2626;
    color: #fff;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
