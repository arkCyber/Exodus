<!--
  Exodus Browser — vertical tab layout settings.
-->
<template>
  <section id="settings-section-tabs" class="settings-section" data-testid="vertical-tabs-settings">
    <h3>Tab layout</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <p class="hint">Chrome-style vertical tabs along the side of the window.</p>
      <template v-if="settings">
        <label class="checkbox-row">
          <input v-model="settings.enabled" type="checkbox" @change="() => void persist()" data-testid="vertical-tabs-enabled" />
          <span>Vertical tabs</span>
        </label>
        <template v-if="settings.enabled">
          <label>
            Position
            <select v-model="settings.position" @change="() => void persist()" data-testid="vertical-tabs-position">
              <option value="Left">Left</option>
              <option value="Right">Right</option>
            </select>
          </label>
          <label>
            Width mode
            <select v-model="settings.width_mode" @change="() => void persist()" data-testid="vertical-tabs-width-mode">
              <option value="Fixed">Fixed</option>
              <option value="Auto">Auto</option>
              <option value="Compact">Compact</option>
            </select>
          </label>
        </template>
        <div class="toolbar">
          <button type="button" class="nav-button secondary" @click="() => void resetToDefaults()" data-testid="vertical-tabs-reset">Reset to defaults</button>
        </div>
      </template>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { loadVerticalTabSettings, saveVerticalTabSettings, type VerticalTabSettings } from '$lib/verticalTabs';

const emit = defineEmits<{
  status: [message: string];
  layoutChange: [settings: VerticalTabSettings];
}>();

const settings = ref<VerticalTabSettings | null>(null);
const loading = ref(true);

async function load(): Promise<void> {
  loading.value = true;
  try {
    settings.value = await loadVerticalTabSettings();
    if (settings.value) emit('layoutChange', settings.value);
  } catch (error) {
    console.error('VerticalTabsSettings load failed:', error);
    emit('status', 'Failed to load tab layout settings');
  } finally {
    loading.value = false;
  }
}

async function persist(): Promise<void> {
  if (!settings.value) return;
  try {
    await saveVerticalTabSettings(settings.value);
    emit('layoutChange', settings.value);
    emit('status', 'Tab layout updated');
  } catch (error) {
    console.error('saveVerticalTabSettings failed:', error);
  }
}

async function resetToDefaults(): Promise<void> {
  const defaults: VerticalTabSettings = {
    enabled: false,
    position: 'Left',
    width_mode: 'Fixed',
    fixed_width: 220,
    show_icons: true,
    show_titles: true,
    show_close_buttons: true,
    collapse_inactive: false,
    tab_spacing: 0,
  };
  settings.value = defaults;
  await persist();
  emit('status', 'Tab layout reset to defaults');
}

onMounted(() => void load());
</script>

<style scoped>
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.checkbox-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
label { display: flex; flex-direction: column; gap: 4px; margin-bottom: 8px; font-size: 13px; }
select { padding: 6px; border-radius: 6px; }
.toolbar { display: flex; gap: 8px; margin-top: 12px; }
</style>
