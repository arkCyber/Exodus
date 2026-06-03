<!--
  Exodus Browser — modal shell for downloads and overlay panels.
-->
<template>
  <div
    v-if="open"
    class="panel-modal exodus-panel-modal"
    role="dialog"
    aria-modal="true"
    :aria-labelledby="titleId"
    tabindex="-1"
    @click.self="emit('close')"
    @keydown.escape="emit('close')"
  >
    <div>
      <div class="panel-header">
        <h2 :id="titleId">{{ title }}</h2>
        <button type="button" class="close-btn" aria-label="Close" @click="emit('close')">×</button>
      </div>
      <div class="panel-content">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  open: boolean;
  title: string;
}>();

const emit = defineEmits<{ close: [] }>();

const titleId = computed(() => `panel-title-${props.title.replace(/\s+/g, '-').toLowerCase()}`);
</script>

<style scoped>
.panel-modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: backdropFadeIn 0.2s ease;
}

@keyframes backdropFadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.panel-modal > div {
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  animation: modalSlideIn 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .panel-modal > div {
    background: #2d2e30;
    border-color: #5f6368;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .panel-header {
    border-color: #5f6368;
  }
}

.panel-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .panel-header h2 {
    color: #e8eaed;
  }
}

.panel-content {
  padding: 20px;
  overflow-y: auto;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 24px;
  cursor: pointer;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .close-btn {
    color: #9aa0a6;
  }
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}
</style>
