<!--
  Exodus Browser — Safe Browsing warning before proceeding to a risky URL.
-->
<template>
  <template v-if="offer">
    <button type="button" class="prompt-backdrop" aria-label="Close warning" @click="emit('cancel')" />
    <div class="prompt-dialog" role="alertdialog" aria-labelledby="sb-title">
      <h3 id="sb-title">Security warning</h3>
      <p class="reason">{{ offer.reason }}</p>
      <p class="url">{{ offer.url }}</p>
      <div class="prompt-actions">
        <button type="button" class="btn secondary" @click="emit('cancel')">Go back</button>
        <button type="button" class="btn danger" @click="emit('proceed')">Proceed anyway</button>
      </div>
    </div>
  </template>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — modal when Safe Browsing flags a URL but allows proceed-after-warning.
 */
import type { SafeBrowsingOffer } from '@/composables/useSafeBrowsingNavigation';

defineProps<{
  offer: SafeBrowsingOffer | null;
}>();

const emit = defineEmits<{
  proceed: [];
  cancel: [];
}>();
</script>

<style scoped>
.prompt-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: rgba(0, 0, 0, 0.55);
  border: none;
  cursor: default;
}

.prompt-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10002;
  width: min(440px, 92vw);
  background: #2d2d2d;
  border: 1px solid #b45309;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.55);
}

.prompt-dialog h3 {
  margin: 0 0 12px;
  color: #fbbf24;
  font-size: 18px;
}

.reason {
  margin: 0 0 8px;
  color: #e0e0e0;
  font-size: 14px;
  line-height: 1.4;
}

.url {
  margin: 0 0 16px;
  font-size: 12px;
  color: #9ca3af;
  word-break: break-all;
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
  font-size: 14px;
}

.btn.secondary {
  background: #404040;
  color: #e0e0e0;
}

.btn.danger {
  background: #b45309;
  color: #fff;
}
</style>
