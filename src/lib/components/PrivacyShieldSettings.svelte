<script lang="ts">
  /**
   * Exodus Browser — Safe Browsing & tracking protection toggles (Settings).
   */
  import {
    loadEncryptedSyncSettings,
    loadSafeBrowsingSettings,
    loadTrackingProtectionSettings,
    refreshSafeBrowsingList,
    saveSafeBrowsingSettings,
    saveTrackingProtectionSettings,
    setEncryptedSyncPassphrase,
    setEncryptedSyncServer,
    setTrackingSubscription,
    storeEncryptedBookmarkVault,
    uploadEncryptedVault,
    downloadEncryptedVault,
    type EncryptedSyncSettings,
    type SafeBrowsingSettings,
    type TrackingProtectionSettings,
  } from '$lib/browserIntegrations';
  import { refreshTrackerBlocklist } from '$lib/siteShields';

  type Props = {
    onStatus: (message: string) => void;
  };

  let { onStatus }: Props = $props();

  let safeBrowsing = $state<SafeBrowsingSettings>({
    enabled: true,
    block_malware: true,
    block_phishing: true,
    block_unwanted_software: true,
    show_warnings: true,
    allow_proceed: true,
  });
  let tracking = $state<TrackingProtectionSettings>({
    enabled: true,
    block_advertising: true,
    block_analytics: true,
    block_social: false,
    block_fingerprinting: true,
    block_cryptomining: true,
    block_tracking: true,
  });
  let loaded = $state(false);
  let safeListUrl = $state('');
  let subscriptionUrl = $state('');
  let subscriptionHours = $state(24);
  let syncPassphrase = $state('');
  let syncServerUrl = $state('');
  let syncToken = $state('');
  let encryptedSync = $state<EncryptedSyncSettings>({
    enabled: false,
    has_passphrase: false,
    last_sync_at: 0,
    sync_server_url: null,
    sync_token: null,
    device_id: null,
  });

  async function load() {
    try {
      safeBrowsing = await loadSafeBrowsingSettings();
      tracking = await loadTrackingProtectionSettings();
      encryptedSync = await loadEncryptedSyncSettings();
      syncServerUrl = encryptedSync.sync_server_url ?? '';
      syncToken = encryptedSync.sync_token ?? '';
      safeListUrl = safeBrowsing.list_url ?? '';
      subscriptionUrl = tracking.subscription_url ?? '';
      subscriptionHours = tracking.subscription_refresh_hours ?? 24;
      loaded = true;
    } catch (error) {
      console.error('PrivacyShieldSettings load failed:', error);
      onStatus('Failed to load privacy shield settings');
    }
  }

  async function persistSafe() {
    try {
      await saveSafeBrowsingSettings(safeBrowsing);
      onStatus('Safe Browsing settings saved');
    } catch (error) {
      console.error('saveSafeBrowsingSettings failed:', error);
      onStatus('Failed to save Safe Browsing settings');
    }
  }

  async function persistTracking() {
    try {
      await saveTrackingProtectionSettings(tracking);
      onStatus('Tracking protection settings saved');
    } catch (error) {
      console.error('saveTrackingProtectionSettings failed:', error);
      onStatus('Failed to save tracking protection settings');
    }
  }

  async function updateBlocklist() {
    try {
      const count = await refreshTrackerBlocklist(subscriptionUrl || undefined);
      onStatus(`Tracker list updated (${count} domains)`);
    } catch (error) {
      console.error('refreshTrackerBlocklist failed:', error);
      onStatus('Failed to update tracker list');
    }
  }

  async function saveSubscription() {
    try {
      await setTrackingSubscription(subscriptionUrl || null, subscriptionHours);
      await persistTracking();
      onStatus('Blocklist subscription saved');
    } catch (error) {
      console.error('setTrackingSubscription failed:', error);
      onStatus('Failed to save subscription');
    }
  }

  async function refreshSafeList() {
    try {
      const next = { ...safeBrowsing, list_url: safeListUrl || null };
      await saveSafeBrowsingSettings(next);
      safeBrowsing = next;
      const added = await refreshSafeBrowsingList(safeListUrl || undefined);
      onStatus(`Safe Browsing list refreshed (+${added} threats)`);
    } catch (error) {
      console.error('refreshSafeBrowsingList failed:', error);
      onStatus('Failed to refresh Safe Browsing list');
    }
  }

  async function saveSyncServer() {
    try {
      await setEncryptedSyncServer(syncServerUrl || null, syncToken || null);
      encryptedSync = await loadEncryptedSyncSettings();
      onStatus('Sync server settings saved');
    } catch (error) {
      console.error('setEncryptedSyncServer failed:', error);
      onStatus('Failed to save sync server');
    }
  }

  async function uploadVault() {
    try {
      const msg = await uploadEncryptedVault();
      encryptedSync = await loadEncryptedSyncSettings();
      onStatus(msg);
    } catch (error) {
      console.error('uploadEncryptedVault failed:', error);
      onStatus('Cloud upload failed');
    }
  }

  async function downloadVault() {
    try {
      const n = await downloadEncryptedVault();
      onStatus(`Downloaded vault (${n} bytes)`);
    } catch (error) {
      console.error('downloadEncryptedVault failed:', error);
      onStatus('Cloud download failed');
    }
  }

  async function enableEncryptedSync() {
    if (syncPassphrase.length < 8) {
      onStatus('Passphrase must be at least 8 characters');
      return;
    }
    try {
      await setEncryptedSyncPassphrase(syncPassphrase);
      encryptedSync = await loadEncryptedSyncSettings();
      syncPassphrase = '';
      onStatus('Encrypted sync vault enabled');
    } catch (error) {
      console.error('setEncryptedSyncPassphrase failed:', error);
      onStatus('Failed to set sync passphrase');
    }
  }

  async function backupBookmarksEncrypted() {
    try {
      const raw = localStorage.getItem('exodus-bookmarks');
      if (!raw) {
        onStatus('No local bookmarks to encrypt');
        return;
      }
      await storeEncryptedBookmarkVault(raw);
      onStatus('Bookmarks encrypted to local sync vault');
    } catch (error) {
      console.error('storeEncryptedBookmarkVault failed:', error);
      onStatus('Encrypted bookmark backup failed');
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void load();
  });
</script>

{#if loaded}
  <div class="privacy-shield">
    <h4>Safe Browsing</h4>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={safeBrowsing.enabled} onchange={() => void persistSafe()} />
      <span>Enable Safe Browsing (phishing & malware lists)</span>
    </label>
    <label class="checkbox-row">
      <input
        type="checkbox"
        bind:checked={safeBrowsing.allow_proceed}
        onchange={() => void persistSafe()}
      />
      <span>Allow proceeding after a warning</span>
    </label>

    <h4>Tracking protection</h4>
    <p class="hint">
      Blocks fetch/XHR/beacon to known tracker domains in each page (injected at document start).
    </p>
    <label class="checkbox-row">
      <input type="checkbox" bind:checked={tracking.enabled} onchange={() => void persistTracking()} />
      <span>Enable tracking protection</span>
    </label>
    <label class="checkbox-row">
      <input
        type="checkbox"
        bind:checked={tracking.block_advertising}
        onchange={() => void persistTracking()}
      />
      <span>Block advertising trackers</span>
    </label>
    <label class="checkbox-row">
      <input
        type="checkbox"
        bind:checked={tracking.block_analytics}
        onchange={() => void persistTracking()}
      />
      <span>Block analytics trackers</span>
    </label>
    <label class="field">
      Subscription URL (JSON <code>domains</code> or EasyList/ABP)
      <input type="url" bind:value={subscriptionUrl} placeholder="https://…/easylist.txt" />
    </label>
    <label class="field">
      Auto-refresh (hours, 0 = manual)
      <input type="number" min="0" max="168" bind:value={subscriptionHours} />
    </label>
    <div class="btn-row">
      <button type="button" class="refresh-blocklist" onclick={() => void saveSubscription()}>
        Save subscription
      </button>
      <button type="button" class="refresh-blocklist secondary" onclick={() => void updateBlocklist()}>
        Update tracker blocklist now
      </button>
    </div>

    <h4>Safe Browsing online list</h4>
    <label class="field">
      Threat list URL (JSON <code>threats</code>)
      <input type="url" bind:value={safeListUrl} placeholder="https://…/threats.json" />
    </label>
    <button type="button" class="refresh-blocklist" onclick={() => void refreshSafeList()}>
      Refresh Safe Browsing list
    </button>

    <h4>Encrypted sync (local vault)</h4>
    <p class="hint">
      AES-256-GCM encrypted backup on disk. Cloud sync server is not wired yet; vault is ready for
      E2E upload later.
    </p>
    {#if encryptedSync.has_passphrase}
      <p class="hint">Vault unlocked · last backup {encryptedSync.last_sync_at ? new Date(encryptedSync.last_sync_at * 1000).toLocaleString() : 'never'}</p>
    {/if}
    <label class="field">
      Sync server URL
      <input type="url" bind:value={syncServerUrl} placeholder="http://127.0.0.1:8787/api" />
    </label>
    <label class="field">
      Sync token (optional Bearer)
      <input type="password" bind:value={syncToken} placeholder="API token" />
    </label>
    <button type="button" class="refresh-blocklist secondary" onclick={() => void saveSyncServer()}>
      Save sync server
    </button>
    <label class="field">
      Sync passphrase
      <input type="password" bind:value={syncPassphrase} placeholder="Min 8 characters" />
    </label>
    <div class="btn-row">
      <button type="button" class="refresh-blocklist" onclick={() => void enableEncryptedSync()}>
        Set passphrase
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        disabled={!encryptedSync.has_passphrase}
        onclick={() => void backupBookmarksEncrypted()}
      >
        Encrypt bookmarks to vault
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        disabled={!encryptedSync.has_passphrase || !syncServerUrl}
        onclick={() => void uploadVault()}
      >
        Upload to cloud
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        disabled={!encryptedSync.has_passphrase || !syncServerUrl}
        onclick={() => void downloadVault()}
      >
        Download from cloud
      </button>
    </div>

    <p class="hint">Per-site: Shift+click the shield icon in the address bar to allow trackers on the current site.</p>
  </div>
{/if}

<style>
  .privacy-shield h4 {
    margin: 12px 0 8px;
    font-size: 14px;
    color: #e0e0e0;
  }

  .privacy-shield h4:first-child {
    margin-top: 0;
  }

  .hint {
    font-size: 12px;
    color: #999;
    margin: 0 0 8px;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
    font-size: 13px;
    color: #ccc;
    cursor: pointer;
  }

  .refresh-blocklist {
    margin-top: 10px;
    padding: 8px 12px;
    border-radius: 6px;
    border: 1px solid #6366f1;
    background: rgba(99, 102, 241, 0.15);
    color: #e0e7ff;
    cursor: pointer;
    font-size: 13px;
  }

  .refresh-blocklist:hover {
    background: rgba(99, 102, 241, 0.35);
  }

  .refresh-blocklist.secondary {
    border-color: #4b5563;
    background: rgba(75, 85, 99, 0.25);
    color: #d1d5db;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 10px;
    font-size: 12px;
    color: #aaa;
  }

  .field input {
    padding: 6px 8px;
    border-radius: 6px;
    border: 1px solid #444;
    background: #1a1a1a;
    color: #eee;
    font-size: 13px;
  }

  .btn-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 12px;
  }

  .field code {
    font-size: 11px;
    color: #c4b5fd;
  }
</style>
