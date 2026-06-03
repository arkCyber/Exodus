<script lang="ts">
  /**
   * Exodus Browser — web automation agent sidebar panel.
   */
  import { AGENT_PRESETS } from '$lib/agentActions';

  type Props = {
    command: string;
    log: string[];
    executing: boolean;
    domSummary: string;
    onExecute: () => void;
    onCompress: () => void;
    onBack: () => void;
    onPreset: (actionJson: string) => void;
    onCommandChange: (value: string) => void;
    onAskAi: () => void;
  };

  let {
    command = $bindable(''),
    log,
    executing,
    domSummary,
    onExecute,
    onCompress,
    onBack,
    onPreset,
    onCommandChange,
    onAskAi,
  }: Props = $props();
</script>

<div class="agent-panel">
  {#if domSummary}
    <p class="agent-dom-summary">{domSummary}</p>
  {/if}

  <div class="agent-quick-row">
    {#each AGENT_PRESETS as preset (preset.id)}
      <button
        type="button"
        class="agent-preset-btn"
        disabled={executing}
        onclick={() => onPreset(JSON.stringify(preset.action))}
      >
        {preset.label}
      </button>
    {/each}
  </div>

  <input
    type="text"
    value={command}
    oninput={(e) => onCommandChange(e.currentTarget.value)}
    placeholder='JSON, "scroll down", or ask: your question'
    class="agent-input"
    disabled={executing}
    onkeydown={(e) => e.key === 'Enter' && onExecute()}
  />

  <div class="agent-buttons">
    <button type="button" class="agent-btn-primary" disabled={executing} onclick={onExecute}>
      {executing ? 'Running…' : 'Run'}
    </button>
    <button type="button" class="agent-btn-small" disabled={executing} onclick={onCompress}>
      Compress DOM
    </button>
    <button type="button" class="agent-btn-small" disabled={executing} onclick={onAskAi}>
      Ask AI
    </button>
    <button type="button" class="agent-btn-small" onclick={onBack}>Back to AI</button>
  </div>

  <div class="agent-log" role="log" aria-live="polite">
    {#if log.length === 0}
      <p class="agent-log-empty">Run a preset or enter JSON. Example: scroll down, GetContent, ExtractLinks.</p>
    {:else}
      {#each log as logEntry}
        <div class="log-entry">{logEntry}</div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .agent-panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
    height: 100%;
    min-height: 0;
  }

  .agent-dom-summary {
    margin: 0;
    font-size: 12px;
    color: #9ca3af;
    padding: 6px 8px;
    background: #1a1a1a;
    border-radius: 6px;
    border: 1px solid #333;
  }

  .agent-quick-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .agent-preset-btn {
    background: #333;
    border: 1px solid #505050;
    color: #e0e0e0;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 11px;
    cursor: pointer;
  }

  .agent-preset-btn:hover:not(:disabled) {
    background: #454545;
  }

  .agent-preset-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .agent-input {
    background: #1a1a1a;
    border: 1px solid #404040;
    color: #e0e0e0;
    padding: 8px 12px;
    border-radius: 6px;
    width: 100%;
    box-sizing: border-box;
    font-size: 13px;
  }

  .agent-buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .agent-btn-primary {
    flex: 1;
    min-width: 80px;
    background: #2563eb;
    border: none;
    color: #fff;
    padding: 8px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }

  .agent-btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .agent-btn-small {
    background: #404040;
    border: none;
    color: #e0e0e0;
    padding: 6px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
  }

  .agent-log {
    flex: 1;
    background: #1a1a1a;
    border: 1px solid #404040;
    border-radius: 6px;
    padding: 12px;
    min-height: 120px;
    overflow-y: auto;
    font-family: ui-monospace, monospace;
    font-size: 11px;
  }

  .agent-log-empty {
    margin: 0;
    color: #666;
  }

  .log-entry {
    color: #aaa;
    margin-bottom: 6px;
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
