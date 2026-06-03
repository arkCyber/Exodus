<!--
  Exodus Browser — Downloads preferences (chrome://settings/downloads).
-->
<template>
  <section class="settings-section downloads-settings" data-testid="downloads-settings-panel">
    <div class="settings-card">
      <h3 class="settings-card__title">{{ ui.title }}</h3>
      <p v-if="loading" class="settings-hint" data-testid="downloads-settings-loading">{{ ui.loading }}</p>
      <p v-else-if="loadError" class="settings-hint settings-hint--error">{{ loadError }}</p>
      <template v-else>
        <label>
          {{ ui.defaultDirLabel }}
          <input
            v-model="defaultDirectory"
            type="text"
            class="field"
            data-testid="downloads-default-dir"
            :placeholder="ui.defaultDirPlaceholder"
          />
        </label>
        <label class="checkbox-row">
          <input v-model="askForLocation" type="checkbox" data-testid="downloads-ask-location" />
          <span>{{ ui.askLocation }}</span>
        </label>
        <label class="checkbox-row">
          <input v-model="showNotifications" type="checkbox" data-testid="downloads-notifications" />
          <span>{{ ui.showNotifications }}</span>
        </label>
        <label class="checkbox-row">
          <input v-model="clearCompletedOnExit" type="checkbox" data-testid="downloads-clear-on-exit" />
          <span>{{ ui.clearOnExit }}</span>
        </label>
        <label>
          {{ ui.maxConcurrentLabel }}
          <input
            v-model.number="maxConcurrent"
            type="number"
            min="1"
            max="10"
            class="field field--narrow"
            data-testid="downloads-max-concurrent"
          />
        </label>
        <button
          type="button"
          class="nav-button secondary"
          data-testid="downloads-save"
          :disabled="saving"
          @click="() => void persist()"
        >
          {{ ui.save }}
        </button>
      </template>
    </div>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — download folder and behavior (Tauri DownloadManager settings).
 */
import { computed, onMounted, ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import { type AppLocale } from '@/lib/appLocale';
import { downloadsSettingsStrings } from '@/lib/downloadsSettingsUi';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => downloadsSettingsStrings(props.uiLocale));

type DownloadSettingsDto = {
  default_directory: string;
  ask_for_location: boolean;
  max_concurrent_downloads: number;
  auto_resume: boolean;
  max_retry_attempts: number;
  speed_limit: number;
  clear_completed_on_exit: boolean;
  show_notifications: boolean;
};

const loading = ref(true);
const saving = ref(false);
const loadError = ref('');
const defaultDirectory = ref('~/Downloads');
const askForLocation = ref(false);
const showNotifications = ref(true);
const clearCompletedOnExit = ref(false);
const maxConcurrent = ref(3);
const autoResume = ref(true);
const maxRetryAttempts = ref(3);
const speedLimit = ref(0);

/** Load settings from Tauri backend. */
async function load(): Promise<void> {
  loading.value = true;
  loadError.value = '';
  try {
    if (!isTauri()) {
      loading.value = false;
      return;
    }
    const s = await invoke<DownloadSettingsDto>('get_download_settings');
    defaultDirectory.value = String(s.default_directory ?? '~/Downloads');
    askForLocation.value = Boolean(s.ask_for_location);
    showNotifications.value = Boolean(s.show_notifications);
    clearCompletedOnExit.value = Boolean(s.clear_completed_on_exit);
    maxConcurrent.value = Number(s.max_concurrent_downloads) || 3;
    autoResume.value = Boolean(s.auto_resume);
    maxRetryAttempts.value = Number(s.max_retry_attempts) || 3;
    speedLimit.value = Number(s.speed_limit) || 0;
  } catch (error) {
    console.error('DownloadsSettings.load failed:', error);
    loadError.value = ui.value.loadError;
  } finally {
    loading.value = false;
  }
}

/** Persist settings to Tauri backend. */
async function persist(): Promise<void> {
  if (!isTauri()) return;
  saving.value = true;
  try {
    await invoke('update_download_settings', {
      settings: {
        default_directory: defaultDirectory.value.trim() || '~/Downloads',
        ask_for_location: askForLocation.value,
        max_concurrent_downloads: Math.min(10, Math.max(1, maxConcurrent.value || 3)),
        auto_resume: autoResume.value,
        max_retry_attempts: maxRetryAttempts.value,
        speed_limit: speedLimit.value,
        clear_completed_on_exit: clearCompletedOnExit.value,
        show_notifications: showNotifications.value,
      },
    });
    emit('status', ui.value.saved);
  } catch (error) {
    console.error('DownloadsSettings.persist failed:', error);
    emit('status', ui.value.saveError);
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void load();
});
</script>

<style scoped>
.field--narrow {
  max-width: 120px;
}
.settings-hint--error {
  color: #d93025;
}
</style>
