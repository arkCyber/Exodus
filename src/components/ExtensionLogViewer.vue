<template>
  <div class="extension-log-viewer">
    <div class="header">
      <h2>Extension Log Viewer</h2>
      <div class="controls">
        <select v-model="selectedExtension" class="extension-select">
          <option value="">All Extensions</option>
          <option v-for="ext in extensions" :key="ext.id" :value="ext.id">
            {{ ext.name }}
          </option>
        </select>
        <select v-model="selectedLevel" class="level-select">
          <option value="">All Levels</option>
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warning</option>
          <option value="error">Error</option>
        </select>
        <input 
          v-model="searchQuery" 
          placeholder="Search logs..." 
          class="search-input"
        />
        <button @click="refresh" class="btn-secondary">Refresh</button>
        <button @click="clearLogs" class="btn-danger">Clear</button>
        <button @click="exportLogs" class="btn-primary">Export</button>
      </div>
    </div>

    <div class="log-stats">
      <div class="stat-item">
        <span class="stat-label">Total Logs:</span>
        <span class="stat-value">{{ filteredLogs.length }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Errors:</span>
        <span class="stat-value error">{{ errorCount }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Warnings:</span>
        <span class="stat-value warning">{{ warningCount }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">Auto-refresh:</span>
        <label class="toggle-switch">
          <input type="checkbox" v-model="autoRefresh" />
          <span class="slider"></span>
        </label>
      </div>
    </div>

    <div class="log-container">
      <div 
        v-for="(log, index) in paginatedLogs" 
        :key="index"
        :class="['log-entry', log.level]"
        @click="selectLog(log)"
      >
        <div class="log-header">
          <span class="log-time">{{ formatTime(log.timestamp) }}</span>
          <span :class="['log-level', log.level]">{{ log.level.toUpperCase() }}</span>
          <span class="log-extension">{{ log.extension }}</span>
          <span class="log-source">{{ log.source }}</span>
        </div>
        <div class="log-message">{{ log.message }}</div>
        <div v-if="log.details" class="log-details">
          <pre>{{ formatJson(log.details) }}</pre>
        </div>
      </div>
    </div>

    <div class="pagination">
      <button 
        @click="prevPage" 
        :disabled="currentPage === 1"
        class="btn-secondary"
      >
        Previous
      </button>
      <span class="page-info">Page {{ currentPage }} of {{ totalPages }}</span>
      <button 
        @click="nextPage" 
        :disabled="currentPage === totalPages"
        class="btn-secondary"
      >
        Next
      </button>
    </div>

    <div v-if="selectedLog" class="log-detail-modal" @click.self="closeDetail">
      <div class="modal-content">
        <div class="modal-header">
          <h3>Log Details</h3>
          <button @click="closeDetail" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="detail-row">
            <span class="detail-label">Timestamp:</span>
            <span class="detail-value">{{ formatTime(selectedLog.timestamp) }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Level:</span>
            <span :class="['detail-value', 'log-level', selectedLog.level]">
              {{ selectedLog.level.toUpperCase() }}
            </span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Extension:</span>
            <span class="detail-value">{{ selectedLog.extension }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Source:</span>
            <span class="detail-value">{{ selectedLog.source }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Message:</span>
            <span class="detail-value">{{ selectedLog.message }}</span>
          </div>
          <div v-if="selectedLog.details" class="detail-row full-width">
            <span class="detail-label">Details:</span>
            <pre class="detail-value">{{ formatJson(selectedLog.details) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

interface Extension {
  id: string;
  name: string;
  version: string;
}

interface LogEntry {
  timestamp: number;
  level: string;
  extension: string;
  source: string;
  message: string;
  details?: any;
}

const selectedExtension = ref('');
const selectedLevel = ref('');
const searchQuery = ref('');
const autoRefresh = ref(false);
const currentPage = ref(1);
const pageSize = 50;
const selectedLog = ref<LogEntry | null>(null);

const extensions = ref<Extension[]>([
  { id: 'ext-1', name: 'Extension API Demo', version: '1.0.0' },
  { id: 'ext-2', name: 'Ad Blocker', version: '2.1.0' },
  { id: 'ext-3', name: 'Password Manager', version: '1.5.0' },
]);

const logs = ref<LogEntry[]>([]);

let refreshInterval: number | null = null;

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    if (selectedExtension.value && log.extension !== selectedExtension.value) {
      return false;
    }
    if (selectedLevel.value && log.level !== selectedLevel.value) {
      return false;
    }
    if (searchQuery.value && !log.message.toLowerCase().includes(searchQuery.value.toLowerCase())) {
      return false;
    }
    return true;
  });
});

const errorCount = computed(() => {
  return filteredLogs.value.filter(log => log.level === 'error').length;
});

const warningCount = computed(() => {
  return filteredLogs.value.filter(log => log.level === 'warn').length;
});

const totalPages = computed(() => {
  return Math.ceil(filteredLogs.value.length / pageSize);
});

const paginatedLogs = computed(() => {
  const start = (currentPage.value - 1) * pageSize;
  const end = start + pageSize;
  return filteredLogs.value.slice(start, end);
});

const refresh = async () => {
  // Simulate fetching logs
  const sampleLogs: LogEntry[] = [
    {
      timestamp: Date.now() - 1000,
      level: 'info',
      extension: 'Extension API Demo',
      source: 'background.js',
      message: 'Extension loaded successfully',
    },
    {
      timestamp: Date.now() - 2000,
      level: 'debug',
      extension: 'Extension API Demo',
      source: 'content.js',
      message: 'Content script injected',
      details: { tabId: 123, url: 'https://example.com' },
    },
    {
      timestamp: Date.now() - 3000,
      level: 'warn',
      extension: 'Ad Blocker',
      source: 'background.js',
      message: 'Filter list update failed, using cached version',
      details: { error: 'Network timeout' },
    },
    {
      timestamp: Date.now() - 4000,
      level: 'error',
      extension: 'Password Manager',
      source: 'popup.js',
      message: 'Failed to decrypt stored passwords',
      details: { error: 'Invalid master password' },
    },
  ];
  
  logs.value = [...sampleLogs, ...logs.value].slice(0, 1000);
};

const clearLogs = () => {
  logs.value = [];
  currentPage.value = 1;
};

const exportLogs = () => {
  const content = JSON.stringify(filteredLogs.value, null, 2);
  const blob = new Blob([content], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `extension-logs-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
};

const selectLog = (log: LogEntry) => {
  selectedLog.value = log;
};

const closeDetail = () => {
  selectedLog.value = null;
};

const prevPage = () => {
  if (currentPage.value > 1) {
    currentPage.value--;
  }
};

const nextPage = () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
  }
};

const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleString();
};

const formatJson = (data: any) => {
  return JSON.stringify(data, null, 2);
};

watch(autoRefresh, (value) => {
  if (value) {
    refreshInterval = window.setInterval(refresh, 5000);
  } else {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }
});

onMounted(() => {
  refresh();
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
});
</script>

<style scoped>
.extension-log-viewer {
  padding: 20px;
  background: #1e1e1e;
  color: #d4d4d4;
  min-height: 100vh;
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
  color: #fff;
}

.controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.extension-select,
.level-select,
.search-input {
  padding: 8px 12px;
  background: #2d2d2d;
  border: 1px solid #3e3e3e;
  color: #d4d4d4;
  border-radius: 4px;
  cursor: pointer;
}

.search-input {
  width: 200px;
  cursor: text;
}

.btn-secondary,
.btn-primary,
.btn-danger {
  padding: 8px 16px;
  border: none;
  color: white;
  border-radius: 4px;
  cursor: pointer;
}

.btn-secondary {
  background: #007acc;
}

.btn-secondary:hover {
  background: #005a9e;
}

.btn-primary {
  background: #4caf50;
}

.btn-primary:hover {
  background: #45a049;
}

.btn-danger {
  background: #f44336;
}

.btn-danger:hover {
  background: #da190b;
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.log-stats {
  display: flex;
  gap: 20px;
  padding: 15px;
  background: #2d2d2d;
  border-radius: 8px;
  margin-bottom: 20px;
  border: 1px solid #3e3e3e;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stat-label {
  color: #888;
  font-size: 12px;
}

.stat-value {
  font-weight: bold;
  color: #fff;
}

.stat-value.error {
  color: #f44336;
}

.stat-value.warning {
  color: #ff9800;
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 20px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #3e3e3e;
  transition: 0.3s;
  border-radius: 20px;
}

.slider:before {
  position: absolute;
  content: '';
  height: 16px;
  width: 16px;
  left: 2px;
  bottom: 2px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #007acc;
}

input:checked + .slider:before {
  transform: translateX(20px);
}

.log-container {
  background: #2d2d2d;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
  border: 1px solid #3e3e3e;
  max-height: 500px;
  overflow-y: auto;
}

.log-entry {
  padding: 10px;
  margin-bottom: 8px;
  border-radius: 4px;
  border-left: 3px solid #3e3e3e;
  cursor: pointer;
  transition: background 0.2s;
}

.log-entry:hover {
  background: #1e1e1e;
}

.log-entry.debug {
  border-left-color: #888;
}

.log-entry.info {
  border-left-color: #2196f3;
}

.log-entry.warn {
  border-left-color: #ff9800;
}

.log-entry.error {
  border-left-color: #f44336;
}

.log-header {
  display: flex;
  gap: 15px;
  margin-bottom: 5px;
  font-size: 12px;
}

.log-time {
  color: #888;
}

.log-level {
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  text-transform: uppercase;
  font-weight: bold;
}

.log-level.debug {
  background: #888;
  color: white;
}

.log-level.info {
  background: #2196f3;
  color: white;
}

.log-level.warn {
  background: #ff9800;
  color: white;
}

.log-level.error {
  background: #f44336;
  color: white;
}

.log-extension {
  color: #007acc;
}

.log-source {
  color: #888;
}

.log-message {
  color: #d4d4d4;
  margin-bottom: 5px;
}

.log-details {
  background: #1e1e1e;
  padding: 10px;
  border-radius: 4px;
  margin-top: 5px;
}

.log-details pre {
  margin: 0;
  font-family: monospace;
  font-size: 11px;
  color: #888;
  white-space: pre-wrap;
}

.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 15px;
}

.page-info {
  color: #888;
}

.log-detail-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: #2d2d2d;
  border-radius: 8px;
  width: 600px;
  max-width: 90%;
  max-height: 80vh;
  overflow-y: auto;
  border: 1px solid #3e3e3e;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #3e3e3e;
}

.modal-header h3 {
  margin: 0;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 24px;
  cursor: pointer;
}

.close-btn:hover {
  color: #fff;
}

.modal-body {
  padding: 20px;
}

.detail-row {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.detail-row.full-width {
  flex-direction: column;
}

.detail-label {
  color: #888;
  font-size: 12px;
  min-width: 100px;
}

.detail-value {
  color: #d4d4d4;
  flex: 1;
}

.detail-value pre {
  margin: 0;
  font-family: monospace;
  font-size: 12px;
  color: #888;
  white-space: pre-wrap;
  background: #1e1e1e;
  padding: 10px;
  border-radius: 4px;
}
</style>
