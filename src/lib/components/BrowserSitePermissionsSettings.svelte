<script lang="ts">
  /**
   * Exodus Browser — manage per-origin camera / microphone / geolocation grants.
   */
  import {
    listBrowserSitePermissions,
    revokeBrowserSitePermission,
    type BrowserSitePermissionEntry,
  } from '$lib/extensions/api';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let entries = $state<BrowserSitePermissionEntry[]>([]);
  let loading = $state(false);

  /** Human-readable permission kind label. */
  function kindLabel(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'camera':
        return 'Camera';
      case 'microphone':
      case 'mic':
        return 'Microphone';
      case 'geolocation':
      case 'location':
        return 'Location';
      case 'notifications':
        return 'Notifications';
      default:
        return kind;
    }
  }

  /** Reload stored site permission decisions from disk. */
  async function refresh() {
    loading = true;
    try {
      entries = await listBrowserSitePermissions();
    } catch (error) {
      console.error('browser_site_permissions_list failed:', error);
      onStatus('Failed to load site permissions');
    } finally {
      loading = false;
    }
  }

  /** Remove one stored decision so the site will prompt again. */
  async function revoke(entry: BrowserSitePermissionEntry) {
    try {
      await revokeBrowserSitePermission(entry.origin, [entry.kind]);
      await refresh();
      onStatus(`Removed ${kindLabel(entry.kind)} for ${entry.origin}`);
    } catch (error) {
      console.error('browser_site_permissions_revoke failed:', error);
      onStatus('Failed to revoke site permission');
    }
  }

  /** Clear all stored decisions for an origin. */
  async function revokeOrigin(origin: string) {
    try {
      await revokeBrowserSitePermission(origin);
      await refresh();
      onStatus(`Cleared all permissions for ${origin}`);
    } catch (error) {
      console.error('browser_site_permissions_revoke failed:', error);
      onStatus('Failed to clear site permissions');
    }
  }

  $effect(() => {
    if (typeof window !== 'undefined') {
      void refresh();
    }
  });
</script>

<div class="site-perms-section">
  <h4 class="subsection-title">Site permissions (camera, mic, location)</h4>
  <p class="settings-hint">
    Per-origin browser decisions (separate from extension site access). Revoking resets the
    site so you will be asked again.
  </p>
  <div class="site-perms-actions">
    <button type="button" class="nav-button secondary" disabled={loading} onclick={() => void refresh()}>
      Refresh
    </button>
  </div>
  {#if loading}
    <p class="settings-hint">Loading…</p>
  {:else if entries.length === 0}
    <p class="settings-hint">No saved site permissions.</p>
  {:else}
    <ul class="site-perm-list">
      {#each entries as entry (`${entry.origin}:${entry.kind}`)}
        <li class="site-perm-item">
          <div class="site-perm-meta">
            <strong>{entry.origin}</strong>
            <span class="muted">
              {kindLabel(entry.kind)} — {entry.granted ? 'Allowed' : 'Blocked'}
            </span>
          </div>
          <div class="site-perm-buttons">
            <button
              type="button"
              class="nav-button secondary"
              onclick={() => void revoke(entry)}
            >
              Reset
            </button>
          </div>
        </li>
      {/each}
    </ul>
    {#if entries.length > 0}
      {@const origins = [...new Set(entries.map((e) => e.origin))]}
      <div class="origin-clear-row">
        {#each origins as origin (origin)}
          <button
            type="button"
            class="nav-button secondary danger"
            onclick={() => void revokeOrigin(origin)}
          >
            Clear all for {origin}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .site-perms-section {
    margin-top: 12px;
  }

  .subsection-title {
    margin: 0 0 8px;
    font-size: 13px;
    color: #ccc;
  }

  .site-perms-actions {
    margin-bottom: 8px;
  }

  .site-perm-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .site-perm-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    background: #2a2a2a;
    border: 1px solid #404040;
    border-radius: 8px;
  }

  .site-perm-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 12px;
    min-width: 0;
  }

  .site-perm-meta strong {
    font-size: 13px;
    color: #e0e0e0;
    word-break: break-all;
  }

  .muted {
    color: #888;
  }

  .origin-clear-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 10px;
  }
</style>
