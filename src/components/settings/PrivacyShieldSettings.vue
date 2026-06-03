<!--
  Exodus Browser — Safe Browsing & tracking protection toggles (Settings).
-->
<template>
  <section v-if="loaded" id="settings-section-privacy-shields" class="settings-section privacy-shield" data-testid="privacy-shield-settings">
    <h3>Privacy shields</h3>

    <h4>Safe Browsing</h4>
    <label class="checkbox-row">
      <input v-model="safeBrowsing.enabled" type="checkbox" @change="() => void persistSafe()" data-testid="safe-browsing-enabled" />
      <span>Enable Safe Browsing (phishing &amp; malware lists)</span>
    </label>
    <label class="checkbox-row">
      <input v-model="safeBrowsing.allow_proceed" type="checkbox" @change="() => void persistSafe()" data-testid="safe-browsing-allow-proceed" />
      <span>Allow proceeding after a warning</span>
    </label>

    <h4>Tracking protection</h4>
    <p class="hint">
      Blocks fetch/XHR/beacon to known tracker domains in each page (injected at document start).
    </p>
    <label class="checkbox-row">
      <input v-model="tracking.enabled" type="checkbox" @change="() => void persistTracking()" data-testid="tracking-enabled" />
      <span>Enable tracking protection</span>
    </label>
    <label class="checkbox-row">
      <input v-model="tracking.block_advertising" type="checkbox" @change="() => void persistTracking()" data-testid="tracking-block-ads" />
      <span>Block advertising trackers</span>
    </label>
    <label class="checkbox-row">
      <input v-model="tracking.block_analytics" type="checkbox" @change="() => void persistTracking()" data-testid="tracking-block-analytics" />
      <span>Block analytics trackers</span>
    </label>
    <label class="field">
      Subscription URL (JSON <code>domains</code> or EasyList/ABP)
      <input v-model="subscriptionUrl" type="url" placeholder="https://…/easylist.txt" data-testid="tracking-subscription-url" />
    </label>
    <label class="field">
      Auto-refresh (hours, 0 = manual)
      <input v-model.number="subscriptionHours" type="number" min="0" max="168" data-testid="tracking-refresh-hours" />
    </label>
    <div class="btn-row">
      <button type="button" class="refresh-blocklist" @click="() => void saveSubscription()" data-testid="tracking-save-subscription">
        Save subscription
      </button>
      <button type="button" class="refresh-blocklist secondary" @click="() => void updateBlocklist()" data-testid="tracking-update-blocklist">
        Update tracker blocklist now
      </button>
    </div>

    <h4>Safe Browsing online list</h4>
    <label class="field">
      Threat list URL (JSON <code>threats</code>)
      <input v-model="safeListUrl" type="url" placeholder="https://…/threats.json" />
    </label>
    <button type="button" class="refresh-blocklist" @click="() => void refreshSafeList()">
      Refresh Safe Browsing list
    </button>

    <h4>Encrypted sync (local vault)</h4>
    <p class="hint">
      AES-256-GCM encrypted backup on disk. Cloud sync server is optional; vault is ready for E2E upload.
    </p>
    <p v-if="encryptedSync.has_passphrase" class="hint">
      Vault unlocked · last backup
      {{
        encryptedSync.last_sync_at
          ? new Date(encryptedSync.last_sync_at * 1000).toLocaleString()
          : 'never'
      }}
    </p>
    <label class="field">
      Sync server URL
      <input v-model="syncServerUrl" type="url" placeholder="http://127.0.0.1:8787/api" />
    </label>
    <label class="field">
      Sync token (optional Bearer)
      <input v-model="syncToken" type="password" placeholder="API token" />
    </label>
    <button type="button" class="refresh-blocklist secondary" @click="() => void saveSyncServer()">
      Save sync server
    </button>
    <label class="field">
      Sync passphrase
      <input v-model="syncPassphrase" type="password" placeholder="Min 8 characters" />
    </label>
    <div class="btn-row">
      <button type="button" class="refresh-blocklist" @click="() => void enableEncryptedSync()">
        Set passphrase
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        :disabled="!encryptedSync.has_passphrase"
        @click="() => void backupBookmarksEncrypted()"
      >
        Encrypt bookmarks to vault
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        :disabled="!encryptedSync.has_passphrase || !syncServerUrl"
        @click="() => void uploadVault()"
      >
        Upload to cloud
      </button>
      <button
        type="button"
        class="refresh-blocklist secondary"
        :disabled="!encryptedSync.has_passphrase || !syncServerUrl"
        @click="() => void downloadVault()"
      >
        Download from cloud
      </button>
    </div>

    <p class="hint">Per-site: Shift+click the shield icon in the address bar to allow trackers on the current site.</p>
  </section>
  <p v-else class="hint">Loading privacy shields…</p>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Safe Browsing, tracking protection, and encrypted sync settings.
 */
import { ref, onMounted } from 'vue';
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

const emit = defineEmits<{
  status: [message: string];
  trackingChanged: [];
}>();

const loaded = ref(false);
const safeBrowsing = ref<SafeBrowsingSettings>({
  enabled: true,
  block_malware: true,
  block_phishing: true,
  block_unwanted_software: true,
  show_warnings: true,
  allow_proceed: true,
});
const tracking = ref<TrackingProtectionSettings>({
  enabled: true,
  block_advertising: true,
  block_analytics: true,
  block_social: false,
  block_fingerprinting: true,
  block_cryptomining: true,
  block_tracking: true,
});
const safeListUrl = ref('');
const subscriptionUrl = ref('');
const subscriptionHours = ref(24);
const syncPassphrase = ref('');
const syncServerUrl = ref('');
const syncToken = ref('');
const encryptedSync = ref<EncryptedSyncSettings>({
  enabled: false,
  has_passphrase: false,
  last_sync_at: 0,
  sync_server_url: null,
  sync_token: null,
  device_id: null,
});

async function load(): Promise<void> {
  try {
    safeBrowsing.value = await loadSafeBrowsingSettings();
    tracking.value = await loadTrackingProtectionSettings();
    encryptedSync.value = await loadEncryptedSyncSettings();
    syncServerUrl.value = encryptedSync.value.sync_server_url ?? '';
    syncToken.value = encryptedSync.value.sync_token ?? '';
    safeListUrl.value = safeBrowsing.value.list_url ?? '';
    subscriptionUrl.value = tracking.value.subscription_url ?? '';
    subscriptionHours.value = tracking.value.subscription_refresh_hours ?? 24;
    loaded.value = true;
  } catch (error) {
    console.error('PrivacyShieldSettings load failed:', error);
    emit('status', 'Failed to load privacy shield settings');
  }
}

async function persistSafe(): Promise<void> {
  try {
    await saveSafeBrowsingSettings(safeBrowsing.value);
    emit('status', 'Safe Browsing settings saved');
  } catch (error) {
    console.error('saveSafeBrowsingSettings failed:', error);
    emit('status', 'Failed to save Safe Browsing settings');
  }
}

async function persistTracking(): Promise<void> {
  try {
    await saveTrackingProtectionSettings(tracking.value);
    emit('trackingChanged');
    emit('status', 'Tracking protection settings saved');
  } catch (error) {
    console.error('saveTrackingProtectionSettings failed:', error);
    emit('status', 'Failed to save tracking protection settings');
  }
}

async function updateBlocklist(): Promise<void> {
  try {
    const count = await refreshTrackerBlocklist(subscriptionUrl.value || undefined);
    emit('status', `Tracker list updated (${count} domains)`);
  } catch (error) {
    console.error('refreshTrackerBlocklist failed:', error);
    emit('status', 'Failed to update tracker list');
  }
}

async function saveSubscription(): Promise<void> {
  try {
    await setTrackingSubscription(subscriptionUrl.value || null, subscriptionHours.value);
    await persistTracking();
    emit('status', 'Blocklist subscription saved');
  } catch (error) {
    console.error('setTrackingSubscription failed:', error);
    emit('status', 'Failed to save subscription');
  }
}

async function refreshSafeList(): Promise<void> {
  try {
    const next = { ...safeBrowsing.value, list_url: safeListUrl.value || null };
    await saveSafeBrowsingSettings(next);
    safeBrowsing.value = next;
    const added = await refreshSafeBrowsingList(safeListUrl.value || undefined);
    emit('status', `Safe Browsing list refreshed (+${added} threats)`);
  } catch (error) {
    console.error('refreshSafeBrowsingList failed:', error);
    emit('status', 'Failed to refresh Safe Browsing list');
  }
}

async function saveSyncServer(): Promise<void> {
  try {
    await setEncryptedSyncServer(syncServerUrl.value || null, syncToken.value || null);
    encryptedSync.value = await loadEncryptedSyncSettings();
    emit('status', 'Sync server settings saved');
  } catch (error) {
    console.error('setEncryptedSyncServer failed:', error);
    emit('status', 'Failed to save sync server');
  }
}

async function uploadVault(): Promise<void> {
  try {
    const msg = await uploadEncryptedVault();
    encryptedSync.value = await loadEncryptedSyncSettings();
    emit('status', msg);
  } catch (error) {
    console.error('uploadEncryptedVault failed:', error);
    emit('status', 'Cloud upload failed');
  }
}

async function downloadVault(): Promise<void> {
  try {
    const n = await downloadEncryptedVault();
    emit('status', `Downloaded vault (${n} bytes)`);
  } catch (error) {
    console.error('downloadEncryptedVault failed:', error);
    emit('status', 'Cloud download failed');
  }
}

async function enableEncryptedSync(): Promise<void> {
  if (syncPassphrase.value.length < 8) {
    emit('status', 'Passphrase must be at least 8 characters');
    return;
  }
  try {
    await setEncryptedSyncPassphrase(syncPassphrase.value);
    encryptedSync.value = await loadEncryptedSyncSettings();
    syncPassphrase.value = '';
    emit('status', 'Encrypted sync vault enabled');
  } catch (error) {
    console.error('setEncryptedSyncPassphrase failed:', error);
    emit('status', 'Failed to set sync passphrase');
  }
}

async function backupBookmarksEncrypted(): Promise<void> {
  try {
    const raw = localStorage.getItem('exodus-bookmarks');
    if (!raw) {
      emit('status', 'No local bookmarks to encrypt');
      return;
    }
    await storeEncryptedBookmarkVault(raw);
    emit('status', 'Bookmarks encrypted to local sync vault');
  } catch (error) {
    console.error('storeEncryptedBookmarkVault failed:', error);
    emit('status', 'Encrypted bookmark backup failed');
  }
}

onMounted(() => void load());
</script>

<style scoped>
.privacy-shield h3 {
  margin: 0 0 12px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}

.privacy-shield h4 {
  margin: 12px 0 8px;
  font-size: 14px;
  color: #e0e0e0;
}

.privacy-shield h4:first-of-type {
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
  margin-top: 6px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #6366f1;
  background: rgba(99, 102, 241, 0.15);
  color: #e0e7ff;
  cursor: pointer;
  font-size: 13px;
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
