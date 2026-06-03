<!--
  Exodus Browser — edit tab group name and color.
-->
<template>
  <template v-if="offer">
    <button type="button" class="prompt-backdrop" aria-label="Cancel" @click="emit('cancel')" />
    <div class="prompt-dialog" role="dialog" aria-labelledby="tg-edit-title">
      <h3 id="tg-edit-title">Edit tab group</h3>
      <label class="field">
        <span>Name</span>
        <input v-model="titleInput" type="text" />
      </label>
      <p class="color-label">Color</p>
      <div class="color-row">
        <button
          v-for="c in TAB_GROUP_COLORS"
          :key="c"
          type="button"
          class="color-swatch"
          :class="{ selected: colorInput === c }"
          :style="{ '--swatch': tabGroupColorCss(c) }"
          :title="c"
          @click="colorInput = c"
        />
      </div>
      <div class="prompt-actions">
        <button type="button" class="btn secondary" :disabled="busy" @click="emit('cancel')">Cancel</button>
        <button
          type="button"
          class="btn primary"
          :disabled="busy || !titleInput.trim()"
          @click="emit('save', titleInput.trim(), colorInput)"
        >
          {{ busy ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { TAB_GROUP_COLORS, tabGroupColorCss, type TabGroupEditOffer } from '$lib/tabGroups';

const props = withDefaults(
  defineProps<{
    offer: TabGroupEditOffer | null;
    busy?: boolean;
  }>(),
  { busy: false },
);

const emit = defineEmits<{
  save: [title: string, color: string];
  cancel: [];
}>();

const titleInput = ref('');
const colorInput = ref('blue');

watch(
  () => props.offer,
  (offer) => {
    if (offer) {
      titleInput.value = offer.title;
      colorInput.value = offer.color.toLowerCase();
    }
  },
  { immediate: true },
);
</script>

<style scoped>
.prompt-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: rgba(0, 0, 0, 0.5);
  border: none;
}

.prompt-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10002;
  width: min(360px, 92vw);
  background: #2d2d2d;
  border: 1px solid #505050;
  border-radius: 12px;
  padding: 20px;
}

.prompt-dialog h3 {
  margin: 0 0 16px;
  font-size: 18px;
  color: #f0f0f0;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 14px;
  font-size: 12px;
  color: #aaa;
}

.field input {
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid #505050;
  background: #1e1e1e;
  color: #e0e0e0;
}

.color-label {
  margin: 0 0 8px;
  font-size: 12px;
  color: #aaa;
}

.color-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 16px;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 2px solid transparent;
  background: var(--swatch);
  cursor: pointer;
  padding: 0;
}

.color-swatch.selected {
  border-color: #fff;
  box-shadow: 0 0 0 2px #6366f1;
}

.prompt-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
}

.btn.secondary {
  background: #404040;
  color: #e0e0e0;
}

.btn.primary {
  background: #6366f1;
  color: #fff;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
