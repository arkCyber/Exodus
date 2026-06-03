<template>
  <div class="extension-permission-request">
    <div class="header">
      <h2>Extension Permission Request</h2>
      <button @click="close" class="close-btn">×</button>
    </div>

    <div class="content">
      <div class="extension-info">
        <div class="icon" v-if="extension.icon">
          <img :src="extension.icon" :alt="extension.name" />
        </div>
        <div class="details">
          <h3>{{ extension.name }}</h3>
          <p class="version">Version {{ extension.version }}</p>
          <p class="description">{{ extension.description }}</p>
        </div>
      </div>

      <div class="permissions-section">
        <h4>Requested Permissions</h4>
        <div class="permissions-list">
          <div
            v-for="permission in requestedPermissions"
            :key="permission.id"
            class="permission-item"
          >
            <div class="permission-icon">
              <span class="icon">{{ getPermissionIcon(permission.id) }}</span>
            </div>
            <div class="permission-details">
              <h5>{{ permission.name }}</h5>
              <p>{{ permission.description }}</p>
            </div>
            <div class="permission-status">
              <span :class="['status', permission.status]">
                {{ permission.status }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div class="host-permissions-section" v-if="hostPermissions.length > 0">
        <h4>Host Permissions</h4>
        <div class="host-permissions-list">
          <div
            v-for="host in hostPermissions"
            :key="host"
            class="host-permission-item"
          >
            <span class="host-icon">🌐</span>
            <span class="host-pattern">{{ host }}</span>
          </div>
        </div>
      </div>

      <div class="warning-section" v-if="hasDangerousPermissions">
        <div class="warning-box">
          <span class="warning-icon">⚠️</span>
          <p>
            This extension requests permissions that could access your data on
            all websites. Only install if you trust the developer.
          </p>
        </div>
      </div>
    </div>

    <div class="actions">
      <button @click="denyAll" class="btn btn-secondary">Deny All</button>
      <button @click="approveSelected" class="btn btn-primary">
        Approve Selected
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';

interface Permission {
  id: string;
  name: string;
  description: string;
  status: 'pending' | 'approved' | 'denied';
}

interface Extension {
  id: string;
  name: string;
  version: string;
  description: string;
  icon?: string;
}

const props = defineProps<{
  extension: Extension;
  requestedPermissions: Permission[];
  hostPermissions: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'approve', permissions: string[]): void;
  (e: 'deny', permissions: string[]): void;
}>();

const hasDangerousPermissions = computed(() => {
  return props.requestedPermissions.some(
    (p) => p.id === 'tabs' || p.id === 'bookmarks' || p.id === 'history'
  );
});

const getPermissionIcon = (id: string): string => {
  const icons: Record<string, string> = {
    contextMenus: '📋',
    webNavigation: '🧭',
    sidePanel: '📊',
    omnibox: '🔍',
    identity: '🔐',
    topSites: '⭐',
    devtools: '🛠️',
    siteIsolation: '🔒',
    storage: '💾',
    tabs: '📑',
    bookmarks: '🔖',
    notifications: '🔔',
    messaging: '💬',
  };
  return icons[id] || '📌';
};

const close = () => {
  emit('close');
};

const approveSelected = () => {
  const approved = props.requestedPermissions
    .filter((p) => p.status === 'pending' || p.status === 'approved')
    .map((p) => p.id);
  emit('approve', approved);
};

const denyAll = () => {
  const allPermissions = props.requestedPermissions.map((p) => p.id);
  emit('deny', allPermissions);
};

onMounted(() => {
  // Auto-approve safe permissions
  props.requestedPermissions.forEach((permission) => {
    if (
      ['storage', 'contextMenus', 'notifications'].includes(permission.id)
    ) {
      permission.status = 'approved';
    }
  });
});
</script>

<style scoped>
.extension-permission-request {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  width: 600px;
  max-width: 90vw;
  max-height: 90vh;
  overflow: hidden;
  z-index: 1000;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
}

.header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 28px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: var(--bg-hover);
}

.content {
  padding: 24px;
  overflow-y: auto;
  max-height: calc(90vh - 140px);
}

.extension-info {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.icon {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  overflow: hidden;
  flex-shrink: 0;
}

.icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.details h3 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.version {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--text-secondary);
}

.description {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.permissions-section,
.host-permissions-section {
  margin-bottom: 24px;
}

.permissions-section h4,
.host-permissions-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.permissions-list,
.host-permissions-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.permission-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  transition: background 0.2s;
}

.permission-item:hover {
  background: var(--bg-hover);
}

.permission-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  border-radius: 8px;
  font-size: 18px;
}

.permission-details {
  flex: 1;
}

.permission-details h5 {
  margin: 0 0 4px 0;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.permission-details p {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.permission-status {
  flex-shrink: 0;
}

.status {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  text-transform: capitalize;
}

.status.pending {
  background: var(--warning-bg);
  color: var(--warning-text);
}

.status.approved {
  background: var(--success-bg);
  color: var(--success-text);
}

.status.denied {
  background: var(--error-bg);
  color: var(--error-text);
}

.host-permission-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.host-icon {
  font-size: 16px;
}

.host-pattern {
  font-size: 13px;
  color: var(--text-primary);
  font-family: monospace;
}

.warning-section {
  margin-bottom: 24px;
}

.warning-box {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px;
  background: var(--warning-bg);
  border: 1px solid var(--warning-border);
  border-radius: 8px;
}

.warning-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.warning-box p {
  margin: 0;
  font-size: 13px;
  color: var(--warning-text);
  line-height: 1.5;
}

.actions {
  display: flex;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
}

.btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover {
  background: var(--primary-hover);
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--bg-hover);
}
</style>
