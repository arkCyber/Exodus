<!--
  Exodus Browser — Chrome-style save password dialog.
-->
<template>
  <template v-if="capture">
    <button type="button" class="prompt-backdrop" aria-label="Dismiss" @click="emit('dismiss')" />
    <div class="prompt-dialog" role="dialog" aria-labelledby="pw-save-title">
      <h3 id="pw-save-title">Save password?</h3>
      <p class="prompt-host">{{ host }}</p>
      <label class="field">
        <span>Username</span>
        <input type="text" readonly :value="capture.username || '(none)'" />
      </label>
      <label class="field">
        <span>Password</span>
        <input type="password" readonly :value="capture.password" />
      </label>
      <div class="prompt-actions">
        <button v-if="showNever" type="button" class="btn never" :disabled="busy" @click="emit('never')">
          Never
        </button>
        <button type="button" class="btn secondary" :disabled="busy" @click="emit('dismiss')">Not now</button>
        <button type="button" class="btn primary" :disabled="busy" @click="emit('save')">
          {{ busy ? 'Saving…' : 'Save' }}
        </button>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { PasswordCapturePayload } from '$lib/passwordAutofill';

const props = withDefaults(
  defineProps<{
    capture: PasswordCapturePayload | null;
    busy?: boolean;
    showNever?: boolean;
  }>(),
  { busy: false, showNever: true },
);

const emit = defineEmits<{
  save: [];
  dismiss: [];
  never: [];
}>();

const host = computed(() => {
  if (!props.capture) return '';
  try {
    return new URL(props.capture.url).hostname;
  } catch {
    return props.capture.url;
  }
});
</script>

<style scoped>
.prompt-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: rgba(0, 0, 0, 0.5);
  border: none;
  cursor: default;
}

.prompt-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10002;
  width: min(400px, 92vw);
  background: #2d2d2d;
  border: 1px solid #505050;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.55);
}

.prompt-dialog h3 {
  margin: 0 0 8px;
  font-size: 18px;
  color: #f0f0f0;
}

.prompt-host {
  margin: 0 0 16px;
  font-size: 13px;
  color: #9cdcfe;
  word-break: break-all;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
  font-size: 12px;
  color: #aaa;
}

.field input {
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid #505050;
  background: #1e1e1e;
  color: #e0e0e0;
  font-size: 14px;
}

.prompt-actions {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.btn.never {
  margin-right: auto;
  background: transparent;
  color: #888;
  padding: 8px 4px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 14px;
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
