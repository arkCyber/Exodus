<!--
  Exodus Browser — synced tabs from other devices (Firefox sidebar).
-->
<template>
  <div class="list-panel synced-panel exodus-sidebar-panel">
    <div class="history-panel-actions">
      <button type="button" class="nav-button secondary" :disabled="loading" @click="() => void refresh()">
        {{ loading ? 'Syncing…' : 'Refresh' }}
      </button>
    </div>
    <p v-if="!syncEnabled" class="muted">Enable mobile sync in Settings to update tabs from your devices.</p>
    <template v-for="device in devices" :key="device.deviceId">
      <h4 class="memory-section-title">{{ device.deviceName }}</h4>
      <div
        v-for="tab in device.tabs"
        :key="tab.id"
        class="list-item"
        role="link"
        tabindex="0"
        @click="() => tab.url ? emit('navigate', tab.url) : null"
      >
        <div class="list-title">{{ tab.title }}</div>
        <div class="list-sub">{{ tab.url }}</div>
      </div>
      <p v-if="device.tabs.length === 0" class="muted">No open tabs on this device.</p>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { refreshSyncedTabs, type OpenTabSnapshot, type SyncedDevice } from '$lib/syncedTabs';
import { isMobileSyncEnabled } from '$lib/mobileSync';

const props = withDefaults(
  defineProps<{ openTabs?: OpenTabSnapshot[] }>(),
  { openTabs: () => [] },
);

const emit = defineEmits<{ navigate: [url: string] }>();

const devices = ref<SyncedDevice[]>([]);
const loading = ref(false);
const syncEnabled = ref(false);

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    syncEnabled.value = await isMobileSyncEnabled();
    devices.value = await refreshSyncedTabs(props.openTabs);
  } catch (error) {
    console.error('SidebarSyncedTabsPanel refresh failed:', error);
    devices.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});

watch(
  () => props.openTabs,
  () => {
    void refresh();
  },
  { deep: true },
);
</script>
