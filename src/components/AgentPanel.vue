<!--
  Exodus Browser — web automation agent sidebar panel (Vue 3).
-->
<template>
  <div class="agent-panel">
    <p v-if="domSummary" class="agent-dom-summary">{{ domSummary }}</p>

    <div class="agent-strategy-row">
      <label class="agent-strategy-label" for="agent-strategy-select">Strategy</label>
      <select
        id="agent-strategy-select"
        v-model="selectedStrategyId"
        class="agent-strategy-select"
        :disabled="executing"
      >
        <option v-for="tpl in strategyTemplates" :key="tpl.id" :value="tpl.id">
          {{ tpl.name }}
        </option>
      </select>
      <button
        type="button"
        class="agent-preset-btn agent-strategy-run"
        :disabled="executing || !selectedStrategyId"
        @click="emit('run-strategy', selectedStrategyId)"
      >
        Run strategy
      </button>
      <button type="button" class="agent-preset-btn" :disabled="executing" @click="showSaveForm = !showSaveForm">
        {{ showSaveForm ? 'Cancel' : 'Save…' }}
      </button>
      <button
        v-if="selectedIsCustom"
        type="button"
        class="agent-preset-btn agent-strategy-delete"
        :disabled="executing"
        @click="deleteSelectedStrategy"
      >
        Delete
      </button>
    </div>

    <div v-if="showSaveForm" class="agent-save-strategy">
      <label class="agent-save-label" for="strategy-save-name">Name</label>
      <input
        id="strategy-save-name"
        v-model="saveName"
        type="text"
        class="agent-input"
        placeholder="My workflow"
        :disabled="executing"
      />
      <label class="agent-save-label" for="strategy-save-desc">Description</label>
      <input
        id="strategy-save-desc"
        v-model="saveDescription"
        type="text"
        class="agent-input"
        placeholder="Optional note"
        :disabled="executing"
      />
      <p v-if="saveError" class="agent-save-error">{{ saveError }}</p>
      <button
        type="button"
        class="agent-btn-small"
        :disabled="executing || !command.trim()"
        @click="saveCurrentAsStrategy"
      >
        Save to browser
      </button>
    </div>

    <div class="agent-quick-row">
      <button
        v-for="preset in AGENT_PRESETS"
        :key="preset.id"
        type="button"
        class="agent-preset-btn"
        :disabled="executing"
        @click="emit('preset', JSON.stringify(preset.action))"
      >
        {{ preset.label }}
      </button>
    </div>

    <input
      type="text"
      :value="command"
      placeholder="JSON, plan: goal, scroll down, or ask: question"
      class="agent-input"
      :disabled="executing"
      @input="onCommandInput"
      @keydown.enter="emit('execute')"
    />

    <div class="agent-buttons">
      <button type="button" class="agent-btn-primary" :disabled="executing" @click="emit('execute')">
        {{ executing ? 'Running…' : 'Run' }}
      </button>
      <button type="button" class="agent-btn-small" :disabled="executing" @click="emit('compress')">
        Compress DOM
      </button>
      <button type="button" class="agent-btn-small" :disabled="executing" @click="emit('ask-ai')">
        Ask AI
      </button>
      <button type="button" class="agent-btn-small" @click="emit('back')">Back to AI</button>
    </div>

    <div class="agent-log" role="log" aria-live="polite">
      <p v-if="log.length === 0" class="agent-log-empty">
        Run a preset or enter JSON. Example: scroll down, GetContent, ExtractLinks.
      </p>
      <div v-for="(entry, i) in log" :key="i" class="log-entry">{{ entry }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { AGENT_PRESETS } from '$lib/agentActions';
import {
  buildCustomTemplateFromCommand,
  deleteCustomHermesTemplate,
  isBuiltinHermesTemplate,
  listHermesStrategyTemplates,
  upsertCustomHermesTemplate,
} from '$lib/hermesStrategies';

const props = defineProps<{
  command: string;
  log: string[];
  executing: boolean;
  domSummary: string;
}>();

const emit = defineEmits<{
  execute: [];
  compress: [];
  back: [];
  preset: [actionJson: string];
  'command-change': [value: string];
  'ask-ai': [];
  'run-strategy': [templateId: string];
  'strategy-saved': [message: string];
}>();

const templateVersion = ref(0);
const selectedStrategyId = ref('');
const saveName = ref('');
const saveDescription = ref('');
const saveError = ref('');
const showSaveForm = ref(false);

const strategyTemplates = computed(() => {
  templateVersion.value;
  const list = listHermesStrategyTemplates();
  if (!selectedStrategyId.value && list.length > 0) {
    selectedStrategyId.value = list[0].id;
  }
  return list;
});

const selectedIsCustom = computed(() =>
  selectedStrategyId.value ? !isBuiltinHermesTemplate(selectedStrategyId.value) : false,
);

function onCommandInput(e: Event): void {
  emit('command-change', (e.target as HTMLInputElement).value);
}

function bumpTemplates(): void {
  templateVersion.value += 1;
}

function saveCurrentAsStrategy(): void {
  saveError.value = '';
  try {
    const template = buildCustomTemplateFromCommand(
      saveName.value || 'My strategy',
      saveDescription.value,
      props.command,
    );
    upsertCustomHermesTemplate(template);
    bumpTemplates();
    selectedStrategyId.value = template.id;
    showSaveForm.value = false;
    saveName.value = '';
    saveDescription.value = '';
    emit('strategy-saved', `Saved strategy "${template.name}"`);
  } catch (err) {
    saveError.value = err instanceof Error ? err.message : String(err);
  }
}

function deleteSelectedStrategy(): void {
  if (!selectedStrategyId.value || !deleteCustomHermesTemplate(selectedStrategyId.value)) {
    return;
  }
  bumpTemplates();
  const first = strategyTemplates.value[0];
  selectedStrategyId.value = first?.id ?? '';
  emit('strategy-saved', 'Custom strategy deleted');
}
</script>

<style scoped>
.agent-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  min-height: 0;
}

.agent-dom-summary {
  margin: 0;
  font-size: 12px;
  color: #9ca3af;
  padding: 6px 8px;
  background: #1a1a1a;
  border-radius: 6px;
  border: 1px solid #333;
}

.agent-strategy-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.agent-strategy-label {
  font-size: 11px;
  color: #9ca3af;
}

.agent-strategy-select {
  flex: 1;
  min-width: 120px;
  background: #1a1a1a;
  border: 1px solid #404040;
  color: #e0e0e0;
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 12px;
}

.agent-preset-btn,
.agent-btn-small {
  font-size: 11px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid #555;
  background: #2a2a2a;
  color: #ddd;
  cursor: pointer;
}

.agent-quick-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-input {
  width: 100%;
  padding: 8px;
  border-radius: 6px;
  border: 1px solid #404040;
  background: #1a1a1a;
  color: #e0e0e0;
}

.agent-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.agent-btn-primary {
  padding: 6px 12px;
  background: #6366f1;
  border: none;
  border-radius: 6px;
  color: #fff;
  cursor: pointer;
}

.agent-log {
  flex: 1;
  min-height: 80px;
  overflow-y: auto;
  font-size: 11px;
  font-family: ui-monospace, monospace;
  background: #111;
  border: 1px solid #333;
  border-radius: 6px;
  padding: 8px;
}

.log-entry {
  margin-bottom: 4px;
  color: #a3a3a3;
}

.agent-log-empty {
  color: #6b7280;
  margin: 0;
}

.agent-save-strategy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: #1a1a1a;
  border: 1px solid #404040;
  border-radius: 6px;
}

.agent-save-error {
  color: #f87171;
  margin: 0;
  font-size: 12px;
}
</style>
