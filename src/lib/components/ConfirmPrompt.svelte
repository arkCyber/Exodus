<script lang="ts">
  /**
   * Exodus Browser — reusable confirmation dialog (replaces window.confirm).
   */

  export type ConfirmOffer = {
    title: string;
    message: string;
    confirmLabel?: string;
    cancelLabel?: string;
    danger?: boolean;
  };

  type Props = {
    offer: ConfirmOffer | null;
    busy?: boolean;
    onConfirm: () => void | Promise<void>;
    onCancel: () => void;
  };

  let { offer, busy = false, onConfirm, onCancel }: Props = $props();
</script>

{#if offer}
  <button type="button" class="prompt-backdrop" aria-label="Cancel" onclick={onCancel}></button>
  <div class="prompt-dialog" role="alertdialog" aria-labelledby="confirm-title">
    <h3 id="confirm-title">{offer.title}</h3>
    <p>{offer.message}</p>
    <div class="prompt-actions">
      <button type="button" class="btn secondary" disabled={busy} onclick={onCancel}>
        {offer.cancelLabel ?? 'Cancel'}
      </button>
      <button
        type="button"
        class="btn"
        class:danger={offer.danger !== false}
        class:primary={offer.danger === false}
        disabled={busy}
        onclick={() => void onConfirm()}
      >
        {busy ? 'Working…' : (offer.confirmLabel ?? 'Confirm')}
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
    width: min(400px, 92vw);
    background: #2d2d2d;
    border: 1px solid #505050;
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.55);
  }

  .prompt-dialog h3 {
    margin: 0 0 12px;
    color: #f0f0f0;
    font-size: 18px;
  }

  .prompt-dialog p {
    margin: 0 0 16px;
    color: #ccc;
    font-size: 14px;
    line-height: 1.45;
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

  .btn.primary {
    background: #6366f1;
    color: #fff;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
