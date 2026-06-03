<!--
  Exodus Browser — InferenceEngine registry settings (Tauri inference_* commands).
-->
<template>
  <section class="settings-section" data-testid="inference-engine-settings">
    <h3>Inference engine</h3>
    <div v-if="loading" class="loading-state">Loading…</div>
    <template v-else>
      <p class="hint">{{ engineStatus || 'Local model runtime' }}</p>
      <label>
        Model
        <select v-model="selectedModel" :disabled="busy" data-testid="inference-model-select">
          <option v-for="m in models" :key="m.name" :value="m.name">{{ m.name }}{{ m.loaded ? ' (loaded)' : '' }}</option>
        </select>
      </label>
      <div class="toolbar">
        <button type="button" class="nav-button secondary" :disabled="busy" @click="() => void refresh()" data-testid="inference-refresh">Refresh</button>
        <button type="button" class="nav-button secondary" :disabled="busy || !selectedModel" @click="() => void loadModel()" data-testid="inference-load">Load</button>
        <button type="button" class="nav-button secondary" :disabled="busy" @click="() => void unloadModel()" data-testid="inference-unload">Unload</button>
      </div>
      <p v-if="statsPreview" class="mono">{{ statsPreview }}</p>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import {
  inferenceGetLoadedModel,
  inferenceGetStats,
  inferenceGetStatus,
  inferenceListModels,
  inferenceLoadModel,
  inferenceUnloadModel,
  type InferenceModelInfo,
} from '$lib/inferenceClient';

const props = defineProps<{ aiModel: string }>();
const emit = defineEmits<{
  status: [message: string];
  /** Emitted when user picks a different model (triggers config auto-save). */
  modelChange: [name: string];
}>();

const models = ref<InferenceModelInfo[]>([]);
const selectedModel = ref('');
const engineStatus = ref('');
const statsPreview = ref('');
const busy = ref(false);
const loading = ref(true);

async function refresh(): Promise<void> {
  loading.value = true;
  busy.value = true;
  try {
    models.value = await inferenceListModels();
    const loaded = await inferenceGetLoadedModel();
    engineStatus.value = await inferenceGetStatus();
    const stats = await inferenceGetStats();
    statsPreview.value = JSON.stringify(stats).slice(0, 200);
    if (!selectedModel.value && models.value.length) {
      selectedModel.value =
        models.value.find((m) => m.name === props.aiModel)?.name ??
        models.value.find((m) => m.loaded)?.name ??
        models.value[0].name;
    }
    if (loaded) selectedModel.value = loaded;
  } catch (error) {
    emit('status', `Inference: ${error}`);
  } finally {
    busy.value = false;
    loading.value = false;
  }
}

async function loadModel(): Promise<void> {
  if (!selectedModel.value) return;
  busy.value = true;
  try {
    await inferenceLoadModel(selectedModel.value);
    emit('status', `Loaded ${selectedModel.value}`);
    await refresh();
  } catch (error) {
    emit('status', `Load failed: ${error}`);
  } finally {
    busy.value = false;
  }
}

async function unloadModel(): Promise<void> {
  busy.value = true;
  try {
    await inferenceUnloadModel();
    emit('status', 'Model unloaded');
    await refresh();
  } catch (error) {
    emit('status', `Unload failed: ${error}`);
  } finally {
    busy.value = false;
  }
}

let syncingModelFromParent = false;

watch(
  () => props.aiModel,
  (name) => {
    if (!name || name === selectedModel.value) return;
    syncingModelFromParent = true;
    selectedModel.value = name;
    syncingModelFromParent = false;
  },
);

watch(selectedModel, (name) => {
  if (syncingModelFromParent || !name || name === props.aiModel) return;
  emit('modelChange', name);
});

onMounted(() => void refresh());
</script>

<style scoped>
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.toolbar { display: flex; gap: 8px; margin-top: 8px; flex-wrap: wrap; }
.mono { font-size: 11px; word-break: break-all; color: var(--color-text-secondary, #888); }
.nav-button.secondary { padding: 6px 12px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-bg-tertiary, #404040); color: #fff; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }
label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
select { padding: 6px; border-radius: 6px; }
</style>
