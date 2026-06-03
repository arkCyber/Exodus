<script lang="ts">
  /**
   * Exodus Browser — Allama inference microservice (Ollama replacement, port 11435).
   */
  import { invoke } from '@tauri-apps/api/core';
  import { allamaHealth } from '$lib/allamaClient';

  type AllamaStatusDto = {
    state: string;
    port: number;
    detail: string;
    endpointOnline: boolean;
    mode: string;
    modelsRegistered: number;
    binaryPath?: string | null;
  };

  type Props = {
    spawnAllama: boolean;
    aiPort: number;
    onStatus: (message: string) => void;
    onSpawnAllamaChange: (enabled: boolean) => void;
    onAiPortChange: (port: number) => void;
  };

  let {
    spawnAllama,
    aiPort,
    onStatus,
    onSpawnAllamaChange,
    onAiPortChange,
  }: Props = $props();

  let status = $state<AllamaStatusDto | null>(null);
  let busy = $state(false);
  let httpProbe = $state<'idle' | 'ok' | 'fail'>('idle');

  async function refresh() {
    try {
      status = await invoke<AllamaStatusDto>('allama_service_status');
    } catch (error) {
      console.error('allama_service_status failed:', error);
    }
  }

  async function start() {
    busy = true;
    try {
      status = await invoke<AllamaStatusDto>('allama_service_start');
      onStatus('Allama started');
    } catch (error) {
      console.error('allama_service_start failed:', error);
      onStatus('Failed to start Allama');
    } finally {
      busy = false;
    }
  }

  async function stop() {
    busy = true;
    try {
      await invoke('allama_service_stop');
      await refresh();
      onStatus('Allama stopped');
    } catch (error) {
      console.error('allama_service_stop failed:', error);
      onStatus('Failed to stop Allama');
    } finally {
      busy = false;
    }
  }

  async function testHttp() {
    busy = true;
    httpProbe = 'idle';
    try {
      const ok = await allamaHealth(aiPort);
      httpProbe = ok ? 'ok' : 'fail';
      onStatus(ok ? `HTTP OK on port ${aiPort}` : `HTTP unreachable on port ${aiPort}`);
    } catch (error) {
      console.error('allama HTTP probe failed:', error);
      httpProbe = 'fail';
      onStatus('HTTP probe failed');
    } finally {
      busy = false;
    }
  }

  async function restart() {
    busy = true;
    try {
      status = await invoke<AllamaStatusDto>('allama_service_restart');
      onStatus('Allama restarted');
    } catch (error) {
      console.error('allama_service_restart failed:', error);
      onStatus('Failed to restart Allama');
    } finally {
      busy = false;
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void refresh();
  });
</script>

<div class="allama-settings" id="settings-section-allama">
  <h3>Allama inference (replaces Ollama)</h3>
  <p class="hint">
    Unified local LLM API on port <strong>11435</strong> — Ollama-compatible <code>/api/*</code> and OpenAI
    <code>/v1/*</code>. Uses native <code>allama serve</code> when built, otherwise Exodus embedded gateway.
  </p>

  <label class="checkbox-row">
    <input
      type="checkbox"
      checked={spawnAllama}
      onchange={(e) => onSpawnAllamaChange(e.currentTarget.checked)}
    />
    <span>Start Allama with the app</span>
  </label>

  <label>
    AI port
    <input
      type="number"
      min="1024"
      max="65535"
      value={aiPort}
      onchange={(e) => onAiPortChange(Number(e.currentTarget.value) || 11435)}
    />
  </label>

  {#if status}
    <div class="status-box">
      <span class="state" class:online={status.endpointOnline}>{status.state}</span>
      <span class="detail">{status.detail}</span>
      <span class="meta">
        {status.modelsRegistered} model(s) · mode: {status.mode}
        {#if status.binaryPath}
          · binary: {status.binaryPath}
        {/if}
      </span>
    </div>
  {/if}

  <div class="actions">
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void refresh()}>
      Refresh
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void start()}>
      Start
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void stop()}>
      Stop
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void restart()}>
      Restart
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void testHttp()}>
      Test HTTP
    </button>
  </div>
  {#if httpProbe === 'ok'}
    <p class="http-probe ok">GET /api/tags succeeded on port {aiPort}</p>
  {:else if httpProbe === 'fail'}
    <p class="http-probe fail">Could not reach Allama on port {aiPort}</p>
  {/if}
</div>

<style>
  .allama-settings h3 {
    margin: 0 0 8px;
    color: #e8e8e8;
  }

  .hint {
    font-size: 12px;
    color: #888;
    margin: 0 0 12px;
    line-height: 1.45;
  }

  .hint code {
    color: #9cdcfe;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    font-size: 13px;
    color: #ccc;
  }

  label {
    display: block;
    margin-bottom: 12px;
    font-size: 13px;
    color: #ccc;
  }

  label input {
    display: block;
    width: 100%;
    margin-top: 4px;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid #505050;
    background: #1e1e1e;
    color: #e0e0e0;
  }

  .status-box {
    padding: 10px;
    margin-bottom: 12px;
    background: #252525;
    border-radius: 8px;
    border: 1px solid #404040;
    font-size: 12px;
  }

  .state {
    display: block;
    font-weight: 600;
    color: #f87171;
    margin-bottom: 4px;
  }

  .state.online {
    color: #4ade80;
  }

  .detail {
    display: block;
    color: #ccc;
    margin-bottom: 4px;
  }

  .meta {
    color: #666;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .nav-button {
    padding: 6px 12px;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    font-size: 13px;
  }

  .nav-button.secondary {
    background: #404040;
    color: #e0e0e0;
  }

  .nav-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .http-probe {
    margin: 8px 0 0;
    font-size: 12px;
  }

  .http-probe.ok {
    color: #4ade80;
  }

  .http-probe.fail {
    color: #f87171;
  }
</style>
