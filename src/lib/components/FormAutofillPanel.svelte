<script lang="ts">
  /**
   * Exodus Browser — form autofill settings and saved entries (Settings).
   */
  import {
    fieldTypeKey,
    listFormAutofillEntries,
    loadFormAutofillSettings,
    removeFormAutofillEntry,
    saveFormAutofillSettings,
    type FormAutofillEntry,
    type FormAutofillSettings,
  } from '$lib/formAutofill';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let settings = $state<FormAutofillSettings | null>(null);
  let entries = $state<FormAutofillEntry[]>([]);
  let loaded = $state(false);

  async function load() {
    try {
      settings = await loadFormAutofillSettings();
      entries = await listFormAutofillEntries();
      loaded = true;
    } catch (error) {
      console.error('FormAutofillPanel load failed:', error);
      onStatus('Failed to load form autofill');
    }
  }

  async function persist() {
    if (!settings) return;
    try {
      await saveFormAutofillSettings(settings);
      onStatus('Form autofill settings saved');
    } catch (error) {
      console.error('update_autofill_settings failed:', error);
      onStatus('Failed to save form autofill settings');
    }
  }

  async function removeEntry(id: string) {
    try {
      await removeFormAutofillEntry(id);
      entries = entries.filter((e) => e.id !== id);
      onStatus('Autofill entry removed');
    } catch (error) {
      console.error('remove_autofill_entry failed:', error);
      onStatus('Failed to remove entry');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

{#if loaded && settings}
  <div class="form-autofill-panel" id="settings-section-autofill">
    <h3>Addresses &amp; more</h3>
    <p class="hint">Saves email, phone, and address fields when you fill forms (native webview).</p>

    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.enabled} onchange={() => void persist()} />
      <span>Enable form autofill</span>
    </label>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.save_addresses} onchange={() => void persist()} />
      <span>Save addresses and contact fields</span>
    </label>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={settings.autofill_on_load} onchange={() => void persist()} />
      <span>Autofill on page load</span>
    </label>

    {#if entries.length > 0}
      <ul class="entry-list">
        {#each entries.slice(0, 40) as entry (entry.id)}
          <li class="entry-row">
            <span class="entry-type">{fieldTypeKey(entry)}</span>
            <span class="entry-value">{entry.value}</span>
            <span class="entry-domain">{entry.domain}</span>
            <button type="button" class="link-btn" onclick={() => void removeEntry(entry.id)}>Remove</button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="hint">No saved form data yet.</p>
    {/if}
  </div>
{/if}

<style>
  .form-autofill-panel h3 {
    margin: 0 0 8px;
    color: #e8e8e8;
  }

  .hint {
    font-size: 12px;
    color: #888;
    margin: 0 0 10px;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 13px;
    color: #ccc;
  }

  .entry-list {
    list-style: none;
    margin: 12px 0 0;
    padding: 0;
    max-height: 160px;
    overflow-y: auto;
  }

  .entry-row {
    display: grid;
    grid-template-columns: 72px 1fr auto auto;
    gap: 8px;
    align-items: center;
    padding: 6px 0;
    border-bottom: 1px solid #3a3a3a;
    font-size: 12px;
  }

  .entry-type {
    color: #9cdcfe;
    text-transform: capitalize;
  }

  .entry-value {
    color: #e0e0e0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .entry-domain {
    color: #666;
    font-size: 11px;
  }

  .link-btn {
    background: none;
    border: none;
    color: #f87171;
    cursor: pointer;
    font-size: 11px;
  }
</style>
