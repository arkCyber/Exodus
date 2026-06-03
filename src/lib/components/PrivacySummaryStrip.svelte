<script lang="ts">
  /**
   * Exodus Browser — compact privacy stats at top of sidebar.
   */
  import type { PrivacyStatsSummary } from '$lib/privacyStats';

  type Props = {
    stats: PrivacyStatsSummary | null;
    onOpenSettings?: () => void;
  };

  let { stats, onOpenSettings }: Props = $props();

  const totalBlocks = $derived(
    stats
      ? stats.trackers_blocked +
        stats.malicious_sites_blocked +
        stats.cookies_blocked +
        stats.fingerprinting_blocked
      : 0,
  );
</script>

{#if stats && totalBlocks > 0}
  <div class="privacy-strip">
    <span class="shield" title="Privacy protections active">🛡</span>
    <span class="counts">
      {#if stats.trackers_blocked > 0}
        <span>{stats.trackers_blocked} trackers</span>
      {/if}
      {#if stats.malicious_sites_blocked > 0}
        <span>{stats.malicious_sites_blocked} threats</span>
      {/if}
    </span>
    {#if onOpenSettings}
      <button type="button" class="details-btn" onclick={onOpenSettings}>Details</button>
    {/if}
  </div>
{:else if stats}
  <div class="privacy-strip muted">
    <span>Privacy protections on</span>
    {#if onOpenSettings}
      <button type="button" class="details-btn" onclick={onOpenSettings}>Details</button>
    {/if}
  </div>
{/if}

<style>
  .privacy-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    margin-bottom: 8px;
    background: #1a2e1a;
    border: 1px solid #2d5a2d;
    border-radius: 8px;
    font-size: 12px;
    color: #a7f3d0;
    flex-wrap: wrap;
  }

  .privacy-strip.muted {
    background: #252525;
    border-color: #404040;
    color: #888;
  }

  .counts {
    display: flex;
    gap: 10px;
    flex: 1;
    flex-wrap: wrap;
  }

  .details-btn {
    background: transparent;
    border: 1px solid #4ade80;
    color: #4ade80;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    cursor: pointer;
  }

  .privacy-strip.muted .details-btn {
    border-color: #666;
    color: #aaa;
  }
</style>
