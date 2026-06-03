<script lang="ts">
  /**
   * Exodus Browser — Web Extensions management (Settings section).
   */
  import type { ExtensionInfo, StoreExtensionEntry } from '$lib/extensions/types';
  import {
    installExtensionFolder,
    installExtensionCrx,
    listExtensions,
    listStoreExtensions,
    fetchRemoteStoreExtensions,
    rescanExtensions,
    setExtensionEnabled,
    uninstallExtension,
    setConfirmHostPermissionsOnInstall,
    listExtensionSitePermissions,
    revokeExtensionSitePermissions,
    revokeAllExtensionSitePermissions,
  } from '$lib/extensions/api';
  import { ensureExtensionBackgrounds } from '$lib/extensions/backgroundHosts';
  import { invoke } from '@tauri-apps/api/core';
  import type { ExodusConfigDto } from '$lib/browserSettings';

  type Props = {
    onStatus: (message: string) => void;
    contentHost?: HTMLElement;
  };

  let { onStatus, contentHost }: Props = $props();

  let extensions = $state<ExtensionInfo[]>([]);
  let storeItems = $state<StoreExtensionEntry[]>([]);
  let loading = $state(false);
  let storeUrl = $state('');
  let confirmHostOnInstall = $state(true);
  let expandedSiteAccessId = $state<string | null>(null);
  let hostPatternsByExt = $state<Record<string, string[]>>({});
  let hostPatternsLoading = $state<string | null>(null);

  /** Reload extension list from the backend (local + optional remote catalog). */
  async function refresh() {
    loading = true;
    try {
      extensions = await listExtensions();
      const local = await listStoreExtensions();
      let remote: StoreExtensionEntry[] = [];
      try {
        remote = await fetchRemoteStoreExtensions();
      } catch {
        /* remote catalog optional */
      }
      const byId = new Map<string, StoreExtensionEntry>();
      for (const item of [...local, ...remote]) {
        byId.set(item.id, item);
      }
      storeItems = [...byId.values()].sort((a, b) => a.name.localeCompare(b.name));
    } catch (error) {
      console.error('extension_list failed:', error);
      onStatus('Failed to load extensions');
    } finally {
      loading = false;
    }
  }

  async function rebootBackgrounds() {
    if (contentHost) {
      await ensureExtensionBackgrounds(contentHost);
    }
  }

  /** Rescan extension directories. */
  async function handleRescan() {
    try {
      const count = await rescanExtensions();
      await refresh();
      await rebootBackgrounds();
      onStatus(`Rescanned extensions (${count} scanned)`);
    } catch (error) {
      console.error('extension_rescan failed:', error);
      onStatus('Extension rescan failed');
    }
  }

  /** Toggle extension enabled state. */
  async function toggleEnabled(ext: ExtensionInfo) {
    try {
      await setExtensionEnabled(ext.id, !ext.enabled);
      await refresh();
      await rebootBackgrounds();
      onStatus(ext.enabled ? `Disabled ${ext.name}` : `Enabled ${ext.name}`);
    } catch (error) {
      console.error('extension_set_enabled failed:', error);
      onStatus('Failed to update extension');
    }
  }

  /** Install from folder via native dialog path input. */
  async function handleInstall() {
    const path = window.prompt('Path to unpacked extension folder (manifest.json inside):');
    if (!path?.trim()) return;
    try {
      await installExtensionFolder(path.trim());
      await refresh();
      await rebootBackgrounds();
      onStatus('Extension installed');
    } catch (error) {
      console.error('extension_install_folder failed:', error);
      onStatus(`Install failed: ${error}`);
    }
  }

  /** Install from `.crx` or `.zip` package. */
  async function handleInstallCrx() {
    const path = window.prompt('Path to .crx or .zip extension package:');
    if (!path?.trim()) return;
    try {
      await installExtensionCrx(path.trim());
      await refresh();
      await rebootBackgrounds();
      onStatus('Extension package installed');
    } catch (error) {
      console.error('extension_install_crx failed:', error);
      onStatus(`CRX install failed: ${error}`);
    }
  }

  /** Install a store catalog entry (dev folder). */
  async function installStoreItem(item: StoreExtensionEntry) {
    try {
      await installExtensionFolder(item.path);
      await refresh();
      await rebootBackgrounds();
      onStatus(`Installed ${item.name}`);
    } catch (error) {
      console.error('extension_install_folder failed:', error);
      onStatus(`Install failed: ${error}`);
    }
  }

  /** Uninstall extension after confirmation. */
  async function handleUninstall(ext: ExtensionInfo) {
    if (!confirm(`Uninstall extension "${ext.name}"?`)) return;
    try {
      await uninstallExtension(ext.id);
      await refresh();
      onStatus(`Uninstalled ${ext.name}`);
    } catch (error) {
      console.error('extension_uninstall failed:', error);
      onStatus('Uninstall failed');
    }
  }

  /** Load extension-related settings from app config. */
  async function loadExtensionSettings() {
    try {
      const cfg = await invoke<ExodusConfigDto>('get_ai_config');
      storeUrl = cfg.extension_store_url ?? '';
      confirmHostOnInstall = cfg.confirm_host_permissions_on_install ?? true;
    } catch {
      storeUrl = '';
      confirmHostOnInstall = true;
    }
  }

  /** Persist install-time host permission confirmation preference. */
  async function toggleConfirmHostOnInstall() {
    try {
      await setConfirmHostPermissionsOnInstall(confirmHostOnInstall);
      onStatus(
        confirmHostOnInstall
          ? 'Install will ask before granting site access'
          : 'Install will auto-grant manifest site access',
      );
    } catch (error) {
      console.error('extension_set_confirm_host_permissions failed:', error);
      onStatus('Failed to save host permission setting');
    }
  }

  /** Load granted host patterns for an extension. */
  async function loadHostPatterns(extensionId: string) {
    hostPatternsLoading = extensionId;
    try {
      const patterns = await listExtensionSitePermissions(extensionId);
      hostPatternsByExt = { ...hostPatternsByExt, [extensionId]: patterns };
    } catch (error) {
      console.error('extension_site_permissions_list failed:', error);
      onStatus('Failed to load extension site access');
    } finally {
      hostPatternsLoading = null;
    }
  }

  /** Expand or collapse site-access panel for an extension. */
  async function toggleSiteAccess(ext: ExtensionInfo) {
    if (expandedSiteAccessId === ext.id) {
      expandedSiteAccessId = null;
      return;
    }
    expandedSiteAccessId = ext.id;
    if (!hostPatternsByExt[ext.id]) {
      await loadHostPatterns(ext.id);
    }
  }

  /** Revoke every granted host pattern for an extension. */
  async function revokeAllHostPatterns(extensionId: string, extName: string) {
    const patterns = hostPatternsByExt[extensionId] ?? [];
    if (patterns.length === 0) return;
    if (!confirm(`Revoke all site access for "${extName}" (${patterns.length} patterns)?`)) return;
    try {
      await revokeAllExtensionSitePermissions(extensionId);
      await loadHostPatterns(extensionId);
      onStatus(`Revoked all site access for ${extName}`);
    } catch (error) {
      console.error('extension_site_permissions_revoke_all failed:', error);
      onStatus('Failed to revoke all site access');
    }
  }

  /** Revoke one host pattern so the extension loses access until re-granted. */
  async function revokeHostPattern(extensionId: string, pattern: string) {
    try {
      await revokeExtensionSitePermissions(extensionId, [pattern]);
      await loadHostPatterns(extensionId);
      onStatus(`Revoked site access: ${pattern}`);
    } catch (error) {
      console.error('extension_site_permissions_revoke failed:', error);
      onStatus('Failed to revoke site access');
    }
  }

  /** Persist remote store URL. */
  async function saveStoreUrl() {
    try {
      await invoke('extension_set_store_url', { url: storeUrl.trim() });
      onStatus('Extension store URL saved');
      await refresh();
    } catch (error) {
      console.error('extension_set_store_url failed:', error);
      onStatus('Failed to save store URL');
    }
  }

  $effect(() => {
    if (typeof window !== 'undefined') {
      void loadExtensionSettings();
      void refresh();
    }
  });
</script>

<div class="settings-section">
  <h3>Web Extensions</h3>
  <p class="settings-hint">
    Manifest V3 extensions: content scripts, background service worker, storage, tabs API,
  </p>
  <label class="checkbox-row">
    <input
      type="checkbox"
      bind:checked={confirmHostOnInstall}
      onchange={() => void toggleConfirmHostOnInstall()}
    />
    <span>Ask before granting extension site access on install</span>
  </label>

  <label class="store-url-row">
    Remote store catalog URL (JSON)
    <input
      type="url"
      bind:value={storeUrl}
      placeholder="https://example.com/extensions/catalog.json"
    />
    <button type="button" class="nav-button secondary" onclick={() => void saveStoreUrl()}>
      Save store URL
    </button>
  </label>

  <div class="extensions-actions">
    <button type="button" class="nav-button secondary" onclick={() => void refresh()} disabled={loading}>
      Refresh
    </button>
    <button type="button" class="nav-button secondary" onclick={() => void handleRescan()}>
      Rescan
    </button>
    <button type="button" class="nav-button secondary" onclick={() => void handleInstall()}>
      Install folder…
    </button>
    <button type="button" class="nav-button secondary" onclick={() => void handleInstallCrx()}>
      Install .crx / .zip…
    </button>
  </div>

  {#if storeItems.length > 0}
    <h4 class="subsection-title">Extension store (dev)</h4>
    <ul class="extension-list compact">
      {#each storeItems as item (item.id)}
        <li class="extension-item">
          <div class="extension-meta">
            <strong>{item.name}</strong>
            <span class="muted">v{item.version} · {item.id}</span>
            {#if item.description}
              <span class="muted">{item.description}</span>
            {/if}
          </div>
          <button
            type="button"
            class="nav-button secondary"
            disabled={item.installed}
            onclick={() => void installStoreItem(item)}
          >
            {item.installed ? 'Installed' : 'Install'}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <h4 class="subsection-title">Installed</h4>
  {#if loading}
    <p class="settings-hint">Loading extensions…</p>
  {:else if extensions.length === 0}
    <p class="settings-hint">No extensions installed.</p>
  {:else}
    <ul class="extension-list">
      {#each extensions as ext (ext.id)}
        <li class="extension-item">
          <div class="extension-row">
          <div class="extension-meta">
            <strong>{ext.name}</strong>
            <span class="muted">v{ext.version} · {ext.id}</span>
            {#if ext.description}
              <span class="muted">{ext.description}</span>
            {/if}
            <span class="muted">Permissions: {ext.permissions.join(', ') || 'none'}</span>
            {#if ext.actionPopup}
              <span class="muted">Popup: {ext.actionPopup}</span>
            {/if}
          </div>
          <div class="extension-buttons">
            <label class="checkbox-row compact">
              <input
                type="checkbox"
                checked={ext.enabled}
                onchange={() => void toggleEnabled(ext)}
              />
              <span>Enabled</span>
            </label>
            <button
              type="button"
              class="nav-button secondary"
              onclick={() => void toggleSiteAccess(ext)}
            >
              {expandedSiteAccessId === ext.id ? 'Hide site access' : 'Site access'}
            </button>
            <button
              type="button"
              class="nav-button secondary danger"
              onclick={() => void handleUninstall(ext)}
            >
              Uninstall
            </button>
          </div>
          </div>
          {#if expandedSiteAccessId === ext.id}
            <div class="host-patterns-panel">
              {#if hostPatternsLoading === ext.id}
                <p class="settings-hint">Loading granted sites…</p>
              {:else if (hostPatternsByExt[ext.id] ?? []).length === 0}
                <p class="settings-hint">No granted site patterns yet.</p>
              {:else}
                <div class="host-patterns-toolbar">
                  <button
                    type="button"
                    class="nav-button secondary danger"
                    onclick={() => void revokeAllHostPatterns(ext.id, ext.name)}
                  >
                    Revoke all
                  </button>
                </div>
                <ul class="host-pattern-list">
                  {#each hostPatternsByExt[ext.id] ?? [] as pattern (pattern)}
                    <li class="host-pattern-row">
                      <code>{pattern}</code>
                      <button
                        type="button"
                        class="nav-button secondary"
                        onclick={() => void revokeHostPattern(ext.id, pattern)}
                      >
                        Revoke
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .store-url-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
    font-size: 13px;
  }

  .store-url-row input {
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid #444;
    background: #1a1a1a;
    color: #eee;
  }

  .extensions-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .subsection-title {
    margin: 16px 0 8px;
    font-size: 13px;
    color: #ccc;
  }

  .extension-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .extension-list.compact .extension-item {
    padding: 8px 10px;
  }

  .extension-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    background: #2a2a2a;
    border: 1px solid #404040;
    border-radius: 8px;
  }

  .extension-row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .host-patterns-panel {
    width: 100%;
    padding-top: 8px;
    border-top: 1px solid #404040;
  }

  .host-patterns-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 8px;
  }

  .host-pattern-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .host-pattern-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
  }

  .host-pattern-row code {
    word-break: break-all;
    color: #ccc;
  }

  .extension-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 12px;
  }

  .extension-meta strong {
    font-size: 14px;
    color: #e0e0e0;
  }

  .extension-buttons {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    flex-shrink: 0;
  }

  .checkbox-row.compact {
    margin: 0;
  }

  .muted {
    color: #888;
  }
</style>
