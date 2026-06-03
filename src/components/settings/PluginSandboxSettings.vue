<!--
  Exodus Browser — Plugin Sandbox Settings
-->
<template>
  <section id="settings-section-plugins" class="settings-section plugin-sandbox" data-testid="plugin-sandbox-settings">
    <h3>Plugin Sandbox Configuration</h3>
    <div v-if="!loaded" class="loading-state">Loading…</div>
    <template v-else>
    <p class="hint">
      Configure sandbox isolation for native plugins. Sandbox mode runs each plugin in a separate process with restricted system calls.
    </p>

    <h4>Sandbox Status</h4>
    <div class="sandbox-status" :class="{ enabled: sandboxEnabled }" data-testid="sandbox-status">
      <Shield :size="20" />
      <span>{{ sandboxEnabled ? 'Enabled' : 'Disabled' }}</span>
    </div>

    <h4>Security Settings</h4>
    <label class="checkbox-row">
      <input v-model="sandboxConfig.enableSeccomp" type="checkbox" :disabled="!sandboxEnabled" data-testid="sandbox-seccomp" />
      <span>Enable seccomp (Linux only)</span>
    </label>
    <p class="hint">Restrict system calls using seccomp filters for enhanced security</p>

    <label class="checkbox-row">
      <input v-model="sandboxConfig.allowNetwork" type="checkbox" :disabled="!sandboxEnabled" data-testid="sandbox-network" />
      <span>Allow Network Access</span>
    </label>
    <p class="hint">Allow plugins to make network requests (disabled by default for security)</p>

    <label class="checkbox-row">
      <input v-model="sandboxConfig.allowFilesystem" type="checkbox" :disabled="!sandboxEnabled" data-testid="sandbox-filesystem" />
      <span>Allow Filesystem Access</span>
    </label>
    <p class="hint">Allow plugins to access the filesystem (disabled by default for security)</p>

    <h4>Resource Limits</h4>
    <label class="field">
      Max Memory (MB)
      <input
        v-model.number="sandboxConfig.maxMemoryMb"
        type="number"
        min="64"
        max="4096"
        :disabled="!sandboxEnabled"
        data-testid="sandbox-max-memory"
      />
    </label>
    <p class="hint">Maximum memory per plugin (64-4096 MB, default: 512 MB)</p>

    <div class="btn-row">
      <button
        v-if="sandboxEnabled"
        type="button"
        class="nav-button secondary danger"
        @click="disableSandbox"
        :disabled="loading"
        data-testid="sandbox-disable"
      >
        <Loader2 v-if="loading" class="spinner" :size="16" />
        <span v-if="loading">Disabling...</span>
        <span v-else>Disable Sandbox</span>
      </button>
      <button
        v-else
        type="button"
        class="nav-button"
        @click="enableSandbox"
        :disabled="loading"
        data-testid="sandbox-enable"
      >
        <Loader2 v-if="loading" class="spinner" :size="16" />
        <span v-if="loading">Enabling...</span>
        <span v-else>Enable Sandbox</span>
      </button>
    </div>

    <div v-if="error" class="error-message" data-testid="sandbox-error">
      <AlertCircle :size="16" />
      <span>{{ error }}</span>
    </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Shield, AlertCircle, Loader2 } from '@lucide/vue';
import { nativePluginManager, type SandboxStatus } from '@/lib/nativePlugins';

const loaded = ref(false);
const loading = ref(false);
const sandboxEnabled = ref(false);
const sandboxConfig = ref({
  enableSeccomp: true,
  allowNetwork: false,
  allowFilesystem: false,
  maxMemoryMb: 512,
});
const error = ref<string | null>(null);

const loadSandboxStatus = async () => {
  loading.value = true;
  error.value = null;
  try {
    const status: SandboxStatus = await nativePluginManager.getSandboxStatus();
    sandboxEnabled.value = status.enabled;
    sandboxConfig.value = {
      enableSeccomp: status.config.enableSeccomp,
      allowNetwork: status.config.allowNetwork,
      allowFilesystem: status.config.allowFilesystem,
      maxMemoryMb: status.config.maxMemoryMb,
    };
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load sandbox status';
  } finally {
    loading.value = false;
  }
};

const enableSandbox = async () => {
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.enableSandbox(
      sandboxConfig.value.enableSeccomp,
      sandboxConfig.value.allowNetwork,
      sandboxConfig.value.allowFilesystem,
      sandboxConfig.value.maxMemoryMb
    );
    sandboxEnabled.value = true;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to enable sandbox';
  } finally {
    loading.value = false;
  }
};

const disableSandbox = async () => {
  if (!confirm('Are you sure you want to disable sandbox isolation? This will reduce security.')) return;
  
  loading.value = true;
  error.value = null;
  try {
    await nativePluginManager.disableSandbox();
    sandboxEnabled.value = false;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to disable sandbox';
  } finally {
    loading.value = false;
  }
};

onMounted(async () => {
  await loadSandboxStatus();
  loaded.value = true;
});
</script>

<style scoped>
.settings-section {
  padding: 20px;
}

.settings-section h3 {
  margin: 0 0 16px 0;
  font-size: 20px;
  font-weight: 600;
  color: #111827;
}

.settings-section h4 {
  margin: 24px 0 12px 0;
  font-size: 16px;
  font-weight: 500;
  color: #374151;
}

.loading-state {
  padding: 20px;
  text-align: center;
  color: var(--color-text-secondary, #9ca3af);
}

.hint {
  margin: 0 0 16px 0;
  font-size: 13px;
  color: #6b7280;
  line-height: 1.5;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  cursor: pointer;
}

.checkbox-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.checkbox-row input[type="checkbox"]:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.checkbox-row span {
  font-size: 14px;
  color: #374151;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.field label {
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.field input[type="number"] {
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
}

.field input[type="number"]:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.field input[type="number"]:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.sandbox-status {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #f3f4f6;
  border-radius: 6px;
  font-weight: 500;
  color: #6b7280;
  margin-bottom: 16px;
}

.sandbox-status.enabled {
  background: #d1fae5;
  color: #065f46;
}

.btn-row {
  display: flex;
  gap: 8px;
  margin-top: 24px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-success {
  background: #10b981;
  color: white;
}

.btn-success:hover {
  background: #059669;
}

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover {
  background: #dc2626;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #fee2e2;
  color: #991b1b;
  border-radius: 6px;
  margin-top: 16px;
  font-size: 14px;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
