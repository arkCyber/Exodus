<!--
  Exodus Browser — reusable confirmation dialog (replaces window.confirm).
-->
<template>
  <template v-if="offer">
    <button type="button" class="prompt-backdrop" aria-label="Cancel" @click="emit('cancel')" />
    <div class="prompt-dialog" role="alertdialog" aria-labelledby="confirm-title">
      <h3 id="confirm-title">{{ offer.title }}</h3>
      <p>{{ offer.message }}</p>
      <div class="prompt-actions">
        <button type="button" class="btn secondary" :disabled="busy" @click="emit('cancel')">
          {{ offer.cancelLabel ?? 'Cancel' }}
        </button>
        <button
          type="button"
          class="btn"
          :class="offer.danger === false ? 'primary' : 'danger'"
          :disabled="busy"
          @click="emit('confirm')"
        >
          {{ busy ? 'Working…' : (offer.confirmLabel ?? 'Confirm') }}
        </button>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import type { ConfirmOffer } from '$lib/confirm';

defineProps<{
  offer: ConfirmOffer | null;
  busy?: boolean;
}>();

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();
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
  width: min(400px, 92vw);
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  animation: modalFadeIn 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .prompt-dialog {
    background: #2d2e30;
    border-color: #5f6368;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
}

@keyframes modalFadeIn {
  from {
    opacity: 0;
    transform: translate(-50%, -48%);
  }
  to {
    opacity: 1;
    transform: translate(-50%, -50%);
  }
}

.prompt-dialog h3 {
  margin: 0 0 12px;
  color: var(--chrome-tab-text-active, #202124);
  font-size: 18px;
  font-weight: 500;
}

@media (prefers-color-scheme: dark) {
  .prompt-dialog h3 {
    color: #e8eaed;
  }
}

.prompt-dialog p {
  margin: 0 0 16px;
  color: var(--chrome-tab-text, #5f6368);
  font-size: 14px;
  line-height: 1.45;
}

@media (prefers-color-scheme: dark) {
  .prompt-dialog p {
    color: #9aa0a6;
  }
}

.prompt-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border-radius: 16px;
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: background-color 0.15s ease;
}

.btn.secondary {
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  border: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .btn.secondary {
    color: #9aa0a6;
    border-color: #5f6368;
  }
}

.btn.secondary:hover {
  background: rgba(0, 0, 0, 0.04);
}

@media (prefers-color-scheme: dark) {
  .btn.secondary:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

.btn.danger {
  background: #d93025;
  color: #fff;
}

.btn.danger:hover {
  background: #b92b20;
}

.btn.primary {
  background: var(--color-primary, #1a73e8);
  color: #fff;
}

.btn.primary:hover {
  background: #1557b0;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
