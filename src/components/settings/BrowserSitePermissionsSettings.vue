<!--
  Exodus Browser — manage per-origin camera / microphone / geolocation grants.
-->
<template>
  <section id="settings-section-site-perms" class="settings-section site-perms-section" data-testid="site-permissions-settings">
    <h3>Site permissions</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <p class="hint">
        Per-origin browser decisions (separate from extension site access). Revoking resets the
        site so you will be asked again.
      </p>
      <div class="site-perms-actions">
        <button type="button" class="nav-button secondary" :disabled="loading" @click="() => void refresh()" data-testid="site-perms-refresh">
          Refresh
        </button>
      </div>
      <p v-if="entries.length === 0" class="hint">No saved site permissions.</p>
      <ul v-else class="site-perm-list">
        <li v-for="entry in entries" :key="`${entry.origin}:${entry.kind}`" class="site-perm-item">
          <div class="site-perm-meta">
            <strong>{{ entry.origin }}</strong>
            <span class="muted">
              {{ kindLabel(entry.kind) }} — {{ entry.granted ? 'Allowed' : 'Blocked' }}
            </span>
          </div>
          <button type="button" class="nav-button secondary" @click="() => void revokeEntry(entry)" data-testid="site-perm-reset">
            Reset
          </button>
        </li>
      </ul>
      <div v-if="uniqueOrigins.length > 0" class="origin-clear-row">
        <button
          v-for="origin in uniqueOrigins"
          :key="origin"
          type="button"
          class="nav-button secondary danger"
          @click="() => void revokeOrigin(origin)"
          data-testid="site-perm-clear-origin"
        >
          Clear all for {{ origin }}
        </button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — site permission grants list in Settings (Vue port).
 */
import { ref, computed, onMounted } from 'vue';
import {
  listBrowserSitePermissions,
  revokeBrowserSitePermission,
  type BrowserSitePermissionEntry,
} from '$lib/extensions/api';

const emit = defineEmits<{
  status: [message: string];
}>();

const entries = ref<BrowserSitePermissionEntry[]>([]);
const loading = ref(false);

const uniqueOrigins = computed(() => [...new Set(entries.value.map((e) => e.origin))]);

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
async function refresh(): Promise<void> {
  loading.value = true;
  try {
    entries.value = await listBrowserSitePermissions();
  } catch (error) {
    console.error('browser_site_permissions_list failed:', error);
    emit('status', 'Failed to load site permissions');
  } finally {
    loading.value = false;
  }
}

/** Remove one stored decision so the site will prompt again. */
async function revokeEntry(entry: BrowserSitePermissionEntry): Promise<void> {
  if (!confirm(`Reset ${kindLabel(entry.kind)} for ${entry.origin}?`)) return;
  try {
    await revokeBrowserSitePermission(entry.origin, [entry.kind]);
    await refresh();
    emit('status', `Removed ${kindLabel(entry.kind)} for ${entry.origin}`);
  } catch (error) {
    console.error('browser_site_permissions_revoke failed:', error);
    emit('status', 'Failed to revoke site permission');
  }
}

/** Clear all stored decisions for an origin. */
async function revokeOrigin(origin: string): Promise<void> {
  if (!confirm(`Clear all permissions for ${origin}?`)) return;
  try {
    await revokeBrowserSitePermission(origin);
    await refresh();
    emit('status', `Cleared all permissions for ${origin}`);
  } catch (error) {
    console.error('browser_site_permissions_revoke failed:', error);
    emit('status', 'Failed to clear site permissions');
  }
}

onMounted(() => void refresh());
</script>

<style scoped>
.site-perms-section h3 {
  margin: 0 0 12px;
  font-size: 14px;
  text-transform: uppercase;
  color: var(--color-text-secondary, #9ca3af);
}

.hint {
  font-size: 12px;
  color: var(--color-text-secondary, #888);
  margin: 0 0 8px;
}

.loading-state {
  padding: 20px;
  text-align: center;
  color: var(--color-text-secondary, #9ca3af);
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

.nav-button.secondary {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #505050;
  background: #404040;
  color: #e0e0e0;
  cursor: pointer;
  font-size: 12px;
}

.nav-button.danger {
  border-color: #7f1d1d;
  color: #fecaca;
}
</style>
