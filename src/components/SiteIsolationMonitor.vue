<template>
  <div class="site-isolation-monitor">
    <div class="monitor-header">
      <h2>Site Isolation Monitor</h2>
      <p class="subtitle">Monitor site isolation processes and security</p>
    </div>

    <div class="monitor-content">
      <!-- Overview Stats -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon active">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 6v6l4 2"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Active Processes</span>
            <span class="stat-value">{{ activeProcesses }}</span>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon sites">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
              <polyline points="9 22 9 12 15 12 15 22"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Active Sites</span>
            <span class="stat-value">{{ activeSites }}</span>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon crashed">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
              <line x1="12" y1="9" x2="12" y2="13"/>
              <line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Crashed Processes</span>
            <span class="stat-value">{{ crashedProcesses }}</span>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon memory">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M2 12h20"/>
              <path d="M2 12l5-5"/>
              <path d="M2 12l5 5"/>
              <path d="M22 12l-5-5"/>
              <path d="M22 12l-5 5"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Memory Usage</span>
            <span class="stat-value">{{ memoryUsage }} MB</span>
          </div>
        </div>
      </div>

      <!-- Isolation Policy -->
      <div class="card">
        <div class="card-header">
          <h3>Isolation Policy</h3>
          <button @click="editPolicy" class="btn btn-sm btn-secondary">
            Edit Policy
          </button>
        </div>
        <div class="policy-details">
          <div class="policy-item">
            <span class="policy-label">Enabled:</span>
            <span :class="['policy-value', policy.enabled ? 'enabled' : 'disabled']">
              {{ policy.enabled ? 'Yes' : 'No' }}
            </span>
          </div>
          <div class="policy-item">
            <span class="policy-label">Strict Mode:</span>
            <span :class="['policy-value', policy.strictMode ? 'enabled' : 'disabled']">
              {{ policy.strictMode ? 'Yes' : 'No' }}
            </span>
          </div>
          <div class="policy-item">
            <span class="policy-label">Max Processes:</span>
            <span class="policy-value">{{ policy.maxProcesses }}</span>
          </div>
          <div class="policy-item">
            <span class="policy-label">Process Memory Limit:</span>
            <span class="policy-value">{{ policy.processMemoryLimit }} MB</span>
          </div>
          <div class="policy-item">
            <span class="policy-label">Spectre Mitigations:</span>
            <span :class="['policy-value', policy.enableSpectreMitigations ? 'enabled' : 'disabled']">
              {{ policy.enableSpectreMitigations ? 'Enabled' : 'Disabled' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Active Processes -->
      <div class="card">
        <div class="card-header">
          <h3>Active Processes</h3>
          <button @click="refreshProcesses" class="btn btn-sm btn-secondary">
            Refresh
          </button>
        </div>
        <div class="process-list">
          <div
            v-for="process in processes"
            :key="process.processId"
            :class="['process-item', process.isCrashed ? 'crashed' : 'active']"
          >
            <div class="process-info">
              <span class="process-id">{{ process.processId }}</span>
              <span class="process-site">{{ process.siteId.etldPlusOne }}</span>
              <span class="process-pid">PID: {{ process.pid || 'N/A' }}</span>
            </div>
            <div class="process-metrics">
              <span class="process-memory">{{ process.memoryUsage }} MB</span>
              <span class="process-cpu">{{ process.cpuUsage }}%</span>
            </div>
            <div class="process-actions">
              <button
                v-if="process.isCrashed"
                @click="recoverProcess(process.processId)"
                class="btn btn-sm btn-recover"
              >
                Recover
              </button>
              <button
                @click="terminateProcess(process.processId)"
                class="btn btn-sm btn-terminate"
              >
                Terminate
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Site Instances -->
      <div class="card">
        <div class="card-header">
          <h3>Site Instances</h3>
          <button @click="refreshSites" class="btn btn-sm btn-secondary">
            Refresh
          </button>
        </div>
        <div class="site-list">
          <div
            v-for="site in sites"
            :key="site.siteId.etldPlusOne"
            class="site-item"
          >
            <div class="site-info">
              <span class="site-scheme">{{ site.siteId.scheme }}://</span>
              <span class="site-domain">{{ site.siteId.etldPlusOne }}</span>
            </div>
            <div class="site-details">
              <span class="site-process">{{ site.processId || 'No process' }}</span>
              <span class="site-webviews">{{ site.webviewCount }} webview(s)</span>
            </div>
            <div class="site-actions">
              <button @click="viewSiteDetails(site)" class="btn btn-sm btn-view">
                Details
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Blacklisted Sites -->
      <div class="card">
        <div class="card-header">
          <h3>Blacklisted Sites</h3>
          <button @click="refreshBlacklist" class="btn btn-sm btn-secondary">
            Refresh
          </button>
        </div>
        <div v-if="blacklistedSites.length > 0" class="blacklist-list">
          <div
            v-for="site in blacklistedSites"
            :key="site"
            class="blacklist-item"
          >
            <span class="blacklist-url">{{ site }}</span>
            <button @click="unblockSite(site)" class="btn btn-sm btn-unblock">
              Unblock
            </button>
          </div>
        </div>
        <div v-else class="empty-state">
          <p>No blacklisted sites</p>
        </div>
      </div>

      <!-- Event Log -->
      <div class="card">
        <div class="card-header">
          <h3>Event Log</h3>
          <button @click="clearLog" class="btn btn-sm btn-secondary">
            Clear
          </button>
        </div>
        <div class="event-log">
          <div
            v-for="(event, index) in eventLog"
            :key="index"
            :class="['event-item', event.type]"
          >
            <span class="event-time">{{ formatTime(event.timestamp) }}</span>
            <span class="event-message">{{ event.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

// State
const activeProcesses = ref(0);
const activeSites = ref(0);
const crashedProcesses = ref(0);
const memoryUsage = ref(0);

const policy = ref({
  enabled: true,
  strictMode: true,
  maxProcesses: 50,
  processMemoryLimit: 512,
  processCpuLimit: 80,
  idleProcessTimeout: 300,
  enableProcessPooling: true,
  minPoolSize: 5,
  maxPoolSize: 20,
  enableSpectreMitigations: true,
  strictSameOrigin: true,
});

const processes = ref<any[]>([]);
const sites = ref<any[]>([]);
const blacklistedSites = ref<string[]>([]);
const eventLog = ref<any[]>([]);

let refreshInterval: number | null = null;

// Methods
const loadIsolationPolicy = async () => {
  try {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_isolation_policy', {});
    policy.value = result;
  } catch (error) {
    console.error('Failed to load isolation policy:', error);
  }
};

const loadProcesses = async () => {
  try {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_processes', {});
    processes.value = result;
    
    activeProcesses.value = result.filter((p: any) => !p.isCrashed).length;
    crashedProcesses.value = result.filter((p: any) => p.isCrashed).length;
    memoryUsage.value = result.reduce((sum: number, p: any) => sum + p.memoryUsage, 0);
  } catch (error) {
    console.error('Failed to load processes:', error);
  }
};

const loadSites = async () => {
  try {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_site_instances', {});
    sites.value = result;
    activeSites.value = result.length;
  } catch (error) {
    console.error('Failed to load sites:', error);
  }
};

const loadBlacklist = async () => {
  try {
    const result = await window.__EXODUS_TAURI_INVOKE__('get_blacklisted_sites', {});
    blacklistedSites.value = result;
  } catch (error) {
    console.error('Failed to load blacklist:', error);
  }
};

const refreshProcesses = () => {
  loadProcesses();
  addEvent('info', 'Processes refreshed');
};

const refreshSites = () => {
  loadSites();
  addEvent('info', 'Sites refreshed');
};

const refreshBlacklist = () => {
  loadBlacklist();
  addEvent('info', 'Blacklist refreshed');
};

const recoverProcess = async (processId: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('recover_process', { processId });
    addEvent('success', `Process ${processId} recovered`);
    loadProcesses();
  } catch (error) {
    console.error('Failed to recover process:', error);
    addEvent('error', `Failed to recover process ${processId}`);
  }
};

const terminateProcess = async (processId: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('release_site', { url: processId });
    addEvent('warning', `Process ${processId} terminated`);
    loadProcesses();
  } catch (error) {
    console.error('Failed to terminate process:', error);
    addEvent('error', `Failed to terminate process ${processId}`);
  }
};

const unblockSite = async (site: string) => {
  try {
    await window.__EXODUS_TAURI_INVOKE__('unblock_site', { url: site });
    addEvent('success', `Site ${site} unblocked`);
    loadBlacklist();
  } catch (error) {
    console.error('Failed to unblock site:', error);
    addEvent('error', `Failed to unblock site ${site}`);
  }
};

const viewSiteDetails = (site: any) => {
  console.log('View site details:', site);
  // Open details modal
};

const editPolicy = () => {
  console.log('Edit policy');
  // Open policy editor modal
};

const addEvent = (type: string, message: string) => {
  eventLog.value.unshift({
    timestamp: Date.now(),
    type,
    message,
  });
  
  // Keep only last 100 events
  if (eventLog.value.length > 100) {
    eventLog.value = eventLog.value.slice(0, 100);
  }
};

const clearLog = () => {
  eventLog.value = [];
};

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
};

onMounted(() => {
  loadIsolationPolicy();
  loadProcesses();
  loadSites();
  loadBlacklist();
  
  // Auto-refresh every 5 seconds
  refreshInterval = window.setInterval(() => {
    loadProcesses();
    loadSites();
  }, 5000);
  
  addEvent('info', 'Site isolation monitor started');
});

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
  }
});
</script>

<style scoped>
.site-isolation-monitor {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.monitor-header {
  margin-bottom: 30px;
}

.monitor-header h2 {
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 8px;
}

.subtitle {
  color: #666;
  font-size: 14px;
}

.monitor-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 20px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-icon.active {
  background: #d4edda;
  color: #28a745;
}

.stat-icon.sites {
  background: #cce5ff;
  color: #007bff;
}

.stat-icon.crashed {
  background: #f8d7da;
  color: #dc3545;
}

.stat-icon.memory {
  background: #fff3cd;
  color: #ffc107;
}

.stat-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 13px;
  color: #666;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #333;
}

.card {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.card h3 {
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.btn {
  padding: 6px 12px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  transition: background 0.2s;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 12px;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #545b62;
}

.btn-recover {
  background: #28a745;
  color: white;
}

.btn-recover:hover {
  background: #218838;
}

.btn-terminate {
  background: #dc3545;
  color: white;
}

.btn-terminate:hover {
  background: #c82333;
}

.btn-view {
  background: #007bff;
  color: white;
}

.btn-unblock {
  background: #28a745;
  color: white;
}

.policy-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.policy-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: #f8f9fa;
  border-radius: 4px;
}

.policy-label {
  font-size: 14px;
  color: #666;
}

.policy-value {
  font-weight: 600;
  color: #333;
}

.policy-value.enabled {
  color: #28a745;
}

.policy-value.disabled {
  color: #dc3545;
}

.process-list,
.site-list,
.blacklist-list,
.event-log {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 400px;
  overflow-y: auto;
}

.process-item,
.site-item,
.blacklist-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 4px;
}

.process-item.crashed {
  background: #f8d7da;
  border: 1px solid #dc3545;
}

.process-item.active {
  background: #d4edda;
  border: 1px solid #28a745;
}

.process-info,
.site-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.process-id,
.site-scheme {
  font-weight: 600;
  color: #333;
}

.process-site,
.site-domain {
  font-size: 13px;
  color: #666;
}

.process-pid,
.site-process {
  font-size: 12px;
  color: #999;
}

.process-metrics,
.site-details {
  display: flex;
  gap: 15px;
  font-size: 13px;
  color: #666;
}

.process-actions,
.site-actions {
  display: flex;
  gap: 8px;
}

.blacklist-url {
  font-size: 14px;
  color: #333;
}

.empty-state {
  padding: 20px;
  text-align: center;
  color: #999;
}

.event-item {
  display: flex;
  gap: 12px;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 13px;
}

.event-item.info {
  background: #cce5ff;
}

.event-item.success {
  background: #d4edda;
}

.event-item.warning {
  background: #fff3cd;
}

.event-item.error {
  background: #f8d7da;
}

.event-time {
  color: #666;
  min-width: 80px;
}

.event-message {
  color: #333;
}
</style>
