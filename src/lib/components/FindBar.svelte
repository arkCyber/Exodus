<script lang="ts">
  /**
   * Exodus Browser — in-page find bar (⌘F).
   */
  type Props = {
    open: boolean;
    findQuery: string;
    findResults: number;
    currentFindIndex: number;
    onFindInput: () => void;
    onFind: (direction?: 'next' | 'prev') => void;
    onClose: () => void;
  };

  let {
    open,
    findQuery = $bindable(''),
    findResults,
    currentFindIndex,
    onFindInput,
    onFind,
    onClose,
  }: Props = $props();
</script>

{#if open}
  <div class="find-bar exodus-find-bar" role="search">
    <input
      type="text"
      class="find-input"
      bind:value={findQuery}
      oninput={onFindInput}
      onkeydown={(e) => e.key === 'Enter' && onFind()}
      placeholder="Find in page..."
      aria-label="Find in page"
    />
    <span class="find-count" aria-live="polite">
      {findResults > 0 ? `${currentFindIndex}/${findResults}` : '0/0'}
    </span>
    <button type="button" class="find-btn" onclick={() => onFind('prev')} title="Previous" aria-label="Previous match">▲</button>
    <button type="button" class="find-btn" onclick={() => onFind('next')} title="Next" aria-label="Next match">▼</button>
    <button type="button" class="find-btn close" onclick={onClose} title="Close" aria-label="Close find bar">×</button>
  </div>
{/if}

<style>
  .find-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: #2d2d30;
    border-bottom: 1px solid #3c4043;
  }

  .find-input {
    flex: 1;
    background: #202124;
    border: 1px solid #5f6368;
    color: #e8eaed;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 13px;
  }

  .find-input:focus {
    outline: none;
    border-color: #8ab4f8;
  }

  .find-count {
    color: #9aa0a6;
    font-size: 12px;
    min-width: 60px;
    text-align: center;
  }

  .find-btn {
    background: #3c4043;
    border: 1px solid #5f6368;
    color: #e8eaed;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .find-btn:hover {
    background: #4a4d51;
    border-color: #8ab4f8;
  }

  .find-btn.close {
    background: #3c4043;
    color: #9aa0a6;
  }

  .find-btn.close:hover {
    background: #5f6368;
    color: #e8eaed;
  }
</style>
