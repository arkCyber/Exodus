<template>
  <div class="extension-monitor-panel">
    <div class="header">
      <h2>Extension Monitor</h2>
      <div class="controls">
        <select v-model="selectedExtension" class="extension-select">
          <option value="">All Extensions</option>
          <option v-for="ext in extensions" :key="ext.id" :value="ext.id">
            {{ ext.name }}
          </option>
        </select>
        <button @click="refresh" class="btn-secondary">Refresh</button>
      </div>
    </div>

    <div class="metrics-grid">
      <div class="metric-card">
        <div class="metric-label">Active Extensions</div>
        <div class="metric-value">{{ activeExtensions }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">Total API Calls</div>
        <div class="metric-value">{{ totalApiCalls }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">Memory Usage</div>
        <div class="metric-value">{{ memoryUsage }} MB</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">CPU Usage</div>
        <div class="metric-value">{{ cpuUsage }}%</div>
      </div>
    </div>

    <div class="tabs">
      <button 
        v-for="tab in tabs" 
        :key="tab.id"
        @click="activeTab = tab.id"
        :class="{ active: activeTab === tab.id }"
        class="tab"
      >
        {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <!-- Performance Tab -->
      <div v-if="activeTab === 'performance'" class="performance-tab">
        <div class="chart-container">
          <h3>API Calls Over Time</h3>
          <div class="chart-placeholder">
            <div class="chart-bar" v-for="(value, index) in apiCallsData" :key="index" :style="{ height: value + '%' }"></div>
          </div>
        </div>
        <div class="chart-container">
          <h3>Memory Usage Over Time</h3>
          <div class="chart-placeholder">
            <div class="chart-bar memory" v-for="(value, index) in memoryData" :key="index" :style="{ height: value + '%' }"></div>
          </div>
        </div>
      </div>

      <!-- Errors Tab -->
      <div v-if="activeTab === 'errors'" class="errors-tab">
        <div class="error-list">
          <div v-for="error in errors" :key="error.id" class="error-item">
            <div class="error-header">
              <span class="error-time">{{ formatTime(error.timestamp) }}</span>
              <span class="error-extension">{{ error.extension }}</span>
              <span :class="['error-severity', error.severity]">{{ error.severity }}</span>
            </div>
            <div class="error-message">{{ error.message }}</div>
            <div class="error-stack">{{ error.stack }}</div>
          </div>
        </div>
      </div>

      <!-- Events Tab -->
      <div v-if="activeTab === 'events'" class="events-tab">
        <div class="event-list">
          <div v-for="event in events" :key="event.id" class="event-item">
            <div class="event-header">
              <span class="event-time">{{ formatTime(event.timestamp) }}</span>
              <span class="event-type">{{ event.type }}</span>
              <span class="event-extension">{{ event.extension }}</span>
            </div>
            <div class="event-data">{{ formatJson(event.data) }}</div>
          </div>
        </div>
      </div>

      <!-- Network Tab -->
      <div v-if="activeTab === 'network'" class="network-tab">
        <div class="network-list">
          <div v-for="request in networkRequests" :key="request.id" class="network-item">
            <div class="network-header">
              <span class="network-method">{{ request.method }}</span>
              <span class="network-url">{{ request.url }}</span>
              <span :class="['network-status', getStatusClass(request.status)]">{{ request.status }}</span>
            </div>
            <div class="network-details">
              <span class="network-time">{{ request.duration }}ms</span>
              <span class="network-size">{{ formatBytes(request.size) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Extension {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
}

interface Error {
  id: string;
  timestamp: number;
  extension: string;
  severity: string;
  message: string;
  stack: string;
}

interface Event {
  id: string;
  timestamp: number;
  type: string;
  extension: string;
  data: any;
}

interface NetworkRequest {
  id: string;
  method: string;
  url: string;
  status: number;
  duration: number;
  size: number;
}

const selectedExtension = ref('');
const activeTab = ref('performance');
const extensions = ref<Extension[]>([]);
const activeExtensions = ref(0);
const totalApiCalls = ref(0);
const memoryUsage = ref(0);
const cpuUsage = ref(0);
const apiCallsData = ref<number[]>([]);
const memoryData = ref<number[]>([]);
const errors = ref<Error[]>([]);
const events = ref<Event[]>([]);
const networkRequests = ref<NetworkRequest[]>([]);

const tabs = [
  { id: 'performance', name: 'Performance' },
  { id: 'errors', name: 'Errors' },
  { id: 'events', name: 'Events' },
  { id: 'network', name: 'Network' },
];

let refreshInterval: number | null = null;

const refresh = async () => {
  // Simulate fetching data
  extensions.value = [
    { id: 'ext-1', name: 'Extension API Demo', version: '1.0.0', enabled: true },
    { id: 'ext-2', name: 'Ad Blocker', version: '2.1.0', enabled: true },
    { id: 'ext-3', name: 'Password Manager', version: '1.5.0', enabled: false },
  ];
  
  activeExtensions.value = extensions.value.filter(e => e.enabled).length;
  totalApiCalls.value = Math.floor(Math.random() * 10000) + 1000;
  memoryUsage.value = Math.floor(Math.random() * 500) + 100;
  cpuUsage.value = Math.floor(Math.random() * 50) + 10;
  
  apiCallsData.value = Array.from({ length: 20 }, () => Math.floor(Math.random() * 100));
  memoryData.value = Array.from({ length: 20 }, () => Math.floor(Math.random() * 100));
  
  errors.value = [
    {
      id: 'err-1',
      timestamp: Date.now() - 3600000,
      extension: 'Extension API Demo',
      severity: 'error',
      message: 'Failed to load storage',
      stack: 'at background.js:45:12',
    },
  ];
  
  events.value = [
    {
      id: 'evt-1',
      timestamp: Date.now() - 1800000,
      type: 'storage.onChanged',
      extension: 'Extension API Demo',
      data: { key: 'test', oldValue: null, newValue: 'value' },
    },
  ];
  
  networkRequests.value = [
    {
      id: 'req-1',
      method: 'GET',
      url: 'https://api.example.com/data',
      status: 200,
      duration: 125,
      size: 1024,
    },
  ];
};

const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString();
};

const formatJson = (data: any) => {
  return JSON.stringify(data, null, 2);
};

const formatBytes = (bytes: number) => {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
};

const getStatusClass = (status: number) => {
  if (status >= 200 && status < 300) return 'success';
  if (status >= 400 && status < 500) return 'client-error';
  if (status >= 500) return 'server-error';
  return '';
};

onMounted(() => {
  refresh();
  refreshInterval = window.setInterval(refresh, 5000);
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
});
</script>

<style scoped>
.extension-monitor-panel {
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
}

.extension-select {
  padding: 8px 12px;
  background: #2d2d2d;
  border: 1px solid #3e3e3e;
  color: #d4d4d4;
  border-radius: 4px;
  cursor: pointer;
}

.btn-secondary {
  padding: 8px 16px;
  background: #007acc;
  border: none;
  color: white;
  border-radius: 4px;
  cursor: pointer;
}

.btn-secondary:hover {
  background: #005a9e;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 15px;
  margin-bottom: 20px;
}

.metric-card {
  background: #2d2d2d;
  padding: 15px;
  border-radius: 8px;
  border: 1px solid #3e3e3e;
}

.metric-label {
  font-size: 12px;
  color: #888;
  margin-bottom: 5px;
}

.metric-value {
  font-size: 24px;
  font-weight: bold;
  color: #fff;
}

.tabs {
  display: flex;
  gap: 5px;
  margin-bottom: 20px;
  border-bottom: 1px solid #3e3e3e;
}

.tab {
  padding: 10px 20px;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
  border-bottom: 2px solid transparent;
}

.tab:hover {
  color: #d4d4d4;
}

.tab.active {
  color: #007acc;
  border-bottom-color: #007acc;
}

.tab-content {
  background: #2d2d2d;
  border-radius: 8px;
  padding: 20px;
  border: 1px solid #3e3e3e;
}

.chart-container {
  margin-bottom: 30px;
}

.chart-container h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #fff;
}

.chart-placeholder {
  display: flex;
  align-items: flex-end;
  gap: 5px;
  height: 150px;
  background: #1e1e1e;
  padding: 10px;
  border-radius: 4px;
}

.chart-bar {
  flex: 1;
  background: #007acc;
  border-radius: 2px 2px 0 0;
  transition: height 0.3s ease;
}

.chart-bar.memory {
  background: #4caf50;
}

.error-list,
.event-list,
.network-list {
  max-height: 400px;
  overflow-y: auto;
}

.error-item,
.event-item,
.network-item {
  background: #1e1e1e;
  padding: 15px;
  margin-bottom: 10px;
  border-radius: 4px;
  border-left: 3px solid #007acc;
}

.error-header,
.event-header,
.network-header {
  display: flex;
  gap: 15px;
  margin-bottom: 10px;
  font-size: 12px;
}

.error-time,
.event-time {
  color: #888;
}

.error-extension,
.event-extension {
  color: #007acc;
}

.error-severity {
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  text-transform: uppercase;
}

.error-severity.error {
  background: #f44336;
  color: white;
}

.error-severity.warning {
  background: #ff9800;
  color: white;
}

.error-message {
  color: #d4d4d4;
  margin-bottom: 5px;
}

.error-stack {
  font-family: monospace;
  font-size: 11px;
  color: #888;
  white-space: pre-wrap;
}

.event-type {
  color: #4caf50;
}

.event-data {
  font-family: monospace;
  font-size: 12px;
  color: #888;
  white-space: pre-wrap;
}

.network-method {
  padding: 2px 8px;
  background: #007acc;
  color: white;
  border-radius: 3px;
  font-size: 11px;
}

.network-url {
  color: #d4d4d4;
  flex: 1;
}

.network-status {
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
}

.network-status.success {
  background: #4caf50;
  color: white;
}

.network-status.client-error {
  background: #ff9800;
  color: white;
}

.network-status.server-error {
  background: #f44336;
  color: white;
}

.network-details {
  display: flex;
  gap: 15px;
  font-size: 12px;
  color: #888;
}
</style>
