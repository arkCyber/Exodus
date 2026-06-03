<script lang="ts">
  /**
   * Exodus Browser — floating AI summary button for selected page text.
   */
  type Props = {
    visible: boolean;
    x: number;
    y: number;
    onSummarize: () => void;
  };

  let { visible, x, y, onSummarize }: Props = $props();
</script>

{#if visible}
  <div class="selection-popup" style="left: {x}px; top: {y}px;">
    <button type="button" class="summarize-button" onclick={onSummarize}>✨ AI Summary</button>
  </div>
{/if}

<style>
  .selection-popup {
    position: fixed;
    z-index: 1000;
    transform: translateX(-50%);
  }

  .summarize-button {
    background: #6366f1;
    border: none;
    color: white;
    padding: 8px 16px;
    border-radius: 20px;
    cursor: pointer;
    font-size: 13px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
</style>
