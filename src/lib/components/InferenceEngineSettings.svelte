<script lang="ts">
  /**
   * Exodus Browser — InferenceEngine registry UI (Tauri `inference_*` commands).
   */
  import { onMount } from 'svelte';
  import {
    inferenceChat,
    inferenceGenerate,
    inferenceGetConfig,
    inferenceGetLoadedModel,
    inferenceGetStats,
    inferenceGetStatus,
    inferenceListModels,
    inferenceLoadModel,
    inferenceUnloadModel,
    type InferenceModelInfo,
  } from '$lib/inferenceClient';

  type Props = {
    aiModel: string;
    onStatus: (message: string) => void;
  };

  let { aiModel, onStatus }: Props = $props();

  let models = $state<InferenceModelInfo[]>([]);
  let loadedModel = $state<string | null>(null);
  let engineStatus = $state('');
  let statsPreview = $state('');
  let configPreview = $state('');
  let selectedModel = $state('');
  let busy = $state(false);
  let testPrompt = $state('Hello from Exodus InferenceEngine');

  async function refresh() {
    busy = true;
    try {
      models = await inferenceListModels();
      loadedModel = await inferenceGetLoadedModel();
      engineStatus = await inferenceGetStatus();
      const stats = await inferenceGetStats();
      statsPreview = JSON.stringify(stats, null, 0).slice(0, 280);
      const cfg = await inferenceGetConfig();
      configPreview = `${cfg.backendType} · max ${cfg.maxTokens} tok · ${cfg.modelPath}`;
      if (!selectedModel && models.length > 0) {
        selectedModel =
          models.find((m) => m.name === aiModel)?.name ??
          models.find((m) => m.loaded)?.name ??
          models[0].name;
      }
    } catch (error) {
      console.error('[Exodus] inference refresh failed:', error);
      onStatus(`Inference engine: ${error}`);
    } finally {
      busy = false;
    }
  }

  async function loadSelected() {
    if (!selectedModel) return;
    busy = true;
    try {
      await inferenceLoadModel(selectedModel);
      onStatus(`Loaded model: ${selectedModel}`);
      await refresh();
    } catch (error) {
      console.error('[Exodus] inference_load_model failed:', error);
      onStatus(`Load model failed: ${error}`);
    } finally {
      busy = false;
    }
  }

  async function unload() {
    busy = true;
    try {
      await inferenceUnloadModel();
      onStatus('Model unloaded');
      await refresh();
    } catch (error) {
      console.error('[Exodus] inference_unload_model failed:', error);
      onStatus(`Unload failed: ${error}`);
    } finally {
      busy = false;
    }
  }

  async function testGenerate() {
    const model = selectedModel || aiModel || 'exodus-default';
    busy = true;
    try {
      const res = await inferenceGenerate({
        model,
        prompt: testPrompt,
        maxTokens: 64,
        stream: false,
      });
      if (res.success && res.text) {
        onStatus(`Generate OK: ${res.text.slice(0, 120)}`);
      } else {
        onStatus(`Generate: ${res.error ?? 'no text'}`);
      }
    } catch (error) {
      console.error('[Exodus] inference_generate failed:', error);
      onStatus(`Generate failed: ${error}`);
    } finally {
      busy = false;
    }
  }

  async function testChat() {
    const model = selectedModel || aiModel || 'exodus-default';
    busy = true;
    try {
      const res = await inferenceChat({
        model,
        messages: [{ role: 'user', content: testPrompt }],
        maxTokens: 64,
        stream: false,
      });
      if (res.success && res.text) {
        onStatus(`Chat OK: ${res.text.slice(0, 120)}`);
      } else {
        onStatus(`Chat: ${res.error ?? 'no text'}`);
      }
    } catch (error) {
      console.error('[Exodus] inference_chat failed:', error);
      onStatus(`Chat failed: ${error}`);
    } finally {
      busy = false;
    }
  }

  $effect(() => {
    if (aiModel && !selectedModel) {
      selectedModel = aiModel;
    }
  });

  onMount(() => {
    void refresh();
  });
</script>

<div class="inference-engine-block">
  <h4>Inference engine (Rust)</h4>
  <p class="settings-hint">
    Registry and routing via Tauri <code>inference_*</code> commands. Uses Allama HTTP when the service is online.
  </p>
  <p class="inference-meta">
    Status: <strong>{engineStatus || '—'}</strong>
    {#if loadedModel}
      · Loaded: <strong>{loadedModel}</strong>
    {/if}
  </p>
  {#if configPreview}
    <p class="inference-meta">{configPreview}</p>
  {/if}
  {#if statsPreview}
    <p class="inference-stats" title={statsPreview}>Stats: {statsPreview}</p>
  {/if}

  <label>
    Registry model
    <select bind:value={selectedModel} disabled={busy || models.length === 0}>
      {#if models.length === 0}
        <option value="">(no models registered)</option>
      {:else}
        {#each models as m (m.name)}
          <option value={m.name}>
            {m.name}{m.loaded ? ' · loaded' : ''} ({m.backend})
          </option>
        {/each}
      {/if}
    </select>
  </label>

  <label>
    Test prompt
    <input type="text" bind:value={testPrompt} disabled={busy} />
  </label>

  <div class="inference-actions">
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void refresh()}>
      Refresh
    </button>
    <button type="button" class="nav-button secondary" disabled={busy || !selectedModel} onclick={() => void loadSelected()}>
      Load model
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void unload()}>
      Unload
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void testGenerate()}>
      Test generate
    </button>
    <button type="button" class="nav-button secondary" disabled={busy} onclick={() => void testChat()}>
      Test chat
    </button>
  </div>
</div>

<style>
  .inference-engine-block {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid #333;
  }

  .inference-engine-block h4 {
    margin: 0 0 6px;
    font-size: 13px;
    color: #d1d5db;
  }

  .inference-meta {
    margin: 4px 0;
    font-size: 12px;
    color: #9ca3af;
  }

  .inference-stats {
    margin: 4px 0;
    font-size: 11px;
    color: #6b7280;
    word-break: break-all;
  }

  .inference-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 8px;
  }

  .inference-engine-block label {
    display: block;
    margin-top: 8px;
    font-size: 12px;
    color: #9ca3af;
  }

  .inference-engine-block select,
  .inference-engine-block input {
    display: block;
    width: 100%;
    margin-top: 4px;
    padding: 6px 8px;
    background: #1a1a1a;
    border: 1px solid #404040;
    color: #e5e7eb;
    border-radius: 6px;
    box-sizing: border-box;
  }
</style>
