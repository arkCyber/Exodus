<template>
  <div class="extension-permission-manager">
    <div class="manager-header">
      <h2>Extension Permission Manager</h2>
      <p class="subtitle">Manage extension permissions and access controls</p>
    </div>

    <div class="manager-content">
      <!-- Extension Selector -->
      <div class="card">
        <h3>Select Extension</h3>
        <select v-model="selectedExtension" @change="loadExtensionPermissions" class="extension-select">
          <option value="">-- Select Extension --</option>
          <option v-for="ext in extensions" :key="ext.id" :value="ext.id">
            {{ ext.name }} ({{ ext.id }})
          </option>
        </select>
      </div>

      <!-- Permission Status -->
      <div v-if="selectedExtension" class="card">
        <h3>Permission Status</h3>
        <div class="permission-grid">
          <div
            v-for="permission in allPermissions"
            :key="permission.id"
            :class="['permission-card', getPermissionStatus(permission.id)]"
          >
            <div class="permission-header">
              <span class="permission-name">{{ permission.name }}</span>
              <span :class="['permission-badge', getPermissionStatus(permission.id)]">
                {{ getPermissionLabel(permission.id) }}
              </span>
            </div>
            <p class="permission-description">{{ permission.description }}</p>
            <div class="permission-actions">
              <button
                v-if="getPermissionStatus(permission.id) !== 'granted'"
                @click="grantPermission(permission.id)"
                class="btn btn-grant"
              >
                Grant
              </button>
              <button
                v-if="getPermissionStatus(permission.id) === 'granted'"
                @click="revokePermission(permission.id)"
                class="btn btn-revoke"
              >
                Revoke
              </button>
              <button
                v-if="getPermissionStatus(permission.id) === 'pending'"
                @click="denyPermission(permission.id)"
                class="btn btn-deny"
              >
                Deny
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Pending Requests -->
      <div v-if="pendingRequests.length > 0" class="card">
        <h3>Pending Permission Requests</h3>
        <div class="pending-list">
          <div
            v-for="request in pendingRequests"
            :key="request.id"
            class="pending-item"
          >
            <div class="pending-info">
              <span class="pending-extension">{{ request.extensionId }}</span>
              <span class="pending-permission">{{ request.permission }}</span>
            </div>
            <div class="pending-actions">
              <button @click="approveRequest(request)" class="btn btn-approve">
                Approve
              </button>
              <button @click="rejectRequest(request)" class="btn btn-reject">
                Reject
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Permission History -->
      <div v-if="selectedExtension" class="card">
        <h3>Permission History</h3>
        <div class="history-list">
          <div
            v-for="(event, index) in permissionHistory"
            :key="index"
            class="history-item"
          >
            <span class="history-time">{{ formatTime(event.timestamp) }}</span>
            <span class="history-action">{{ event.action }}</span>
            <span class="history-permission">{{ event.permission }}</span>
          </div>
        </div>
      </div>

      <!-- Global Settings -->
      <div class="card">
        <h3>Global Settings</h3>
        <div class="settings-list">
          <div class="setting-item">
            <label class="setting-label">
              <input
                type="checkbox"
                v-model="strictMode"
                @change="updateStrictMode"
              />
              Strict Mode (deny by default)
            </label>
            <p class="setting-description">
              When enabled, extensions must explicitly request and be granted permissions
            </p>
          </div>
          <div class="setting-item">
            <label class="setting-label">
              <input
                type="checkbox"
                v-model="showNotifications"
                @change="updateNotifications"
              />
              Show Permission Notifications
            </label>
            <p class="setting-description">
              Display notifications when extensions request permissions
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

// State
const selectedExtension = ref('');
const extensions = ref([
  { id: 'test-extension-1', name: 'Test Extension 1' },
  { id: 'test-extension-2', name: 'Test Extension 2' },
  { id: 'demo-extension', name: 'Demo Extension' },
]);

const allPermissions = ref([
  { id: 'contextMenus', name: 'Context Menus', description: 'Add items to browser context menu' },
  { id: 'webNavigation', name: 'Web Navigation', description: 'Monitor and control navigation events' },
  { id: 'sidePanel', name: 'Side Panel', description: 'Create and manage side panels' },
  { id: 'omnibox', name: 'Omnibox', description: 'Add address bar suggestions' },
  { id: 'identity', name: 'Identity', description: 'Authenticate users and manage tokens' },
  { id: 'topSites', name: 'Top Sites', description: 'Access most visited websites' },
  { id: 'devtools', name: 'DevTools', description: 'Interact with developer tools' },
  { id: 'siteIsolation', name: 'Site Isolation', description: 'Access site isolation information' },
]);

const extensionPermissions = ref<Record<string, string>>({});
const pendingRequests = ref<any[]>([]);
const permissionHistory = ref<any[]>([]);
const strictMode = ref(true);
const showNotifications = ref(true);

// Methods
const loadExtensionPermissions = async () => {
  if (!selectedExtension.value) return;

  try {
    // Mock API call
    extensionPermissions.value = {
      contextMenus: 'granted',
      topSites: 'granted',
      webNavigation: 'pending',
    };
  } catch (error) {
    console.error('Failed to load permissions:', error);
  }
};

const getPermissionStatus = (permissionId: string) => {
  return extensionPermissions.value[permissionId] || 'denied';
};

const getPermissionLabel = (permissionId: string) => {
  const status = getPermissionStatus(permissionId);
  const labels: Record<string, string> = {
    granted: 'Granted',
    denied: 'Denied',
    pending: 'Pending',
  };
  return labels[status] || 'Unknown';
};

const grantPermission = async (permissionId: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('permission_grant', {
      extensionId: selectedExtension.value,
      permission: permissionId,
    });
    extensionPermissions.value[permissionId] = 'granted';
    addHistoryEvent('Granted', permissionId);
  } catch (error) {
    console.error('Failed to grant permission:', error);
  }
};

const revokePermission = async (permissionId: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('permission_deny', {
      extensionId: selectedExtension.value,
      permission: permissionId,
    });
    extensionPermissions.value[permissionId] = 'denied';
    addHistoryEvent('Revoked', permissionId);
  } catch (error) {
    console.error('Failed to revoke permission:', error);
  }
};

const denyPermission = async (permissionId: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('permission_deny', {
      extensionId: selectedExtension.value,
      permission: permissionId,
    });
    extensionPermissions.value[permissionId] = 'denied';
    addHistoryEvent('Denied', permissionId);
  } catch (error) {
    console.error('Failed to deny permission:', error);
  }
};

const approveRequest = async (request: any) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('permission_approve_pending', {
      extensionId: request.extensionId,
      permission: request.permission,
    });
    pendingRequests.value = pendingRequests.value.filter(r => r.id !== request.id);
    addHistoryEvent('Approved', request.permission);
  } catch (error) {
    console.error('Failed to approve request:', error);
  }
};

const rejectRequest = async (request: any) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('permission_reject_pending', {
      extensionId: request.extensionId,
      permission: request.permission,
    });
    pendingRequests.value = pendingRequests.value.filter(r => r.id !== request.id);
    addHistoryEvent('Rejected', request.permission);
  } catch (error) {
    console.error('Failed to reject request:', error);
  }
};

const addHistoryEvent = (action: string, permission: string) => {
  permissionHistory.value.unshift({
    timestamp: Date.now(),
    action,
    permission,
  });
};

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
};

const updateStrictMode = () => {
  // Save setting
  console.log('Strict mode updated:', strictMode.value);
};

const updateNotifications = () => {
  // Save setting
  console.log('Notifications updated:', showNotifications.value);
};

onMounted(() => {
  // Load pending requests
  pendingRequests.value = [
    {
      id: '1',
      extensionId: 'test-extension-2',
      permission: 'webNavigation',
    },
  ];
});
</script>

<style scoped>
.extension-permission-manager {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.manager-header {
  margin-bottom: 30px;
}

.manager-header h2 {
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 8px;
}

.subtitle {
  color: #666;
  font-size: 14px;
}

.manager-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.card {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.card h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 15px;
  color: #333;
}

.extension-select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 14px;
}

.permission-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 15px;
}

.permission-card {
  padding: 15px;
  border-radius: 6px;
  border: 2px solid #e0e0e0;
  background: #f8f9fa;
}

.permission-card.granted {
  border-color: #28a745;
  background: #d4edda;
}

.permission-card.denied {
  border-color: #dc3545;
  background: #f8d7da;
}

.permission-card.pending {
  border-color: #ffc107;
  background: #fff3cd;
}

.permission-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.permission-name {
  font-weight: 600;
  color: #333;
}

.permission-badge {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
}

.permission-badge.granted {
  background: #28a745;
  color: white;
}

.permission-badge.denied {
  background: #dc3545;
  color: white;
}

.permission-badge.pending {
  background: #ffc107;
  color: #333;
}

.permission-description {
  font-size: 13px;
  color: #666;
  margin-bottom: 12px;
  line-height: 1.4;
}

.permission-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 6px 12px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.btn-grant {
  background: #28a745;
  color: white;
}

.btn-grant:hover {
  background: #218838;
}

.btn-revoke {
  background: #dc3545;
  color: white;
}

.btn-revoke:hover {
  background: #c82333;
}

.btn-deny {
  background: #6c757d;
  color: white;
}

.btn-deny:hover {
  background: #545b62;
}

.btn-approve {
  background: #28a745;
  color: white;
}

.btn-reject {
  background: #dc3545;
  color: white;
}

.pending-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.pending-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 4px;
}

.pending-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pending-extension {
  font-weight: 600;
  color: #333;
}

.pending-permission {
  font-size: 13px;
  color: #666;
}

.pending-actions {
  display: flex;
  gap: 8px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  gap: 12px;
  padding: 8px 12px;
  background: #f8f9fa;
  border-radius: 4px;
  font-size: 13px;
}

.history-time {
  color: #666;
  min-width: 80px;
}

.history-action {
  font-weight: 600;
  color: #333;
  min-width: 80px;
}

.history-permission {
  color: #666;
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #333;
  cursor: pointer;
}

.setting-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.setting-description {
  font-size: 13px;
  color: #666;
  margin-left: 26px;
}
</style>
