<script lang="ts">
  /**
   * Exodus Browser — confirm manifest host_permissions on extension install.
   */
  import type { ExtensionHostInstallRequestEvent } from '$lib/extensions/extensionEvents';
  import { resolveExtensionHostInstall } from '$lib/extensions/api';

  type Props = {
    request: ExtensionHostInstallRequestEvent | null;
    onResolved: () => void;
  };

  let { request, onResolved }: Props = $props();
  let busy = $state(false);

  async function answer(granted: boolean) {
    if (!request || busy) return;
    busy = true;
    try {
      await resolveExtensionHostInstall(request.requestId, granted);
    } catch (error) {
      console.error('extension_host_install_resolve failed:', error);
    } finally {
      busy = false;
      onResolved();
    }
  }
</script>

{#if request}
  <div class="perm-backdrop" role="presentation">
    <div class="perm-dialog" role="dialog" aria-labelledby="host-install-title">
      <h3 id="host-install-title">Extension site access</h3>
      <p><strong>{request.extensionName}</strong> wants access to:</p>
      <ul>
        {#each request.hostPermissions as pattern}
          <li><code>{pattern}</code></li>
        {/each}
      </ul>
      <p class="hint">You can change this later in extension settings.</p>
      <div class="perm-actions">
        <button type="button" class="btn secondary" disabled={busy} onclick={() => answer(false)}>
          Deny sites
        </button>
        <button type="button" class="btn primary" disabled={busy} onclick={() => answer(true)}>
          Allow sites
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .perm-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10001;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
  }
  .perm-dialog {
    max-width: 460px;
    padding: 1.25rem 1.5rem;
    border-radius: 12px;
    background: var(--exodus-surface, #1e1e1e);
    color: var(--exodus-text, #eee);
  }
  .perm-dialog ul {
    margin: 0.5rem 0;
    padding-left: 1.25rem;
    max-height: 200px;
    overflow-y: auto;
  }
  .hint {
    font-size: 0.85rem;
    opacity: 0.8;
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
