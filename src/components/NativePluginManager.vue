<template>
  <div class="native-plugin-manager">
    <div class="header">
      <h2>Native Plugins</h2>
      <div class="actions">
        <button @click="showAuditLogDialog = true" class="btn btn-secondary">
          <FileText :size="16" />
          Audit Log
        </button>
        <button @click="showSandboxDialog = true" class="btn btn-info">
          <Shield :size="16" />
          Sandbox Config
        </button>
        <button @click="reloadPlugins" class="btn btn-secondary">
          <RefreshCw :size="16" />
          Reload Changed
        </button>
        <button @click="scanPlugins" class="btn btn-primary">
          <Package :size="16" />
          Scan Plugins
        </button>
        <button @click="showLoadDialog = true" class="btn btn-secondary">
          <Plus :size="16" />
          Load Plugin
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">
      <Loader2 class="spinner" :size="24" />
      <span>Loading plugins...</span>
    </div>

    <div v-else-if="error" class="error">
      <AlertCircle :size="20" />
      <span>{{ error }}</span>
    </div>

    <div v-else-if="plugins.length === 0" class="empty">
      <Package :size="48" />
      <p>No native plugins loaded</p>
      <p class="hint">Load a plugin or scan the plugins directory</p>
    </div>

    <div v-else class="plugin-list">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="plugin-item"
        :class="{ disabled: !plugin.enabled }"
      >
        <div class="plugin-info">
          <div class="plugin-header">
            <h3>{{ plugin.name }}</h3>
            <span class="version">v{{ plugin.version }}</span>
          </div>
          <p class="description">{{ plugin.description }}</p>
          <div class="meta">
            <span class="author">by {{ plugin.author }}</span>
            <span class="id">{{ plugin.id }}</span>
          </div>
          <div class="permissions">
            <span
              v-for="permission in plugin.permissions"
              :key="permission"
              class="permission-tag"
              :class="{ sensitive: isSensitivePermission(permission) }"
            >
              {{ permission }}
            </span>
          </div>
        </div>

        <div class="plugin-actions">
          <button
            @click="togglePlugin(plugin.id, !plugin.enabled)"
            class="btn btn-sm"
            :class="plugin.enabled ? 'btn-warning' : 'btn-success'"
          >
            {{ plugin.enabled ? 'Disable' : 'Enable' }}
          </button>
          <button
            @click="showCommandDialog(plugin)"
            class="btn btn-sm btn-info"
            :disabled="!plugin.enabled"
          >
            <Terminal :size="14" />
            Commands
          </button>
          <button
            @click="showResourceStats(plugin)"
            class="btn btn-sm btn-secondary"
            :disabled="!plugin.enabled"
          >
            <BarChart :size="14" />
            Stats
          </button>
          <button
            @click="unloadPlugin(plugin.id)"
            class="btn btn-sm btn-danger"
          >
            <Trash2 :size="14" />
            Unload
          </button>
        </div>
      </div>
    </div>

    <!-- Load Plugin Dialog -->
    <Dialog v-model:open="showLoadDialog" title="Load Native Plugin">
      <div class="dialog-content">
        <div class="form-group">
          <label>Plugin Path</label>
          <input
            v-model="pluginPath"
            type="text"
            placeholder="/path/to/plugin.dylib"
            class="input"
          />
        </div>
        <div class="dialog-actions">
          <button @click="showLoadDialog = false" class="btn btn-secondary">Cancel</button>
          <button @click="loadPlugin" class="btn btn-primary" :disabled="!pluginPath">
            Load
          </button>
        </div>
      </div>
    </Dialog>

    <!-- Command Dialog -->
    <Dialog v-model:open="showCommandDialogOpen" :title="`Commands - ${selectedPlugin?.name}`">
      <div class="dialog-content">
        <div v-if="selectedPlugin" class="command-interface">
          <div class="form-group">
            <label>Command</label>
            <select v-model="selectedCommand" class="input">
              <option value="ping">ping</option>
              <option value="increment">increment</option>
              <option value="get_counter">get_counter</option>
              <option value="echo">echo</option>
            </select>
          </div>
          <div class="form-group">
            <label>Parameters (JSON)</label>
            <textarea
              v-model="commandParams"
              class="input"
              rows="4"
              placeholder='{"key": "value"}'
            />
          </div>
          <button @click="executeCommand" class="btn btn-primary" :disabled="executing">
            <Loader2 v-if="executing" class="spinner-sm" :size="14" />
            Execute
          </button>
          <div v-if="commandResult" class="result">
            <label>Result</label>
            <pre>{{ JSON.stringify(commandResult, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </Dialog>

    <!-- Sandbox Config Dialog -->
    <Dialog v-model:open="showSandboxDialog" title="Sandbox Configuration">
      <div class="dialog-content">
        <div class="sandbox-config">
          <div class="form-group">
            <label>Sandbox Status</label>
            <div class="sandbox-status" :class="{ enabled: sandboxEnabled }">
              <Shield :size="20" />
              <span>{{ sandboxEnabled ? 'Enabled' : 'Disabled' }}</span>
            </div>
          </div>
          
          <div class="form-group">
            <label>
              <input type="checkbox" v-model="sandboxConfig.enableSeccomp" />
              Enable seccomp (Linux only)
            </label>
            <p class="hint">Restrict system calls using seccomp filters</p>
          </div>
          
          <div class="form-group">
            <label>
              <input type="checkbox" v-model="sandboxConfig.allowNetwork" />
              Allow Network Access
            </label>
            <p class="hint">Allow plugins to make network requests</p>
          </div>
          
          <div class="form-group">
            <label>
              <input type="checkbox" v-model="sandboxConfig.allowFilesystem" />
              Allow Filesystem Access
            </label>
            <p class="hint">Allow plugins to access the filesystem</p>
          </div>
          
          <div class="form-group">
            <label>Max Memory (MB)</label>
            <input
              v-model.number="sandboxConfig.maxMemoryMb"
              type="number"
              min="64"
              max="4096"
              class="input"
            />
            <p class="hint">Maximum memory per plugin (64-4096 MB)</p>
          </div>
          
          <div class="dialog-actions">
            <button @click="showSandboxDialog = false" class="btn btn-secondary">Cancel</button>
            <button @click="disableSandbox" class="btn btn-danger" v-if="sandboxEnabled">
              Disable Sandbox
            </button>
            <button @click="enableSandbox" class="btn btn-success" v-else>
              Enable Sandbox
            </button>
          </div>
        </div>
      </div>
    </Dialog>

    <!-- Resource Stats Dialog -->
    <Dialog v-model:open="showResourceStatsDialog" :title="`Resource Stats - ${selectedPlugin?.name}`">
      <div class="dialog-content">
        <div v-if="resourceStats" class="resource-stats">
          <div class="stat-item">
            <label>Command Count</label>
            <div class="stat-value">{{ resourceStats.command_count }}</div>
            <div class="stat-bar">
              <div class="stat-fill" :style="{ width: `${Math.min(resourceStats.command_count / resourceStats.max_concurrent_commands * 100, 100)}%` }"></div>
            </div>
            <p class="hint">{{ resourceStats.max_concurrent_commands }} max concurrent</p>
          </div>
          
          <div class="stat-item">
            <label>Network Requests</label>
            <div class="stat-value">{{ resourceStats.network_request_count }}</div>
            <div class="stat-bar">
              <div class="stat-fill" :style="{ width: `${Math.min(resourceStats.network_request_count / resourceStats.max_network_requests_per_minute * 100, 100)}%` }"></div>
            </div>
            <p class="hint">{{ resourceStats.max_network_requests_per_minute }} per minute</p>
          </div>
        </div>
        <div v-else class="loading">
          <Loader2 class="spinner" :size="24" />
          <span>Loading resource stats...</span>
        </div>
      </div>
    </Dialog>

    <!-- Audit Log Dialog -->
    <Dialog v-model:open="showAuditLogDialog" title="Audit Log">
      <div class="dialog-content">
        <div class="audit-log-controls">
          <button @click="loadAuditLogs" class="btn btn-sm btn-secondary" :disabled="loading">
            <Loader2 v-if="loading" class="spinner" :size="14" />
            <RefreshCw v-else :size="14" />
            Refresh
          </button>
          <button @click="clearOldAuditLogs" class="btn btn-sm btn-danger" :disabled="loading">
            <Loader2 v-if="loading" class="spinner" :size="14" />
            <Trash2 v-else :size="14" />
            Clear Old (24h)
          </button>
        </div>
        <div v-if="loading && auditLogs.length === 0" class="loading">
          <Loader2 class="spinner" :size="24" />
          <span>Loading audit logs...</span>
        </div>
        <div v-else-if="auditLogs.length === 0" class="empty">
          <FileText :size="48" />
          <p>No audit log entries</p>
        </div>
        <div v-else class="audit-log-list">
          <div v-for="log in auditLogs" :key="log.timestamp" class="log-entry" :class="log.status">
            <div class="log-header">
              <span class="log-time">{{ new Date(log.timestamp * 1000).toLocaleString() }}</span>
              <span class="log-plugin">{{ log.plugin_id }}</span>
              <span class="log-operation">{{ log.operation }}</span>
              <span class="log-status" :class="log.status">{{ log.status }}</span>
            </div>
            <div class="log-details">{{ log.details }}</div>
          </div>
        </div>
      </div>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  RefreshCw,
  Plus,
  Loader2,
  AlertCircle,
  Package,
  Terminal,
  Trash2,
  Shield,
  BarChart,
  FileText
} from 'lucide-vue-next';
import { nativePluginManager, type PluginMetadata, type ResourceStats, type AuditLogEntry } from '@/lib/nativePlugins';

const plugins = ref<PluginMetadata[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const showLoadDialog = ref(false);
const showCommandDialogOpen = ref(false);
const showSandboxDialog = ref(false);
const showResourceStatsDialog = ref(false);
const showAuditLogDialog = ref(false);
const selectedPlugin = ref<PluginMetadata | null>(null);
const pluginPath = ref('');
const selectedCommand = ref('ping');
const commandParams = ref('{}');
const commandResult = ref<any>(null);
const executing = ref(false);
const resourceStats = ref<ResourceStats | null>(null);
const auditLogs = ref<AuditLogEntry[]>([]);

// Sandbox configuration
const sandboxEnabled = ref(false);
const sandboxConfig = ref({
  enableSeccomp: true,
  allowNetwork: false,
  allowFilesystem: false,
  maxMemoryMb: 512,
});

const loadPlugins = async () => {
  loading.value = true;
  error.value = null;
  try {
    plugins.value = await nativePluginManager.list();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load plugins';
  } finally {
    loading.value = false;
  }
};

const scanPlugins = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.init();
    const count = await nativePluginManager.scan();
    await loadPlugins();
    if (count === 0) {
      error.value = 'No plugins found in plugins directory';
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to scan plugins';
  } finally {
    loading.value = false;
  }
};

const reloadPlugins = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.init();
    const reloaded = await nativePluginManager.reloadChanged();
    await loadPlugins();
    if (reloaded.length > 0) {
      console.log(`Reloaded ${reloaded.length} plugin(s): ${reloaded.join(', ')}`);
    } else {
      error.value = 'No plugins have been modified';
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to reload plugins';
  } finally {
    loading.value = false;
  }
};

const loadPlugin = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.init();
    const metadata = await nativePluginManager.load(pluginPath.value);
    await loadPlugins();
    showLoadDialog.value = false;
    pluginPath.value = '';
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load plugin';
  } finally {
    loading.value = false;
  }
};

const unloadPlugin = async (id: string) => {
  if (!confirm('Are you sure you want to unload this plugin?')) return;
  
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.unload(id);
    await loadPlugins();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to unload plugin';
  } finally {
    loading.value = false;
  }
};

const togglePlugin = async (id: string, enabled: boolean) => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.setEnabled(id, enabled);
    await loadPlugins();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to toggle plugin';
  } finally {
    loading.value = false;
  }
};

const showCommandDialog = (plugin: PluginMetadata) => {
  selectedPlugin.value = plugin;
  showCommandDialogOpen.value = true;
  commandResult.value = null;
};

const executeCommand = async () => {
  if (!selectedPlugin.value) return;
  
  executing.value = true;
  error.value = null;
  try {
    const params = JSON.parse(commandParams.value);
    commandResult.value = await nativePluginManager.execute(
      selectedPlugin.value.id,
      selectedCommand.value,
      params
    );
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to execute command';
    commandResult.value = { error: e instanceof Error ? e.message : 'Unknown error' };
  } finally {
    executing.value = false;
  }
};

const isSensitivePermission = (permission: string): boolean => {
  return ['passwords', 'cookies', 'history'].includes(permission);
};

const enableSandbox = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.enableSandbox(
      sandboxConfig.value.enableSeccomp,
      sandboxConfig.value.allowNetwork,
      sandboxConfig.value.allowFilesystem,
      sandboxConfig.value.maxMemoryMb
    );
    sandboxEnabled.value = true;
    showSandboxDialog.value = false;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to enable sandbox';
  } finally {
    loading.value = false;
  }
};

const disableSandbox = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.disableSandbox();
    sandboxEnabled.value = false;
    showSandboxDialog.value = false;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to disable sandbox';
  } finally {
    loading.value = false;
  }
};

const showResourceStats = async (plugin: PluginMetadata) => {
  selectedPlugin.value = plugin;
  resourceStats.value = null;
  showResourceStatsDialog.value = true;
  loading.value = true;
  error.value = null;
  try {
    resourceStats.value = await nativePluginManager.getResourceStats(plugin.id);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load resource stats';
  } finally {
    loading.value = false;
  }
};

const loadAuditLogs = async () => {
  loading.value = true;
  error.value = null;
  try {
    auditLogs.value = await nativePluginManager.getAuditLog();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load audit logs';
  } finally {
    loading.value = false;
  }
};

const clearOldAuditLogs = async () => {
  if (!confirm('Are you sure you want to clear audit logs older than 24 hours?')) return;
  
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.clearOldAuditLogs(86400); // 24 hours
    await loadAuditLogs();
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to clear audit logs';
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  await loadPlugins();
});
</script>

<style scoped>
.native-plugin-manager {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.actions {
  display: flex;
  gap: 10px;
}

.btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: #3b82f6;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-secondary {
  background: #6b7280;
  color: white;
}

.btn-secondary:hover:not(:disabled) {
  background: #4b5563;
}

.btn-success {
  background: #10b981;
  color: white;
}

.btn-success:hover:not(:disabled) {
  background: #059669;
}

.btn-warning {
  background: #f59e0b;
  color: white;
}

.btn-warning:hover:not(:disabled) {
  background: #d97706;
}

.btn-info {
  background: #8b5cf6;
  color: white;
}

.btn-info:hover:not(:disabled) {
  background: #7c3aed;
}

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

.loading,
.error,
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  color: #6b7280;
}

.spinner {
  animation: spin 1s linear infinite;
}

.spinner-sm {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.error {
  color: #ef4444;
}

.empty {
  gap: 16px;
}

.hint {
  font-size: 14px;
  color: #9ca3af;
}

.plugin-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.plugin-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  transition: all 0.2s;
}

.plugin-item:hover {
  border-color: #d1d5db;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.plugin-item.disabled {
  opacity: 0.6;
}

.plugin-info {
  flex: 1;
}

.plugin-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.plugin-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #111827;
}

.version {
  padding: 2px 8px;
  background: #f3f4f6;
  border-radius: 4px;
  font-size: 12px;
  color: #6b7280;
}

.description {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #4b5563;
}

.meta {
  display: flex;
  gap: 16px;
  margin-bottom: 12px;
  font-size: 12px;
  color: #9ca3af;
}

.permissions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  padding: 2px 8px;
  background: #dbeafe;
  color: #1e40af;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.permission-tag.sensitive {
  background: #fee2e2;
  color: #991b1b;
}

.plugin-actions {
  display: flex;
  gap: 8px;
  margin-left: 16px;
}

.dialog-content {
  padding: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
}

.input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}

.command-interface {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.result {
  margin-top: 16px;
}

.result pre {
  padding: 12px;
  background: #1f2937;
  color: #e5e7eb;
  border-radius: 6px;
  font-size: 12px;
  overflow-x: auto;
}

.sandbox-config {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.sandbox-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #f3f4f6;
  border-radius: 6px;
  font-weight: 500;
  color: #6b7280;
}

.sandbox-status.enabled {
  background: #d1fae5;
  color: #065f46;
}

.sandbox-config .form-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.sandbox-config .form-group input[type="checkbox"] {
  width: auto;
}

.sandbox-config .hint {
  margin-top: 4px;
  font-size: 12px;
  color: #9ca3af;
}

.resource-stats {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.stat-item label {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.stat-value {
  font-size: 32px;
  font-weight: 600;
  color: #111827;
}

.stat-bar {
  height: 8px;
  background: #e5e7eb;
  border-radius: 4px;
  overflow: hidden;
}

.stat-fill {
  height: 100%;
  background: linear-gradient(90deg, #3b82f6, #8b5cf6);
  transition: width 0.3s ease;
}

.audit-log-controls {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.audit-log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.log-entry {
  padding: 12px;
  background: #f9fafb;
  border-radius: 6px;
  border-left: 4px solid #6b7280;
}

.log-entry.success {
  border-left-color: #10b981;
}

.log-entry.error {
  border-left-color: #ef4444;
}

.log-header {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 8px;
  font-size: 12px;
}

.log-time {
  color: #6b7280;
}

.log-plugin {
  font-weight: 500;
  color: #374151;
}

.log-operation {
  color: #8b5cf6;
}

.log-status {
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 500;
}

.log-status.success {
  background: #d1fae5;
  color: #065f46;
}

.log-status.error {
  background: #fee2e2;
  color: #991b1b;
}

.log-details {
  font-size: 13px;
  color: #4b5563;
  line-height: 1.5;
}
</style>
