<script lang="ts">
  /**
   * Exodus Browser — vertical tab layout toggles in Settings.
   */
  import {
    loadVerticalTabSettings,
    saveVerticalTabSettings,
    type VerticalTabSettings,
  } from '$lib/verticalTabs';

  type Props = {
    onStatus: (message: string) => void;
    onLayoutChange?: (settings: VerticalTabSettings) => void;
  };

  let { onStatus, onLayoutChange }: Props = $props();

  let settings = $state<VerticalTabSettings | null>(null);
  let loaded = $state(false);

  async function load() {
    try {
      settings = await loadVerticalTabSettings();
      loaded = true;
      onLayoutChange?.(settings);
    } catch (error) {
      console.error('VerticalTabsSettings load failed:', error);
      onStatus('Failed to load vertical tab settings');
    }
  }

  async function persist() {
    if (!settings) return;
    try {
      await saveVerticalTabSettings(settings);
      onLayoutChange?.(settings);
      onStatus('Tab layout updated — reload may be needed for some tabs');
    } catch (error) {
      console.error('update_vertical_tab_settings failed:', error);
      onStatus('Failed to save tab layout');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

{#if loaded && settings}
  <div class="vt-settings" id="settings-section-tabs">
    <h3>Tab layout</h3>
    <p class="hint">Chrome-style vertical tabs along the side of the window.</p>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.enabled} onchange={() => void persist()} />
      <span>Vertical tabs</span>
    </label>
    {#if settings.enabled}
      <label>
        Position
        <select
          bind:value={settings.position}
          onchange={() => void persist()}
        >
          <option value="Left">Left</option>
          <option value="Right">Right</option>
        </select>
      </label>
      <label>
        Width mode
        <select bind:value={settings.width_mode} onchange={() => void persist()}>
          <option value="Auto">Auto</option>
          <option value="Fixed">Fixed</option>
          <option value="Compact">Compact</option>
        </select>
      </label>
      {#if settings.width_mode === 'Fixed' || settings.width_mode === 'fixed'}
        <label>
          Fixed width (px)
          <input
            type="number"
            min="160"
            max="400"
            bind:value={settings.fixed_width}
            onchange={() => void persist()}
          />
        </label>
      {/if}
      <label class="checkbox-row">
        <input
          type="checkbox"
          bind:checked={settings.collapse_inactive}
          onchange={() => void persist()}
        />
        <span>Collapse inactive tabs (stored per tab)</span>
      </label>
    {/if}
  </div>
{/if}

<style>
  .vt-settings h3 {
    margin: 0 0 8px;
    color: #e8e8e8;
  }

  .hint {
    font-size: 12px;
    color: #888;
    margin: 0 0 10px;
  }

  .vt-settings label {
    display: block;
    margin-bottom: 10px;
    font-size: 13px;
    color: #ccc;
  }

  .vt-settings select,
  .vt-settings input[type='number'] {
    display: block;
    width: 100%;
    margin-top: 4px;
    padding: 6px 8px;
    border-radius: 4px;
    border: 1px solid #505050;
    background: #1e1e1e;
    color: #e0e0e0;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
