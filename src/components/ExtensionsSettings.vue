<!--
  Exodus Browser — Web Extensions management (chrome://settings/extensions, Chrome parity).
-->
<template>
  <div class="settings-section extensions-settings" data-testid="extensions-settings-panel">
    <header class="extensions-settings__header">
      <div>
        <h3>{{ ui.pageTitle }}</h3>
        <p class="settings-hint">{{ ui.pageHint }}</p>
      </div>
      <button
        type="button"
        class="nav-button secondary"
        data-testid="extensions-open-apps"
        @click="emit('openApps')"
      >
        {{ ui.openApps }}
      </button>
    </header>

    <div class="settings-card extensions-settings__card">
      <h4 class="settings-card__title">{{ ui.preferencesTitle }}</h4>
      <label class="checkbox-row">
        <input
          v-model="confirmHostOnInstall"
          type="checkbox"
          data-testid="extensions-confirm-host"
          @change="() => void toggleConfirmHostOnInstall()"
        />
        <span>{{ ui.confirmHostLabel }}</span>
      </label>
      <label class="store-url-row">
        {{ ui.storeUrlLabel }}
        <input
          v-model="storeUrl"
          type="url"
          class="field"
          data-testid="extensions-store-url"
          :placeholder="ui.storeUrlPlaceholder"
        />
        <button
          type="button"
          class="nav-button secondary"
          data-testid="extensions-save-store-url"
          @click="() => void saveStoreUrl()"
        >
          {{ ui.saveStoreUrl }}
        </button>
      </label>
    </div>

    <div class="settings-card extensions-settings__card">
      <h4 class="settings-card__title">{{ ui.actionsTitle }}</h4>
      <div class="extensions-actions">
        <button
          type="button"
          class="nav-button secondary"
          data-testid="extensions-refresh"
          :disabled="loading"
          @click="() => void refresh()"
        >
          {{ ui.refresh }}
        </button>
        <button type="button" class="nav-button secondary" data-testid="extensions-rescan" @click="() => void handleRescan()">
          {{ ui.rescan }}
        </button>
        <button type="button" class="nav-button secondary" data-testid="extensions-install-folder" @click="() => void handleInstall()">
          {{ ui.installFolder }}
        </button>
        <button type="button" class="nav-button secondary" data-testid="extensions-install-crx" @click="() => void handleInstallCrx()">
          {{ ui.installCrx }}
        </button>
      </div>
    </div>

    <div v-if="storeItems.length > 0" class="settings-card extensions-settings__card">
      <h4 class="settings-card__title">{{ ui.storeTitle }}</h4>
      <ul class="extension-list compact" data-testid="extensions-store-list">
        <li v-for="item in storeItems" :key="item.id" class="extension-item">
          <div class="extension-meta">
            <strong>{{ item.name }}</strong>
            <span class="muted">v{{ item.version }} · {{ item.id }}</span>
            <span v-if="item.description" class="muted">{{ item.description }}</span>
          </div>
          <button
            type="button"
            class="nav-button secondary"
            :disabled="item.installed"
            @click="() => void installStoreItem(item)"
          >
            {{ item.installed ? ui.installedBadge : ui.install }}
          </button>
        </li>
      </ul>
    </div>

    <div class="settings-card extensions-settings__card">
      <div class="extensions-installed__head">
        <h4 class="settings-card__title">{{ ui.installedTitle }}</h4>
        <input
          v-model="searchQuery"
          type="search"
          class="field extensions-search"
          data-testid="extensions-search"
          :placeholder="ui.searchPlaceholder"
        />
      </div>
      <p v-if="loading" class="settings-hint" data-testid="extensions-loading">{{ ui.loading }}</p>
      <p v-else-if="filteredExtensions.length === 0" class="settings-hint" data-testid="extensions-empty">
        {{ ui.emptyInstalled }}
      </p>
      <ul v-else class="extension-list" data-testid="extensions-installed-list">
        <li v-for="ext in filteredExtensions" :key="ext.id" class="extension-item" :data-extension-id="ext.id">
          <div class="extension-row">
            <div class="extension-meta">
              <strong>{{ ext.name }}</strong>
              <span class="muted">v{{ ext.version }} · {{ ext.id }}</span>
              <span v-if="ext.description" class="muted">{{ ext.description }}</span>
              <span class="muted">{{ ui.permissions(ext.permissions.join(', ')) }}</span>
              <span v-if="ext.actionPopup" class="muted">{{ ui.popup(ext.actionPopup) }}</span>
            </div>
            <div class="extension-buttons">
              <label class="checkbox-row compact">
                <input type="checkbox" :checked="ext.enabled" @change="() => void toggleEnabled(ext)" />
                <span>{{ ui.enabled }}</span>
              </label>
              <label class="checkbox-row compact">
                <input
                  type="checkbox"
                  :checked="ext.pinned !== false"
                  @change="() => void togglePinned(ext)"
                />
                <span>{{ ui.pinToolbar }}</span>
              </label>
              <button type="button" class="nav-button secondary" @click="() => void toggleSiteAccess(ext)">
                {{ expandedSiteAccessId === ext.id ? ui.hideSiteAccess : ui.siteAccess }}
              </button>
              <button type="button" class="nav-button secondary danger" @click="() => void handleUninstall(ext)">
                {{ ui.uninstall }}
              </button>
            </div>
          </div>
          <div v-if="expandedSiteAccessId === ext.id" class="host-patterns-panel">
            <p v-if="hostPatternsLoading === ext.id" class="settings-hint">{{ ui.loadingSites }}</p>
            <p v-else-if="(hostPatternsByExt[ext.id] ?? []).length === 0" class="settings-hint">
              {{ ui.noSites }}
            </p>
            <template v-else>
              <div class="host-patterns-toolbar">
                <button
                  type="button"
                  class="nav-button secondary danger"
                  @click="() => void revokeAllHostPatterns(ext.id, ext.name)"
                >
                  {{ ui.revokeAll }}
                </button>
              </div>
              <ul class="host-pattern-list">
                <li
                  v-for="pattern in hostPatternsByExt[ext.id] ?? []"
                  :key="pattern"
                  class="host-pattern-row"
                >
                  <code>{{ pattern }}</code>
                  <button
                    type="button"
                    class="nav-button secondary"
                    @click="() => void revokeHostPattern(ext.id, pattern)"
                  >
                    {{ ui.revoke }}
                  </button>
                </li>
              </ul>
            </template>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Web Extensions management (Settings → Extensions, chrome://extensions).
 * Aerospace-level error handling, security validation, and concurrency safety.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ExtensionInfo, StoreExtensionEntry } from '$lib/extensions/types';
import type { ExodusConfigDto } from '$lib/browserSettings';
import {
  installExtensionFolder,
  installExtensionCrx,
  listExtensions,
  listStoreExtensions,
  fetchRemoteStoreExtensions,
  rescanExtensions,
  setExtensionEnabled,
  setExtensionPinned,
  uninstallExtension,
  setConfirmHostPermissionsOnInstall,
  listExtensionSitePermissions,
  revokeExtensionSitePermissions,
  revokeAllExtensionSitePermissions,
} from '$lib/extensions/api';
import { ensureExtensionBackgrounds } from '$lib/extensions/backgroundHosts';
import { extensionsSettingsStrings } from '@/lib/extensionsSettingsUi';
import type { AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  contentHost?: HTMLElement;
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
  extensionsChanged: [];
  openApps: [];
}>();

const ui = computed(() => extensionsSettingsStrings(props.uiLocale));

const extensions = ref<ExtensionInfo[]>([]);
const storeItems = ref<StoreExtensionEntry[]>([]);
const loading = ref(false);
const storeUrl = ref('');
const confirmHostOnInstall = ref(true);
const searchQuery = ref('');
const expandedSiteAccessId = ref<string | null>(null);
const hostPatternsByExt = ref<Record<string, string[]>>({});
const hostPatternsLoading = ref<string | null>(null);

// Aerospace-level concurrency safety
let isLoadingExtensions = false;
let pendingOperations = new Set<string>();

// Aerospace-level security validation patterns
const VALID_EXTENSION_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+/;

const filteredExtensions = computed(() => {
  // Aerospace-level safety check to prevent undefined errors
  if (!extensions.value || !Array.isArray(extensions.value)) {
    return [];
  }
  
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return extensions.value;
  return extensions.value.filter(
    (ext) =>
      ext.name.toLowerCase().includes(q) ||
      ext.id.toLowerCase().includes(q) ||
      (ext.description?.toLowerCase().includes(q) ?? false),
  );
});

function onStatus(message: string): void {
  emit('status', message);
}

/**
 * Reload extension list from the backend (local + optional remote catalog).
 * Aerospace-level error handling with graceful degradation and concurrency safety.
 */
async function refresh(): Promise<void> {
  // Aerospace-level concurrency check
  if (isLoadingExtensions) {
    console.warn('[ExtensionsSettings] Refresh already in progress, skipping duplicate request');
    return;
  }
  
  isLoadingExtensions = true;
  loading.value = true;
  
  try {
    extensions.value = await listExtensions();
    const local = await listStoreExtensions();
    let remote: StoreExtensionEntry[] = [];
    try {
      remote = await fetchRemoteStoreExtensions();
    } catch {
      /* remote catalog optional - log but don't fail */
      console.warn('[ExtensionsSettings] Remote catalog fetch failed, using local only');
    }
    const byId = new Map<string, StoreExtensionEntry>();
    for (const item of [...local, ...remote]) {
      byId.set(item.id, item);
    }
    storeItems.value = [...byId.values()].sort((a, b) => a.name.localeCompare(b.name));
  } catch (error) {
    console.error('[ExtensionsSettings] Extension list refresh failed:', error);
    onStatus('Failed to load extensions');
    // Graceful degradation: show empty state rather than crash
    extensions.value = [];
    storeItems.value = [];
  } finally {
    loading.value = false;
    isLoadingExtensions = false;
  }
}

/**
 * Reboot extension background hosts.
 * Aerospace-level error handling with graceful degradation.
 */
async function rebootBackgrounds(): Promise<void> {
  if (props.contentHost) {
    try {
      await ensureExtensionBackgrounds(props.contentHost);
    } catch (error) {
      console.error('[ExtensionsSettings] Background reboot failed:', error);
      // Non-critical: continue without background reboot
    }
  }
}

/**
 * Toggle extension enabled state.
 * Aerospace-level error handling with input validation and concurrency safety.
 */
async function toggleEnabled(ext: ExtensionInfo): Promise<void> {
  // Aerospace-level input validation
  if (!ext || !ext.id) {
    console.error('[ExtensionsSettings] Invalid extension data for toggleEnabled');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(ext.id)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', ext.id);
    return;
  }
  
  // Aerospace-level concurrency check
  const operationKey = `toggle-enabled-${ext.id}`;
  if (pendingOperations.has(operationKey)) {
    console.warn('[ExtensionsSettings] Toggle enabled already pending for:', ext.id);
    return;
  }
  
  pendingOperations.add(operationKey);
  
  try {
    await setExtensionEnabled(ext.id, !ext.enabled);
    await refresh();
    await rebootBackgrounds();
    emit('extensionsChanged');
    onStatus(ext.enabled ? `Disabled ${ext.name}` : `Enabled ${ext.name}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_set_enabled failed:', error);
    onStatus('Failed to toggle extension');
  } finally {
    pendingOperations.delete(operationKey);
  }
}

/**
 * Toggle extension pinned state.
 * Aerospace-level error handling with input validation and concurrency safety.
 */
async function togglePinned(ext: ExtensionInfo): Promise<void> {
  // Aerospace-level input validation
  if (!ext || !ext.id) {
    console.error('[ExtensionsSettings] Invalid extension data for togglePinned');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(ext.id)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', ext.id);
    return;
  }
  
  // Aerospace-level concurrency check
  const operationKey = `toggle-pinned-${ext.id}`;
  if (pendingOperations.has(operationKey)) {
    console.warn('[ExtensionsSettings] Toggle pinned already pending for:', ext.id);
    return;
  }
  
  pendingOperations.add(operationKey);
  
  try {
    const next = ext.pinned === false;
    await setExtensionPinned(ext.id, next);
    await refresh();
    emit('extensionsChanged');
    onStatus(next ? `Pinned ${ext.name}` : `Unpinned ${ext.name}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_set_pinned failed:', error);
    onStatus('Failed to update toolbar pin');
  } finally {
    pendingOperations.delete(operationKey);
  }
}

/**
 * Rescan extension directories.
 * Aerospace-level error handling with user feedback.
 */
async function handleRescan(): Promise<void> {
  try {
    const count = await rescanExtensions();
    await refresh();
    await rebootBackgrounds();
    emit('extensionsChanged');
    onStatus(ui.value.statusRescanned(count));
  } catch (error) {
    console.error('[ExtensionsSettings] extension_rescan failed:', error);
    onStatus('Rescan failed');
  }
}

/**
 * Install extension from folder.
 * Aerospace-level error handling with input validation and security checks.
 */
async function handleInstall(): Promise<void> {
  const path = window.prompt('Path to unpacked extension folder:');
  if (!path?.trim()) return;
  
  const trimmedPath = path.trim();
  
  // Aerospace-level security validation: prevent path traversal
  if (trimmedPath.includes('..') || trimmedPath.includes('~')) {
    console.error('[ExtensionsSettings] Invalid path with traversal characters:', trimmedPath);
    onStatus('Invalid path: path traversal not allowed');
    return;
  }
  
  try {
    await installExtensionFolder(trimmedPath);
    await refresh();
    await rebootBackgrounds();
    emit('extensionsChanged');
    onStatus('Extension folder installed');
  } catch (error) {
    console.error('[ExtensionsSettings] extension_install_folder failed:', error);
    onStatus(`Install failed: ${error}`);
  }
}

/**
 * Install extension from CRX/ZIP file.
 * Aerospace-level error handling with input validation and security checks.
 */
async function handleInstallCrx(): Promise<void> {
  const path = window.prompt('Path to .crx or .zip file:');
  if (!path?.trim()) return;
  
  const trimmedPath = path.trim();
  
  // Aerospace-level security validation: prevent path traversal
  if (trimmedPath.includes('..') || trimmedPath.includes('~')) {
    console.error('[ExtensionsSettings] Invalid path with traversal characters:', trimmedPath);
    onStatus('Invalid path: path traversal not allowed');
    return;
  }
  
  // Aerospace-level file extension validation
  if (!trimmedPath.endsWith('.crx') && !trimmedPath.endsWith('.zip')) {
    console.error('[ExtensionsSettings] Invalid file extension:', trimmedPath);
    onStatus('Invalid file: must be .crx or .zip');
    return;
  }
  
  try {
    await installExtensionCrx(trimmedPath);
    await refresh();
    await rebootBackgrounds();
    onStatus('Extension package installed');
  } catch (error) {
    console.error('[ExtensionsSettings] extension_install_crx failed:', error);
    onStatus(`CRX install failed: ${error}`);
  }
}

/**
 * Install extension from store catalog.
 * Aerospace-level error handling with input validation.
 */
async function installStoreItem(item: StoreExtensionEntry): Promise<void> {
  // Aerospace-level input validation
  if (!item || !item.path) {
    console.error('[ExtensionsSettings] Invalid store item data');
    return;
  }
  
  // Aerospace-level security validation: prevent path traversal
  if (item.path.includes('..') || item.path.includes('~')) {
    console.error('[ExtensionsSettings] Invalid store item path with traversal characters:', item.path);
    onStatus('Invalid store item path');
    return;
  }
  
  try {
    await installExtensionFolder(item.path);
    await refresh();
    await rebootBackgrounds();
    emit('extensionsChanged');
    onStatus(`Installed ${item.name}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_install_folder failed:', error);
    onStatus(`Install failed: ${error}`);
  }
}

/**
 * Uninstall extension.
 * Aerospace-level error handling with input validation and security checks.
 */
async function handleUninstall(ext: ExtensionInfo): Promise<void> {
  // Aerospace-level input validation
  if (!ext || !ext.id) {
    console.error('[ExtensionsSettings] Invalid extension data for uninstall');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(ext.id)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', ext.id);
    return;
  }
  
  if (!confirm(`Uninstall extension "${ext.name}"?`)) return;
  
  try {
    await uninstallExtension(ext.id);
    await refresh();
    emit('extensionsChanged');
    onStatus(`Uninstalled ${ext.name}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_uninstall failed:', error);
    onStatus('Uninstall failed');
  }
}

/**
 * Load extension settings from backend.
 * Aerospace-level error handling with graceful degradation.
 */
async function loadExtensionSettings(): Promise<void> {
  try {
    const cfg = await invoke<ExodusConfigDto>('get_ai_config');
    storeUrl.value = cfg.extension_store_url ?? '';
    confirmHostOnInstall.value = cfg.confirm_host_permissions_on_install ?? true;
  } catch (error) {
    console.error('[ExtensionsSettings] loadExtensionSettings failed:', error);
    // Graceful degradation: use safe defaults
    storeUrl.value = '';
    confirmHostOnInstall.value = true;
  }
}

/**
 * Toggle confirm host permissions on install setting.
 * Aerospace-level error handling with user feedback.
 */
async function toggleConfirmHostOnInstall(): Promise<void> {
  try {
    await setConfirmHostPermissionsOnInstall(confirmHostOnInstall.value);
    onStatus(confirmHostOnInstall.value ? ui.value.statusConfirmOn : ui.value.statusConfirmOff);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_set_confirm_host_permissions failed:', error);
    onStatus('Failed to save host permission setting');
  }
}

/**
 * Load host patterns for extension site access.
 * Aerospace-level error handling with input validation.
 */
async function loadHostPatterns(extensionId: string): Promise<void> {
  // Aerospace-level input validation
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ExtensionsSettings] Invalid extension ID for loadHostPatterns');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(extensionId)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', extensionId);
    return;
  }
  
  hostPatternsLoading.value = extensionId;
  try {
    const patterns = await listExtensionSitePermissions(extensionId);
    hostPatternsByExt.value = { ...hostPatternsByExt.value, [extensionId]: patterns };
  } catch (error) {
    console.error('[ExtensionsSettings] extension_site_permissions_list failed:', error);
    onStatus('Failed to load extension site access');
  } finally {
    hostPatternsLoading.value = null;
  }
}

/**
 * Toggle site access panel visibility.
 * Aerospace-level input validation.
 */
async function toggleSiteAccess(ext: ExtensionInfo): Promise<void> {
  // Aerospace-level input validation
  if (!ext || !ext.id) {
    console.error('[ExtensionsSettings] Invalid extension data for toggleSiteAccess');
    return;
  }
  
  if (expandedSiteAccessId.value === ext.id) {
    expandedSiteAccessId.value = null;
    return;
  }
  expandedSiteAccessId.value = ext.id;
  if (!hostPatternsByExt.value[ext.id]) {
    await loadHostPatterns(ext.id);
  }
}

/**
 * Revoke all host patterns for extension.
 * Aerospace-level error handling with input validation and security checks.
 */
async function revokeAllHostPatterns(extensionId: string, extName: string): Promise<void> {
  // Aerospace-level input validation
  if (!extensionId || !extName) {
    console.error('[ExtensionsSettings] Invalid parameters for revokeAllHostPatterns');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(extensionId)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', extensionId);
    return;
  }
  
  const patterns = hostPatternsByExt.value[extensionId] ?? [];
  if (patterns.length === 0) return;
  if (!confirm(`Revoke all site access for "${extName}" (${patterns.length} patterns)?`)) return;
  
  try {
    await revokeAllExtensionSitePermissions(extensionId);
    await loadHostPatterns(extensionId);
    onStatus(`Revoked all site access for ${extName}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_site_permissions_revoke_all failed:', error);
    onStatus('Failed to revoke all site access');
  }
}

/**
 * Revoke specific host pattern for extension.
 * Aerospace-level error handling with input validation and security checks.
 */
async function revokeHostPattern(extensionId: string, pattern: string): Promise<void> {
  // Aerospace-level input validation
  if (!extensionId || !pattern) {
    console.error('[ExtensionsSettings] Invalid parameters for revokeHostPattern');
    return;
  }
  
  // Aerospace-level security validation
  if (!VALID_EXTENSION_ID_PATTERN.test(extensionId)) {
    console.error('[ExtensionsSettings] Invalid extension ID format:', extensionId);
    return;
  }
  
  // Aerospace-level pattern validation: prevent malicious patterns
  if (pattern.includes('..') || pattern.includes('~')) {
    console.error('[ExtensionsSettings] Invalid pattern with traversal characters:', pattern);
    onStatus('Invalid pattern: traversal not allowed');
    return;
  }
  
  try {
    await revokeExtensionSitePermissions(extensionId, [pattern]);
    await loadHostPatterns(extensionId);
    onStatus(`Revoked site access: ${pattern}`);
  } catch (error) {
    console.error('[ExtensionsSettings] extension_site_permissions_revoke failed:', error);
    onStatus('Failed to revoke site access');
  }
}

/**
 * Save extension store URL.
 * Aerospace-level error handling with URL validation and security checks.
 */
async function saveStoreUrl(): Promise<void> {
  const trimmedUrl = storeUrl.value.trim();
  
  // Aerospace-level URL validation
  if (trimmedUrl && !VALID_URL_PATTERN.test(trimmedUrl)) {
    console.error('[ExtensionsSettings] Invalid URL format:', trimmedUrl);
    onStatus('Invalid URL: must start with http:// or https://');
    return;
  }
  
  try {
    await invoke('extension_set_store_url', { url: trimmedUrl });
    onStatus(ui.value.statusStoreSaved);
    await refresh();
  } catch (error) {
    console.error('[ExtensionsSettings] extension_set_store_url failed:', error);
    onStatus('Failed to save store URL');
  }
}

/**
 * Cleanup on component unmount.
 * Aerospace-level memory management and resource cleanup.
 */
onUnmounted(() => {
  // Clear pending operations
  pendingOperations.clear();
  
  // Reset loading states
  isLoadingExtensions = false;
  hostPatternsLoading.value = null;
  
  // Close expanded panels
  expandedSiteAccessId.value = null;
});

onMounted(() => {
  void loadExtensionSettings();
  void refresh();
});
</script>

<style scoped>
.extensions-settings__header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 16px;
}

.extensions-settings__header h3 {
  margin: 0 0 6px;
  font-size: 18px;
  font-weight: 500;
}

.extensions-settings__card {
  margin-bottom: 16px;
}

.extensions-settings__card .settings-card__title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 500;
}

.store-url-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 0;
  font-size: 13px;
}

.extensions-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.extensions-installed__head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.extensions-installed__head .settings-card__title {
  margin: 0;
}

.extensions-search {
  max-width: 280px;
  flex: 1;
  min-width: 160px;
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
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.extension-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 14px;
  border: 1px solid var(--cs-border, #e8eaed);
  border-radius: 8px;
  background: var(--cs-muted-surface, #f8f9fa);
}

.extension-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.extension-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  min-width: 0;
}

.extension-meta strong {
  font-size: 14px;
}

.extension-buttons {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.host-patterns-panel {
  width: 100%;
  padding-top: 8px;
  border-top: 1px solid var(--cs-border, #e8eaed);
}

.host-pattern-list {
  list-style: none;
  margin: 8px 0 0;
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
}

.muted {
  color: var(--cs-hint, #5f6368);
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  font-size: 13px;
  cursor: pointer;
}

.checkbox-row.compact {
  margin: 0;
}

/* Buttons inherit from ChromeSettingsPage :deep(.nav-button) */
</style>
