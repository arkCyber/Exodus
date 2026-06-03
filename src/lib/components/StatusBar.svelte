<script lang="ts">
  /**
   * Exodus Browser — status line: privacy mode badges and transient messages.
   */
  import type { PrivacyStatsSummary } from '$lib/privacyStats';

  type Props = {
    message: string;
    privateMode?: boolean;
    httpsOnly?: boolean;
    blockPopups?: boolean;
    privacyStats?: PrivacyStatsSummary | null;
  };

  let {
    message,
    privateMode = false,
    httpsOnly = false,
    blockPopups = false,
    privacyStats = null,
  }: Props = $props();

  const showBar = $derived(
    Boolean(message) ||
      privateMode ||
      httpsOnly ||
      blockPopups ||
      (privacyStats !== null && privacyStats.trackers_blocked > 0),
  );
</script>

{#if showBar}
  <div class="status-bar exodus-status-bar" role="status" aria-live="polite">
    <div class="badges">
      {#if privateMode}
        <span class="badge badge-private" title="Private browsing — visits are not recorded">Private</span>
      {/if}
      {#if httpsOnly}
        <span class="badge badge-https" title="HTTPS-only mode">HTTPS only</span>
      {/if}
      {#if blockPopups}
        <span class="badge badge-popup" title="Popup windows are blocked">Popups blocked</span>
      {/if}
      {#if privacyStats && privacyStats.trackers_blocked > 0}
        <span class="badge badge-trackers" title="Tracker requests blocked this session">
          {privacyStats.trackers_blocked} trackers blocked
        </span>
      {/if}
    </div>
    {#if message}
      <span class="message">{message}</span>
    {/if}
  </div>
{/if}

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 16px;
    font-size: 12px;
    color: #888;
    background: #222;
    border-bottom: 1px solid #333;
    min-height: 26px;
  }

  .badges {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .badge {
    padding: 1px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .badge-private {
    color: #c4b5fd;
    background: rgba(139, 92, 246, 0.2);
  }

  .badge-https {
    color: #6ee7b7;
    background: rgba(16, 185, 129, 0.15);
  }

  .badge-popup {
    color: #fcd34d;
    background: rgba(245, 158, 11, 0.15);
  }

  .badge-trackers {
    color: #86efac;
    background: rgba(34, 197, 94, 0.15);
  }

  .message {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
