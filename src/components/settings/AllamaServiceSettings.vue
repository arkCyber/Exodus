<!--
  Exodus Browser — Allama inference microservice settings.
-->
<template>
  <section class="settings-section" data-testid="allama-service-settings">
    <h3>Allama service</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <p class="hint">Local inference on port {{ aiPort }} (Ollama-compatible API).</p>
      <label class="checkbox-row">
        <input :checked="spawnAllama" type="checkbox" @change="onSpawnChange" data-testid="allama-spawn" />
        <span>Start Allama with the app</span>
      </label>
      <label>
        AI port
        <input :value="aiPort" type="number" min="1" max="65535" @input="onPortInput" data-testid="allama-port" />
      </label>
      <div v-if="status" class="status-box">
        <span :class="{ online: status.endpointOnline }">{{ status.state }} · port {{ status.port }}</span>
        <span class="hint">{{ status.detail }}</span>
      </div>
      <div class="toolbar">
        <button type="button" class="nav-button secondary" :disabled="busy" @click="() => void refresh()" data-testid="allama-refresh">Refresh</button>
        <button type="button" class="nav-button secondary" :disabled="busy" @click="() => void start()" data-testid="allama-start">Start</button>
        <button type="button" class="nav-button secondary" :disabled="busy" @click="() => void stop()" data-testid="allama-stop">Stop</button>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

defineProps<{ spawnAllama: boolean; aiPort: number }>();
const emit = defineEmits<{
  status: [message: string];
  spawnAllamaChange: [enabled: boolean];
  aiPortChange: [port: number];
}>();

type AllamaStatusDto = {
  state: string;
  port: number;
  detail: string;
  endpointOnline: boolean;
};

const status = ref<AllamaStatusDto | null>(null);
const busy = ref(false);
const loading = ref(true);

function onSpawnChange(e: Event): void {
  emit('spawnAllamaChange', (e.target as HTMLInputElement).checked);
}

function onPortInput(e: Event): void {
  const n = Number((e.target as HTMLInputElement).value);
  if (Number.isFinite(n)) emit('aiPortChange', n);
}

async function refresh(): Promise<void> {
  loading.value = true;
  try {
    status.value = await invoke<AllamaStatusDto>('allama_service_status');
  } catch (error) {
    console.error('allama_service_status failed:', error);
  } finally {
    loading.value = false;
  }
}

async function start(): Promise<void> {
  busy.value = true;
  try {
    status.value = await invoke<AllamaStatusDto>('allama_service_start');
    emit('status', 'Allama started');
  } catch (error) {
    emit('status', 'Failed to start Allama');
  } finally {
    busy.value = false;
  }
}

async function stop(): Promise<void> {
  busy.value = true;
  try {
    status.value = await invoke<AllamaStatusDto>('allama_service_stop');
    emit('status', 'Allama stopped');
  } catch (error) {
    emit('status', 'Failed to stop Allama');
  } finally {
    busy.value = false;
  }
}

onMounted(() => void refresh());
</script>

<style scoped>
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.status-box { font-size: 12px; margin: 8px 0; }
.status-box .online { color: #22c55e; }
.toolbar { display: flex; gap: 8px; flex-wrap: wrap; }
.checkbox-row { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.nav-button.secondary { padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-bg-tertiary, #404040); color: #fff; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
</style>
