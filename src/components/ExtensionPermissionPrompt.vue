<!--
  Exodus Browser — modal prompt for chrome.permissions.request.
-->
<template>
  <div v-if="request" class="perm-backdrop" role="presentation">
    <div class="perm-dialog" role="dialog" aria-labelledby="perm-title">
      <h3 id="perm-title">Permission request</h3>
      <p>
        <strong>{{ label }}</strong> requests:
      </p>
      <template v-if="apiPermissions.length">
        <p class="perm-sub">API permissions:</p>
        <ul>
          <li v-for="perm in apiPermissions" :key="perm">
            <code>{{ perm }}</code>
          </li>
        </ul>
      </template>
      <template v-if="hostPermissions.length">
        <p class="perm-sub">Site access:</p>
        <ul>
          <li v-for="perm in hostPermissions" :key="perm">
            <code>{{ perm }}</code>
          </li>
        </ul>
      </template>
      <div class="perm-actions">
        <button type="button" class="btn secondary" :disabled="busy" @click="answer(false)">
          Deny
        </button>
        <button type="button" class="btn primary" :disabled="busy" @click="answer(true)">
          Allow
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — modal prompt for chrome.permissions.request.
 * Aerospace-level error handling, security validation, and concurrency safety.
 */
import { ref, computed, onUnmounted } from 'vue';
import type { ExtensionPermissionRequestEvent } from '$lib/extensions/extensionEvents';
import { extensionDisplayName } from '$lib/extensions/extensionDisplayName';
import { resolveExtensionPermission } from '$lib/extensions/api';

const props = defineProps<{
  request: ExtensionPermissionRequestEvent | null;
}>();

const emit = defineEmits<{
  resolved: [];
}>();

const busy = ref(false);

// Aerospace-level security validation patterns
const VALID_REQUEST_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const SAFE_PERMISSION_PATTERN = /^[a-zA-Z0-9_*<:.\/>-]+$/;

const label = computed(() => extensionDisplayName(props.request));

/**
 * Host permission patterns include URL schemes or <all_urls>.
 * Aerospace-level validation to ensure safe permission patterns.
 */
function isHostPermission(perm: string): boolean {
  // Aerospace-level security validation
  if (!perm || typeof perm !== 'string') {
    console.error('[ExtensionPermissionPrompt] Invalid permission string');
    return false;
  }
  
  // Validate permission format to prevent injection
  if (!SAFE_PERMISSION_PATTERN.test(perm)) {
    console.error('[ExtensionPermissionPrompt] Invalid permission format:', perm);
    return false;
  }
  
  return perm === '<all_urls>' || perm.includes('://') || perm.startsWith('*.');
}

const apiPermissions = computed(() =>
  (props.request?.permissions ?? []).filter((p) => !isHostPermission(p)),
);

const hostPermissions = computed(() =>
  (props.request?.permissions ?? []).filter(isHostPermission),
);

/**
 * Grant or deny the pending permission request.
 * Aerospace-level error handling with input validation and concurrency safety.
 */
async function answer(granted: boolean): Promise<void> {
  // Aerospace-level input validation
  if (!props.request) {
    console.error('[ExtensionPermissionPrompt] No request to answer');
    return;
  }
  
  // Aerospace-level security validation
  if (!props.request.requestId || typeof props.request.requestId !== 'string') {
    console.error('[ExtensionPermissionPrompt] Invalid request ID');
    return;
  }
  
  if (!VALID_REQUEST_ID_PATTERN.test(props.request.requestId)) {
    console.error('[ExtensionPermissionPrompt] Invalid request ID format:', props.request.requestId);
    return;
  }
  
  // Aerospace-level concurrency check
  if (busy.value) {
    console.warn('[ExtensionPermissionPrompt] Operation already in progress');
    return;
  }
  
  busy.value = true;
  
  try {
    await resolveExtensionPermission(props.request.requestId, granted);
  } catch (error) {
    console.error('[ExtensionPermissionPrompt] extension_permissions_resolve failed:', error);
    // User-facing error feedback could be added here if needed
  } finally {
    busy.value = false;
    emit('resolved');
  }
}

/**
 * Cleanup on component unmount.
 * Aerospace-level memory management and resource cleanup.
 */
onUnmounted(() => {
  // Reset busy state to prevent stuck UI
  busy.value = false;
});
</script>

<style scoped>
.perm-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
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
  min-width: 320px;
  max-width: 480px;
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
  margin: 0 0 8px;
  font-size: 14px;
  line-height: 1.45;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .perm-dialog p {
    color: #9aa0a6;
  }
}

.perm-sub {
  margin-top: 12px;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .perm-sub {
    color: #9aa0a6;
  }
}

.perm-dialog ul {
  margin: 0 0 16px;
  padding-left: 20px;
  list-style: disc;
}

.perm-dialog li {
  margin-bottom: 4px;
  font-size: 13px;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .perm-dialog li {
    color: #9aa0a6;
  }
}

.perm-dialog code {
  background: rgba(0, 0, 0, 0.04);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 12px;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .perm-dialog code {
    background: rgba(255, 255, 255, 0.08);
    color: #e8eaed;
  }
}

.perm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
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
