<script lang="ts">
  /**
   * Exodus Browser — privacy protection statistics (Settings).
   */
  import { invoke } from '@tauri-apps/api/core';

  type PrivacyStats = {
    trackers_blocked: number;
    cookies_blocked: number;
    fingerprinting_blocked: number;
    malicious_sites_blocked: number;
    data_saved: number;
    time_saved: number;
    start_timestamp: number;
  };

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let stats = $state<PrivacyStats | null>(null);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      stats = await invoke<PrivacyStats>('get_privacy_stats');
    } catch (error) {
      console.error('get_privacy_stats failed:', error);
      onStatus('Failed to load privacy stats');
    } finally {
      loading = false;
    }
  }

  async function clearStats() {
    try {
      await invoke('reset_privacy_stats');
      await load();
      onStatus('Privacy stats reset');
    } catch (error) {
      console.error('reset_privacy_stats failed:', error);
      onStatus('Failed to reset stats');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

<div class="privacy-dashboard">
  <h4>Privacy dashboard</h4>
  {#if loading}
    <p class="hint">Loading…</p>
  {:else if stats}
    <ul class="stat-list">
      <li><span>Trackers blocked</span><strong>{stats.trackers_blocked}</strong></li>
      <li><span>Malicious sites blocked</span><strong>{stats.malicious_sites_blocked}</strong></li>
      <li><span>Cookies blocked</span><strong>{stats.cookies_blocked}</strong></li>
      <li><span>Fingerprinting blocked</span><strong>{stats.fingerprinting_blocked}</strong></li>
    </ul>
    <div class="actions">
      <button type="button" class="nav-button secondary" onclick={() => void load()}>Refresh</button>
      <button type="button" class="nav-button secondary" onclick={() => void clearStats()}>Reset stats</button>
    </div>
  {/if}
</div>

<style>
  .privacy-dashboard h4 {
    margin: 12px 0 8px;
    font-size: 14px;
    color: #e0e0e0;
  }

  .hint {
    font-size: 13px;
    color: #999;
  }

  .stat-list {
    list-style: none;
    margin: 0 0 12px;
    padding: 0;
  }

  .stat-list li {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    font-size: 13px;
    color: #ccc;
    border-bottom: 1px solid #404040;
  }

  .stat-list strong {
    color: #9cdcfe;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
</style>
