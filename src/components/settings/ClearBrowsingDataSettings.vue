<!--
  Exodus Browser — Clear browsing data (Settings → Privacy and security).
-->
<template>
  <section class="settings-section clear-browsing-data" data-testid="clear-browsing-data-panel">
    <h3 class="settings-card__title">{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label class="checkbox-row">
        <input v-model="clearCookies" type="checkbox" data-testid="clear-data-cookies" />
        <span>{{ ui.cookies }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="clearHistory" type="checkbox" data-testid="clear-data-history" />
        <span>{{ ui.history }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="clearLocalStorage" type="checkbox" data-testid="clear-data-local-storage" />
        <span>{{ ui.localStorage }}</span>
      </label>
      <label class="checkbox-row">
        <input v-model="clearCache" type="checkbox" data-testid="clear-data-cache" />
        <span>{{ ui.cache }}</span>
      </label>
      <button
        type="button"
        class="nav-button secondary danger"
        data-testid="clear-data-submit"
        :disabled="clearing"
        @click="() => void submitClear()"
      >
        {{ clearing ? ui.clearing : ui.clearButton }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — wipe cookies, history, and related local data via Tauri `clear_browsing_data`.
 */
import { computed, ref, onMounted } from 'vue';
import { isTauri } from '@tauri-apps/api/core';
import { type AppLocale } from '@/lib/appLocale';
import { clearBrowsingDataSettingsStrings } from '@/lib/clearBrowsingDataSettingsUi';
import { clearBrowsingData } from '$lib/browserIntegrations';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
  cleared: [];
}>();

const ui = computed(() => clearBrowsingDataSettingsStrings(props.uiLocale));

const clearCookies = ref(true);
const clearHistory = ref(true);
const clearLocalStorage = ref(false);
const clearCache = ref(false);
const clearing = ref(false);
const loading = ref(true);

/** Invoke backend clear for selected categories. */
async function submitClear(): Promise<void> {
  if (!clearCookies.value && !clearHistory.value && !clearLocalStorage.value && !clearCache.value) {
    emit('status', ui.value.nothingSelected);
    return;
  }
  if (!isTauri()) {
    emit('status', ui.value.clearError);
    return;
  }
  clearing.value = true;
  try {
    const summary = await clearBrowsingData({
      clearCookies: clearCookies.value,
      clearHistory: clearHistory.value,
      clearLocalStorage: clearLocalStorage.value,
      clearCache: clearCache.value,
    });
    emit('status', ui.value.cleared(summary));
    emit('cleared');
  } catch (error) {
    console.error('ClearBrowsingDataSettings.submitClear failed:', error);
    emit('status', ui.value.clearError);
  } finally {
    clearing.value = false;
  }
}

onMounted(() => {
  loading.value = false;
});
</script>
