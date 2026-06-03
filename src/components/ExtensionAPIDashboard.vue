<template>
  <div class="extension-api-dashboard">
    <div class="dashboard-header">
      <h2>Extension API Dashboard</h2>
      <p class="subtitle">Monitor and manage Extension API usage</p>
    </div>

    <div class="dashboard-grid">
      <!-- API Overview -->
      <div class="card">
        <h3>API Overview</h3>
        <div class="api-stats">
          <div class="stat-item">
            <span class="stat-label">Total APIs</span>
            <span class="stat-value">{{ totalApis }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Active Extensions</span>
            <span class="stat-value">{{ activeExtensions }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">API Calls/min</span>
            <span class="stat-value">{{ apiCallsPerMinute }}</span>
          </div>
        </div>
      </div>

      <!-- Context Menus -->
      <div class="card">
        <h3>Context Menus</h3>
        <div class="api-section">
          <button @click="createTestMenuItem" class="btn btn-primary">
            Create Test Item
          </button>
          <button @click="loadMenuItems" class="btn btn-secondary">
            Load Items
          </button>
          <div v-if="menuItems.length > 0" class="items-list">
            <div v-for="item in menuItems" :key="item.id" class="item-row">
              <span>{{ item.title }}</span>
              <button @click="removeMenuItem(item.id)" class="btn btn-sm btn-danger">
                Remove
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Top Sites -->
      <div class="card">
        <h3>Top Sites</h3>
        <div class="api-section">
          <button @click="loadTopSites" class="btn btn-secondary">
            Refresh
          </button>
          <div v-if="topSites.length > 0" class="sites-list">
            <div v-for="(site, index) in topSites" :key="index" class="site-row">
              <span class="site-rank">#{{ index + 1 }}</span>
              <span class="site-url">{{ site.url }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Side Panel -->
      <div class="card">
        <h3>Side Panel</h3>
        <div class="api-section">
          <button @click="openSidePanel" class="btn btn-primary">
            Open Panel
          </button>
          <button @click="closeSidePanel" class="btn btn-secondary">
            Close Panel
          </button>
          <div class="panel-status">
            <span>Status:</span>
            <span :class="['status-badge', panelOpen ? 'open' : 'closed']">
              {{ panelOpen ? 'Open' : 'Closed' }}
            </span>
          </div>
        </div>
      </div>

      <!-- Omnibox -->
      <div class="card">
        <h3>Omnibox</h3>
        <div class="api-section">
          <input
            v-model="omniboxInput"
            @input="onOmniboxInput"
            placeholder="Type to search..."
            class="omnibox-input"
          />
          <div v-if="omniboxSuggestions.length > 0" class="suggestions-list">
            <div
              v-for="(suggestion, index) in omniboxSuggestions"
              :key="index"
              class="suggestion-item"
              @click="selectSuggestion(suggestion)"
            >
              {{ suggestion.description }}
            </div>
          </div>
        </div>
      </div>

      <!-- Permissions -->
      <div class="card">
        <h3>Permissions</h3>
        <div class="api-section">
          <div class="permission-list">
            <div
              v-for="permission in permissions"
              :key="permission"
              class="permission-item"
            >
              <span>{{ permission }}</span>
              <span :class="['permission-status', hasPermission(permission) ? 'granted' : 'denied']">
                {{ hasPermission(permission) ? 'Granted' : 'Denied' }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Performance Metrics -->
      <div class="card full-width">
        <h3>Performance Metrics</h3>
        <div class="metrics-grid">
          <div v-for="(metric, api) in performanceMetrics" :key="api" class="metric-item">
            <span class="metric-api">{{ api }}</span>
            <span class="metric-value">{{ metric.avgDuration }}ms</span>
            <span class="metric-ops">{{ metric.opsPerSecond.toFixed(0) }} ops/s</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ContextMenusAPI, TopSitesAPI, SidePanelAPI, OmniboxAPI } from '../lib/extensions/apiExamples';

// State
const totalApis = ref(8);
const activeExtensions = ref(0);
const apiCallsPerMinute = ref(0);
const menuItems = ref<any[]>([]);
const topSites = ref<any[]>([]);
const panelOpen = ref(false);
const omniboxInput = ref('');
const omniboxSuggestions = ref<any[]>([]);
const permissions = ref(['contextMenus', 'webNavigation', 'sidePanel', 'omnibox', 'identity', 'topSites', 'devtools']);
const performanceMetrics = ref<Record<string, any>>({});

// Methods
const createTestMenuItem = async () => {
  try {
    const item = await ContextMenusAPI.createMenuItem(
      'test-extension',
      'Test Menu Item',
      ['all']
    );
    menuItems.value.push(item);
  } catch (error) {
    console.error('Failed to create menu item:', error);
  }
};

const loadMenuItems = async () => {
  try {
    const items = await ContextMenusAPI.getMenuItems('test-extension');
    menuItems.value = items;
  } catch (error) {
    console.error('Failed to load menu items:', error);
  }
};

const removeMenuItem = async (itemId: string) => {
  try {
    await ContextMenusAPI.removeMenuItem(itemId);
    menuItems.value = menuItems.value.filter(item => item.id !== itemId);
  } catch (error) {
    console.error('Failed to remove menu item:', error);
  }
};

const loadTopSites = async () => {
  try {
    const sites = await TopSitesAPI.getTopSites();
    topSites.value = sites.slice(0, 10);
  } catch (error) {
    console.error('Failed to load top sites:', error);
  }
};

const openSidePanel = async () => {
  try {
    await SidePanelAPI.setOptions('test-extension', 'panel.html', 'Test Panel');
    await SidePanelAPI.openPanel('test-extension');
    panelOpen.value = true;
  } catch (error) {
    console.error('Failed to open side panel:', error);
  }
};

const closeSidePanel = async () => {
  try {
    await SidePanelAPI.closePanel('test-extension');
    panelOpen.value = false;
  } catch (error) {
    console.error('Failed to close side panel:', error);
  }
};

const onOmniboxInput = async () => {
  if (omniboxInput.value.length > 2) {
    try {
      const suggestions = [
        {
          content: omniboxInput.value,
          description: `Search for "${omniboxInput.value}"`,
          type: 'search',
          deletable: false
        }
      ];
      await OmniboxAPI.setSuggestions('test-extension', suggestions);
      omniboxSuggestions.value = suggestions;
    } catch (error) {
      console.error('Failed to set suggestions:', error);
    }
  } else {
    omniboxSuggestions.value = [];
  }
};

const selectSuggestion = (suggestion: any) => {
  omniboxInput.value = suggestion.content;
  omniboxSuggestions.value = [];
};

const hasPermission = (permission: string) => {
  // Mock permission check
  return ['contextMenus', 'topSites'].includes(permission);
};

const loadPerformanceMetrics = () => {
  // Mock performance metrics
  performanceMetrics.value = {
    contextMenus: { avgDuration: 1.2, opsPerSecond: 850 },
    webNavigation: { avgDuration: 2.5, opsPerSecond: 400 },
    sidePanel: { avgDuration: 0.8, opsPerSecond: 1200 },
    omnibox: { avgDuration: 0.5, opsPerSecond: 2000 },
    topSites: { avgDuration: 1.0, opsPerSecond: 1000 },
  };
};

onMounted(() => {
  loadTopSites();
  loadPerformanceMetrics();
});
</script>

<style scoped>
.extension-api-dashboard {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.dashboard-header {
  margin-bottom: 30px;
}

.dashboard-header h2 {
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 8px;
}

.subtitle {
  color: #666;
  font-size: 14px;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
}

.card {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.card.full-width {
  grid-column: 1 / -1;
}

.card h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 15px;
  color: #333;
}

.api-stats {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  color: #666;
  font-size: 14px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #333;
}

.api-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.btn {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover {
  background: #0056b3;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #545b62;
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 12px;
}

.items-list,
.sites-list,
.suggestions-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
}

.item-row,
.site-row,
.suggestion-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #e0e0e0;
}

.item-row:last-child,
.site-row:last-child,
.suggestion-item:last-child {
  border-bottom: none;
}

.site-rank {
  font-weight: 600;
  color: #007bff;
  margin-right: 10px;
}

.site-url {
  color: #333;
  font-size: 14px;
}

.panel-status {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
}

.status-badge.open {
  background: #d4edda;
  color: #155724;
}

.status-badge.closed {
  background: #f8d7da;
  color: #721c24;
}

.omnibox-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 14px;
}

.omnibox-input:focus {
  outline: none;
  border-color: #007bff;
}

.suggestion-item {
  cursor: pointer;
}

.suggestion-item:hover {
  background: #f5f5f5;
}

.permission-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.permission-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f8f9fa;
  border-radius: 4px;
}

.permission-status {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
}

.permission-status.granted {
  background: #d4edda;
  color: #155724;
}

.permission-status.denied {
  background: #f8d7da;
  color: #721c24;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 4px;
}

.metric-api {
  font-size: 12px;
  color: #666;
  font-weight: 600;
}

.metric-value {
  font-size: 20px;
  font-weight: 600;
  color: #333;
}

.metric-ops {
  font-size: 12px;
  color: #666;
}
</style>
