<script lang="ts">
  /**
   * Exodus Browser — per-origin site permission (camera, microphone, geolocation).
   */
  import type { BrowserSitePermissionRequestEvent } from '$lib/extensions/extensionEvents';
  import { resolveBrowserSitePermission } from '$lib/extensions/api';

  type Props = {
    request: BrowserSitePermissionRequestEvent | null;
    onResolved: () => void;
  };

  let { request, onResolved }: Props = $props();
  let busy = $state(false);

  /** Human-readable label for permission kind from the bridge. */
  function kindLabel(kind: string): string {
    switch (kind.toLowerCase()) {
      case 'camera':
        return 'use your camera';
      case 'microphone':
      case 'mic':
        return 'use your microphone';
      case 'geolocation':
      case 'location':
        return 'know your location';
      case 'notifications':
        return 'show notifications';
      default:
        return `use ${kind}`;
    }
  }

  /** Grant or deny the pending browser site permission. */
  async function answer(granted: boolean) {
    if (!request || busy) return;
    busy = true;
    try {
      await resolveBrowserSitePermission(request.requestId, granted);
    } catch (error) {
      console.error('browser_site_permission_resolve failed:', error);
    } finally {
      busy = false;
      onResolved();
    }
  }
</script>

{#if request}
  <div class="perm-backdrop" role="presentation">
    <div class="perm-dialog" role="dialog" aria-labelledby="site-perm-title">
      <h3 id="site-perm-title">Site permission</h3>
      <p>
        <strong>{request.origin}</strong> wants to {kindLabel(request.kind)}.
      </p>
      <div class="perm-actions">
        <button type="button" class="btn secondary" disabled={busy} onclick={() => answer(false)}>
          Block
        </button>
        <button type="button" class="btn primary" disabled={busy} onclick={() => answer(true)}>
          Allow
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .perm-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10002;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
  }
  .perm-dialog {
    max-width: 420px;
    padding: 1.25rem 1.5rem;
    border-radius: 12px;
    background: var(--exodus-surface, #1e1e1e);
    color: var(--exodus-text, #eee);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
  }
  .perm-dialog h3 {
    margin: 0 0 0.75rem;
    font-size: 1.1rem;
  }
  .perm-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    margin-top: 1rem;
  }
  .btn {
    padding: 0.4rem 1rem;
    border-radius: 8px;
    border: none;
    cursor: pointer;
  }
  .btn.primary {
    background: var(--exodus-accent, #3b82f6);
    color: #fff;
  }
  .btn.secondary {
    background: var(--exodus-muted, #444);
    color: #fff;
  }
</style>
