<script lang="ts">
  /**
   * Exodus Browser — Safe Browsing warning before proceeding to a risky URL.
   */

  type Offer = {
    url: string;
    reason: string;
  };

  type Props = {
    offer: Offer | null;
    onProceed: () => void;
    onCancel: () => void;
  };

  let { offer, onProceed, onCancel }: Props = $props();
</script>

{#if offer}
  <button type="button" class="prompt-backdrop" aria-label="Close warning" onclick={onCancel}></button>
  <div class="prompt-dialog" role="alertdialog" aria-labelledby="sb-title">
    <h3 id="sb-title">Security warning</h3>
    <p class="reason">{offer.reason}</p>
    <p class="url">{offer.url}</p>
    <div class="prompt-actions">
      <button type="button" class="btn secondary" onclick={onCancel}>Go back</button>
      <button type="button" class="btn danger" onclick={onProceed}>Proceed anyway</button>
    </div>
  </div>
{/if}

<style>
  .prompt-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10001;
    background: rgba(0, 0, 0, 0.55);
    border: none;
    cursor: default;
  }

  .prompt-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 10002;
    width: min(440px, 92vw);
    background: #2d2d2d;
    border: 1px solid #b45309;
    border-radius: 12px;
    padding: 20px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.55);
  }

  .prompt-dialog h3 {
    margin: 0 0 12px;
    color: #fbbf24;
    font-size: 18px;
  }

  .reason {
    margin: 0 0 8px;
    color: #e0e0e0;
    font-size: 14px;
    line-height: 1.4;
  }

  .url {
    margin: 0 0 16px;
    font-size: 12px;
    color: #9ca3af;
    word-break: break-all;
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
    background: #b45309;
    color: #fff;
  }
</style>
