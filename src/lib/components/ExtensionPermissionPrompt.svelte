<script lang="ts">
  /**
   * Exodus Browser — modal prompt for chrome.permissions.request.
   */
  import type { ExtensionPermissionRequestEvent } from '$lib/extensions/extensionEvents';
  import { extensionDisplayName } from '$lib/extensions/extensionDisplayName';
  import { resolveExtensionPermission } from '$lib/extensions/api';

  /** Host permission patterns include URL schemes or &lt;all_urls&gt;. */
  function isHostPermission(perm: string): boolean {
    return perm === '<all_urls>' || perm.includes('://') || perm.startsWith('*.');
  }

  type Props = {
    request: ExtensionPermissionRequestEvent | null;
    onResolved: () => void;
  };

  let { request, onResolved }: Props = $props();

  let busy = $state(false);
  const label = $derived(extensionDisplayName(request));

  /** Grant or deny the pending permission request. */
  async function answer(granted: boolean) {
    if (!request || busy) return;
    busy = true;
    try {
      await resolveExtensionPermission(request.requestId, granted);
    } catch (error) {
      console.error('extension_permissions_resolve failed:', error);
    } finally {
      busy = false;
      onResolved();
    }
  }
</script>

{#if request}
  <div class="perm-backdrop" role="presentation">
    <div class="perm-dialog" role="dialog" aria-labelledby="perm-title">
      <h3 id="perm-title">Permission request</h3>
      <p>
        <strong>{label}</strong> requests:
      </p>
      {#if request.permissions.some((p) => !isHostPermission(p))}
        <p class="perm-sub">API permissions:</p>
        <ul>
          {#each request.permissions.filter((p) => !isHostPermission(p)) as perm}
            <li><code>{perm}</code></li>
          {/each}
        </ul>
      {/if}
      {#if request.permissions.some(isHostPermission)}
        <p class="perm-sub">Site access:</p>
        <ul>
          {#each request.permissions.filter(isHostPermission) as perm}
            <li><code>{perm}</code></li>
          {/each}
        </ul>
      {/if}
      <div class="perm-actions">
        <button type="button" class="btn secondary" disabled={busy} onclick={() => answer(false)}>
          Deny
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
    z-index: 10000;
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
  .perm-dialog ul {
    margin: 0.5rem 0 1rem;
    padding-left: 1.25rem;
  }
  .perm-sub {
    margin: 0.5rem 0 0.25rem;
    font-size: 0.9rem;
    opacity: 0.85;
  }
  .perm-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
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
