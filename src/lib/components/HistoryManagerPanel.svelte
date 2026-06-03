<script lang="ts">
  /**
   * Exodus Browser — full history manager UI in Settings.
   */
  import {
    clearAllManagedHistory,
    getManagedHistoryStats,
    getRecentManagedHistory,
    loadHistoryManagerSettings,
    removeManagedHistoryByDomain,
    removeManagedHistoryEntry,
    saveHistoryManagerSettings,
    searchManagedHistory,
    type HistoryManagerSettings,
    type ManagedHistoryEntry,
  } from '$lib/historyManager';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let entries = $state<ManagedHistoryEntry[]>([]);
  let settings = $state<HistoryManagerSettings | null>(null);
  let stats = $state<Record<string, number>>({});
  let searchQuery = $state('');
  let loading = $state(true);
  let confirmClear = $state(false);

  function formatTime(ts: number): string {
    if (!ts) return '';
    try {
      return new Date(ts * 1000).toLocaleString();
    } catch {
      return String(ts);
    }
  }

  async function load() {
    loading = true;
    try {
      const [recent, s, st] = await Promise.all([
        getRecentManagedHistory(80),
        loadHistoryManagerSettings(),
        getManagedHistoryStats(),
      ]);
      entries = recent;
      settings = s;
      stats = st;
    } catch (error) {
      console.error('HistoryManagerPanel load failed:', error);
      onStatus('Failed to load history');
    } finally {
      loading = false;
    }
  }

  async function runSearch() {
    const q = searchQuery.trim();
    if (!q) {
      entries = await getRecentManagedHistory(80);
      return;
    }
    entries = await searchManagedHistory(q);
  }

  async function persistSettings() {
    if (!settings) return;
    try {
      await saveHistoryManagerSettings(settings);
      onStatus('History settings saved');
    } catch (error) {
      console.error('update_history_settings failed:', error);
      onStatus('Failed to save history settings');
    }
  }

  async function deleteEntry(id: string) {
    try {
      await removeManagedHistoryEntry(id);
      entries = entries.filter((e) => e.id !== id);
      onStatus('History entry removed');
    } catch (error) {
      console.error('remove_history_entry failed:', error);
      onStatus('Failed to remove entry');
    }
  }

  async function deleteDomain(domain: string) {
    try {
      await removeManagedHistoryByDomain(domain);
      await load();
      onStatus(`Removed history for ${domain}`);
    } catch (error) {
      console.error('remove_history_by_domain failed:', error);
      onStatus('Failed to remove domain history');
    }
  }

  async function handleClearAll() {
    if (!confirmClear) {
      confirmClear = true;
      return;
    }
    try {
      await clearAllManagedHistory();
      entries = [];
      confirmClear = false;
      onStatus('Full history cleared');
      stats = await getManagedHistoryStats();
    } catch (error) {
      console.error('clear_all_history failed:', error);
      onStatus('Failed to clear history');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

<div class="history-panel" id="settings-section-history">
  <h3>Browsing history (full)</h3>
  <p class="hint">
    Chrome-style history store (separate from the visit list in the sidebar). Visits are recorded on each navigation.
  </p>

  {#if settings}
    <div class="settings-toggles">
      <label class="checkbox-row">
        <input
          type="checkbox"
          bind:checked={settings.enabled}
          onchange={() => void persistSettings()}
        />
        <span>Enable history manager</span>
      </label>
      <label class="checkbox-row">
        <input
          type="checkbox"
          bind:checked={settings.remember_browsing}
          onchange={() => void persistSettings()}
        />
        <span>Remember browsing</span>
      </label>
      <label>
        Retention (days, 0 = forever)
        <input
          type="number"
          min="0"
          max="3650"
          bind:value={settings.retention_days}
          onchange={() => void persistSettings()}
        />
      </label>
    </div>
  {/if}

  {#if stats.total_entries !== undefined}
    <p class="stats-line">
      {stats.total_entries ?? 0} entries · {stats.unique_domains ?? 0} domains
    </p>
  {/if}

  <div class="search-row">
    <input
      type="search"
      placeholder="Search URL or title…"
      bind:value={searchQuery}
      onkeydown={(e) => e.key === 'Enter' && void runSearch()}
    />
    <button type="button" class="nav-button secondary" onclick={() => void runSearch()}>Search</button>
    <button type="button" class="nav-button secondary" onclick={() => void load()}>Refresh</button>
  </div>

  {#if loading}
    <p class="hint">Loading…</p>
  {:else if entries.length === 0}
    <p class="hint">No history entries yet.</p>
  {:else}
    <ul class="entry-list">
      {#each entries as entry (entry.id)}
        <li class="entry-row">
          <div class="entry-main">
            <a href={entry.url} class="entry-title" onclick={(e) => e.preventDefault()}>{entry.title || entry.url}</a>
            <span class="entry-url">{entry.url}</span>
            <span class="entry-meta">{formatTime(entry.last_visit)} · {entry.visit_count} visits</span>
          </div>
          <div class="entry-actions">
            <button type="button" class="link-btn" onclick={() => void deleteEntry(entry.id)}>Remove</button>
            {#if entry.url}
              {@const host = (() => { try { return new URL(entry.url).hostname; } catch { return ''; } })()}
              {#if host}
                <button type="button" class="link-btn danger" onclick={() => void deleteDomain(host)}>
                  Clear {host}
                </button>
              {/if}
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <button
    type="button"
    class="nav-button secondary full danger"
    onclick={() => void handleClearAll()}
  >
    {confirmClear ? 'Click again to confirm clear all' : 'Clear full history store'}
  </button>
</div>

<style>
  .history-panel h3 {
    margin: 0 0 8px;
    color: #e8e8e8;
  }

  .hint {
    font-size: 12px;
    color: #888;
    margin: 0 0 12px;
  }

  .stats-line {
    font-size: 12px;
    color: #9cdcfe;
    margin: 0 0 10px;
  }

  .settings-toggles label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
    color: #ccc;
  }

  .settings-toggles input[type='number'] {
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

  .search-row {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
    flex-wrap: wrap;
  }

  .search-row input {
    flex: 1;
    min-width: 140px;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid #505050;
    background: #1e1e1e;
    color: #e0e0e0;
  }

  .entry-list {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
    max-height: 220px;
    overflow-y: auto;
  }

  .entry-row {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid #3a3a3a;
  }

  .entry-title {
    display: block;
    color: #e0e0e0;
    font-size: 13px;
    text-decoration: none;
  }

  .entry-url {
    display: block;
    font-size: 11px;
    color: #6a9955;
    word-break: break-all;
  }

  .entry-meta {
    font-size: 11px;
    color: #666;
  }

  .entry-actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  .link-btn {
    background: none;
    border: none;
    color: #9cdcfe;
    font-size: 11px;
    cursor: pointer;
    padding: 0;
    text-align: right;
  }

  .link-btn.danger {
    color: #f87171;
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

  .nav-button.full {
    width: 100%;
    margin-top: 8px;
  }

  .nav-button.danger {
    background: #7f1d1d;
    color: #fecaca;
  }
</style>
