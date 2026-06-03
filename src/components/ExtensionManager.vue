<template>
  <div class="extension-manager">
    <div class="header">
      <h2>Extension Manager</h2>
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search extensions..."
          class="search-input"
        />
        <span class="search-icon">🔍</span>
      </div>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
        <span class="count" v-if="tab.count">{{ tab.count }}</span>
      </button>
    </div>

    <div class="content">
      <div v-if="activeTab === 'installed'" class="extensions-grid">
        <div
          v-for="extension in filteredExtensions"
          :key="extension.id"
          class="extension-card"
        >
          <div class="card-header">
            <div class="extension-icon">
              <img
                v-if="extension.icon"
                :src="extension.icon"
                :alt="extension.name"
              />
              <span v-else class="default-icon">📦</span>
            </div>
            <div class="extension-info">
              <h3>{{ extension.name }}</h3>
              <p class="version">v{{ extension.version }}</p>
            </div>
            <div class="card-actions">
              <button
                @click="toggleExtension(extension)"
                :class="['toggle-btn', { active: extension.enabled }]"
              >
                {{ extension.enabled ? 'Disable' : 'Enable' }}
              </button>
              <button @click="showExtensionMenu(extension)" class="menu-btn">
                ⋮
              </button>
            </div>
          </div>

          <div class="card-body">
            <p class="description">{{ extension.description }}</p>
            <div class="permissions">
              <span
                v-for="permission in extension.permissions.slice(0, 3)"
                :key="permission"
                class="permission-tag"
              >
                {{ permission }}
              </span>
              <span
                v-if="extension.permissions.length > 3"
                class="permission-tag more"
              >
                +{{ extension.permissions.length - 3 }}
              </span>
            </div>
          </div>

          <div class="card-footer">
            <div class="stats">
              <span class="stat">
                <span class="stat-icon">📊</span>
                {{ extension.apiCalls || 0 }} calls
              </span>
              <span class="stat">
                <span class="stat-icon">⚡</span>
                {{ extension.performance || 0 }}ms
              </span>
            </div>
            <button @click="viewDetails(extension)" class="details-btn">
              Details
            </button>
          </div>
        </div>

        <div v-if="filteredExtensions.length === 0" class="empty-state">
          <span class="empty-icon">📦</span>
          <p>No extensions found</p>
        </div>
      </div>

      <div v-if="activeTab === 'updates'" class="updates-list">
        <div
          v-for="update in availableUpdates"
          :key="update.id"
          class="update-item"
        >
          <div class="update-icon">
            <img
              v-if="update.icon"
              :src="update.icon"
              :alt="update.name"
            />
            <span v-else class="default-icon">📦</span>
          </div>
          <div class="update-info">
            <h3>{{ update.name }}</h3>
            <p class="version">
              {{ update.currentVersion }} → {{ update.newVersion }}
            </p>
            <p class="release-notes">{{ update.releaseNotes }}</p>
          </div>
          <div class="update-actions">
            <button @click="updateExtension(update)" class="update-btn">
              Update
            </button>
            <button @click="dismissUpdate(update)" class="dismiss-btn">
              Dismiss
            </button>
          </div>
        </div>

        <div v-if="availableUpdates.length === 0" class="empty-state">
          <span class="empty-icon">✅</span>
          <p>All extensions are up to date</p>
        </div>
      </div>

      <div v-if="activeTab === 'store'" class="store-grid">
        <div
          v-for="item in storeExtensions"
          :key="item.id"
          class="store-item"
        >
          <div class="store-icon">
            <img v-if="item.icon" :src="item.icon" :alt="item.name" />
            <span v-else class="default-icon">📦</span>
          </div>
          <div class="store-info">
            <h3>{{ item.name }}</h3>
            <p class="author">by {{ item.author }}</p>
            <p class="description">{{ item.description }}</p>
            <div class="rating">
              <span class="stars">{{ '⭐'.repeat(item.rating) }}</span>
              <span class="count">({{ item.reviewCount }})</span>
            </div>
          </div>
          <button @click="installExtension(item)" class="install-btn">
            Install
          </button>
        </div>
      </div>
    </div>

    <div v-if="showMenu" class="context-menu" :style="menuPosition">
      <button @click="viewDetails(selectedExtension)">View Details</button>
      <button @click="managePermissions(selectedExtension)">
        Manage Permissions
      </button>
      <button @click="viewLogs(selectedExtension)">View Logs</button>
      <button @click="clearData(selectedExtension)">Clear Data</button>
      <div class="divider"></div>
      <button @click="uninstallExtension(selectedExtension)" class="danger">
        Uninstall
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';

interface Extension {
  id: string;
  name: string;
  version: string;
  description: string;
  icon?: string;
  enabled: boolean;
  permissions: string[];
  apiCalls?: number;
  performance?: number;
}

interface Update {
  id: string;
  name: string;
  currentVersion: string;
  newVersion: string;
  releaseNotes: string;
  icon?: string;
}

interface StoreItem {
  id: string;
  name: string;
  author: string;
  description: string;
  icon?: string;
  rating: number;
  reviewCount: number;
}

const searchQuery = ref('');
const activeTab = ref('installed');
const showMenu = ref(false);
const selectedExtension = ref<Extension | null>(null);
const menuPosition = ref({ top: '0px', left: '0px' });

const tabs = [
  { id: 'installed', label: 'Installed', count: 0 },
  { id: 'updates', label: 'Updates', count: 0 },
  { id: 'store', label: 'Store', count: 0 },
];

const extensions = ref<Extension[]>([
  {
    id: 'sample-extension-api-demo',
    name: 'Extension API Demo',
    version: '1.0.0',
    description: 'Demonstrates various Extension APIs',
    icon: '',
    enabled: true,
    permissions: ['contextMenus', 'webNavigation', 'sidePanel', 'omnibox'],
    apiCalls: 1234,
    performance: 45,
  },
  {
    id: 'ad-blocker',
    name: 'Ad Blocker',
    version: '2.1.0',
    description: 'Blocks ads and trackers',
    icon: '',
    enabled: true,
    permissions: ['webNavigation', 'storage', 'tabs'],
    apiCalls: 5678,
    performance: 32,
  },
  {
    id: 'password-manager',
    name: 'Password Manager',
    version: '1.5.2',
    description: 'Secure password storage',
    icon: '',
    enabled: false,
    permissions: ['storage', 'tabs', 'notifications'],
    apiCalls: 890,
    performance: 28,
  },
]);

const availableUpdates = ref<Update[]>([
  {
    id: 'ad-blocker',
    name: 'Ad Blocker',
    currentVersion: '2.1.0',
    newVersion: '2.2.0',
    releaseNotes: 'Improved blocking performance',
    icon: '',
  },
]);

const storeExtensions = ref<StoreItem[]>([
  {
    id: 'dark-mode',
    name: 'Dark Mode',
    author: 'Theme Dev',
    description: 'Enable dark mode on all websites',
    rating: 4,
    reviewCount: 1234,
  },
  {
    id: 'grammar-checker',
    name: 'Grammar Checker',
    author: 'Linguist AI',
    description: 'Real-time grammar checking',
    rating: 5,
    reviewCount: 5678,
  },
  {
    id: 'screenshot-tool',
    name: 'Screenshot Tool',
    author: 'Capture Co',
    description: 'Advanced screenshot capabilities',
    rating: 4,
    reviewCount: 890,
  },
]);

const filteredExtensions = computed(() => {
  if (!searchQuery.value) return extensions.value;
  const query = searchQuery.value.toLowerCase();
  return extensions.value.filter(
    (ext) =>
      ext.name.toLowerCase().includes(query) ||
      ext.description.toLowerCase().includes(query)
  );
});

tabs[0].count = extensions.value.length;
tabs[1].count = availableUpdates.value.length;

const toggleExtension = (extension: Extension) => {
  extension.enabled = !extension.enabled;
};

const showExtensionMenu = (extension: Extension, event: MouseEvent) => {
  selectedExtension.value = extension;
  menuPosition.value = {
    top: `${event.clientY}px`,
    left: `${event.clientX}px`,
  };
  showMenu.value = true;
};

const viewDetails = (extension: Extension) => {
  console.log('View details:', extension.id);
  showMenu.value = false;
};

const managePermissions = (extension: Extension) => {
  console.log('Manage permissions:', extension.id);
  showMenu.value = false;
};

const viewLogs = (extension: Extension) => {
  console.log('View logs:', extension.id);
  showMenu.value = false;
};

const clearData = (extension: Extension) => {
  console.log('Clear data:', extension.id);
  showMenu.value = false;
};

const uninstallExtension = (extension: Extension) => {
  if (confirm(`Uninstall ${extension.name}?`)) {
    extensions.value = extensions.value.filter((e) => e.id !== extension.id);
  }
  showMenu.value = false;
};

const updateExtension = (update: Update) => {
  console.log('Update extension:', update.id);
  availableUpdates.value = availableUpdates.value.filter((u) => u.id !== update.id);
};

const dismissUpdate = (update: Update) => {
  availableUpdates.value = availableUpdates.value.filter((u) => u.id !== update.id);
};

const installExtension = (item: StoreItem) => {
  console.log('Install extension:', item.id);
};

const closeMenu = () => {
  showMenu.value = false;
};

onMounted(() => {
  document.addEventListener('click', closeMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeMenu);
});
</script>

<style scoped>
.extension-manager {
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  overflow: hidden;
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
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.search-box {
  position: relative;
  width: 300px;
}

.search-input {
  width: 100%;
  padding: 10px 40px 10px 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.search-icon {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 16px;
  color: var(--text-secondary);
}

.tabs {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  padding: 0 24px;
}

.tab {
  padding: 14px 20px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
}

.tab:hover {
  color: var(--text-primary);
}

.tab.active {
  color: var(--primary-color);
  border-bottom-color: var(--primary-color);
}

.tab .count {
  background: var(--primary-color);
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  margin-left: 6px;
}

.content {
  padding: 24px;
  min-height: 400px;
}

.extensions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.extension-card {
  background: var(--bg-secondary);
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--border-color);
  transition: box-shadow 0.2s;
}

.extension-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.extension-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  overflow: hidden;
  flex-shrink: 0;
}

.extension-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.default-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--bg-primary);
  font-size: 24px;
}

.extension-info {
  flex: 1;
}

.extension-info h3 {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.version {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.card-actions {
  display: flex;
  gap: 8px;
}

.toggle-btn {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s;
}

.toggle-btn.active {
  background: var(--success-bg);
  color: var(--success-text);
  border-color: var(--success-border);
}

.menu-btn {
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  border: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-secondary);
}

.card-body {
  padding: 16px;
}

.description {
  margin: 0 0 12px 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.permissions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.permission-tag {
  padding: 4px 8px;
  background: var(--bg-primary);
  border-radius: 4px;
  font-size: 11px;
  color: var(--text-secondary);
}

.permission-tag.more {
  color: var(--primary-color);
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.stats {
  display: flex;
  gap: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary);
}

.stat-icon {
  font-size: 14px;
}

.details-btn {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: var(--primary-color);
  color: white;
  transition: background 0.2s;
}

.details-btn:hover {
  background: var(--primary-hover);
}

.updates-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.update-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.update-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  overflow: hidden;
  flex-shrink: 0;
}

.update-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.update-info {
  flex: 1;
}

.update-info h3 {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.update-info .version {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--primary-color);
  font-weight: 500;
}

.release-notes {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary);
}

.update-actions {
  display: flex;
  gap: 8px;
}

.update-btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: var(--primary-color);
  color: white;
}

.dismiss-btn {
  padding: 8px 16px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-color);
  background: var(--bg-primary);
  color: var(--text-primary);
}

.store-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.store-item {
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  gap: 12px;
}

.store-icon {
  width: 64px;
  height: 64px;
  border-radius: 12px;
  overflow: hidden;
  align-self: center;
}

.store-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.store-info h3 {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  text-align: center;
}

.author {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: center;
}

.store-info .description {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.4;
  text-align: center;
}

.rating {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.stars {
  font-size: 14px;
}

.rating .count {
  font-size: 12px;
  color: var(--text-secondary);
}

.install-btn {
  padding: 10px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: var(--primary-color);
  color: white;
  transition: background 0.2s;
}

.install-btn:hover {
  background: var(--primary-hover);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.context-menu {
  position: fixed;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  padding: 8px 0;
  min-width: 180px;
  z-index: 1000;
}

.context-menu button {
  width: 100%;
  padding: 10px 16px;
  background: none;
  border: none;
  text-align: left;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.1s;
}

.context-menu button:hover {
  background: var(--bg-hover);
}

.context-menu button.danger {
  color: var(--error-text);
}

.context-menu button.danger:hover {
  background: var(--error-bg);
}

.divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}
</style>
