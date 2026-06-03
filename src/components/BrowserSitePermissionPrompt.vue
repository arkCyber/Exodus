<!--
  Exodus Browser — per-origin site permission (camera, microphone, geolocation).
-->
<template>
  <div v-if="request" class="perm-backdrop" role="presentation">
    <div class="perm-dialog" role="dialog" aria-labelledby="site-perm-title">
      <h3 id="site-perm-title">Site permission</h3>
      <p>
        <strong>{{ request.origin }}</strong> wants to {{ kindLabel(request.kind) }}.
      </p>
      <div class="perm-actions">
        <button type="button" class="btn secondary" :disabled="busy" @click="() => void answer(false)">
          Block
        </button>
        <button type="button" class="btn primary" :disabled="busy" @click="() => void answer(true)">
          Allow
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — modal prompt for browser site permissions from web content.
 */
import { ref } from 'vue';
import type { BrowserSitePermissionRequestEvent } from '$lib/extensions/extensionEvents';
import { resolveBrowserSitePermission } from '$lib/extensions/api';

const props = defineProps<{
  request: BrowserSitePermissionRequestEvent | null;
}>();

const emit = defineEmits<{
  resolved: [];
}>();

const busy = ref(false);

/** Human-readable label for permission kind from the bridge. */
function kindLabel(kind: string): string {
  switch (kind.toLowerCase()) {
    case 'camera':
      return 'use your camera';
    case 'microphone':
    case 'mic':
      return 'use your microphone';
    case 'geolocation':
    case 'location':
      return 'know your location';
    case 'notifications':
      return 'show notifications';
    default:
      return `use ${kind}`;
  }
}

/** Grant or deny the pending browser site permission. */
async function answer(granted: boolean): Promise<void> {
  if (!props.request || busy.value) return;
  busy.value = true;
  try {
    await resolveBrowserSitePermission(props.request.requestId, granted);
  } catch (error) {
    console.error('browser_site_permission_resolve failed:', error);
  } finally {
    busy.value = false;
    emit('resolved');
  }
}
</script>

<style scoped>
.perm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10002;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.5);
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

.perm-dialog {
  max-width: 420px;
  padding: 20px 24px;
  border-radius: 8px;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  color: var(--chrome-tab-text-active, #202124);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  animation: modalSlideIn 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .perm-dialog {
    background: #2d2e30;
    border-color: #5f6368;
    color: #e8eaed;
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

.perm-dialog h3 {
  margin: 0 0 12px;
  font-size: 18px;
  font-weight: 500;
}

.perm-dialog p {
  margin: 0 0 16px;
  font-size: 14px;
  line-height: 1.45;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .perm-dialog p {
    color: #9aa0a6;
  }
}

.perm-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 16px;
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

.btn.primary {
  background: var(--color-primary, #1a73e8);
  color: #fff;
}

.btn.primary:hover {
  background: #1557b0;
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

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
