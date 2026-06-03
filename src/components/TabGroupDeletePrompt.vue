<!--
  Exodus Browser — confirm tab group deletion.
-->
<template>
  <template v-if="groupTitle !== null">
    <button type="button" class="prompt-backdrop" aria-label="Cancel" @click="emit('cancel')" />
    <div class="prompt-dialog" role="alertdialog" aria-labelledby="tg-del-title">
      <h3 id="tg-del-title">Delete tab group?</h3>
      <p>Delete <strong>{{ groupTitle }}</strong>? Open tabs will stay open.</p>
      <div class="prompt-actions">
        <button type="button" class="btn secondary" :disabled="busy" @click="emit('cancel')">Cancel</button>
        <button type="button" class="btn danger" :disabled="busy" @click="emit('confirm')">
          {{ busy ? 'Deleting…' : 'Delete' }}
        </button>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    groupTitle: string | null;
    busy?: boolean;
  }>(),
  { busy: false },
);

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
  width: min(360px, 92vw);
  background: #2d2d2d;
  border: 1px solid #505050;
  border-radius: 12px;
  padding: 20px;
}

.prompt-dialog h3 {
  margin: 0 0 12px;
  color: #f0f0f0;
}

.prompt-dialog p {
  margin: 0 0 16px;
  color: #ccc;
  font-size: 14px;
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

.btn.danger {
  background: #dc2626;
  color: #fff;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
