<!--
  Exodus Browser — GPU and hardware acceleration settings.
-->
<template>
  <section class="settings-section" data-testid="settings-section-gpu">
    <h3>{{ ui.title }}</h3>
    <p class="settings-hint">{{ ui.hint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <!-- GPU Information -->
      <div v-if="gpuInfo" class="gpu-info-card">
        <h4>{{ ui.gpuInfo }}</h4>
        <div class="gpu-details">
          <div class="gpu-detail">
            <span class="label">{{ ui.vendor }}:</span>
            <span class="value">{{ gpuInfo.vendor }}</span>
          </div>
          <div class="gpu-detail">
            <span class="label">{{ ui.renderer }}:</span>
            <span class="value">{{ gpuInfo.renderer }}</span>
          </div>
          <div class="gpu-detail">
            <span class="label">{{ ui.driverVersion }}:</span>
            <span class="value">{{ gpuInfo.driver_version }}</span>
          </div>
          <div class="gpu-detail">
            <span class="label">{{ ui.apiType }}:</span>
            <span class="value">{{ gpuInfo.api_type }}</span>
          </div>
        </div>
      </div>

      <!-- GPU Acceleration -->
      <h4>{{ ui.accelerationSection }}</h4>
      <label class="checkbox-row">
        <input 
          v-model="settings.enabled" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="gpu-acceleration-enabled" 
        />
        <span>{{ ui.gpuAcceleration }}</span>
      </label>
      <p class="settings-hint">{{ ui.gpuAccelerationHint }}</p>

      <!-- WebGL Settings -->
      <h4>{{ ui.webglSection }}</h4>
      <label class="checkbox-row">
        <input 
          v-model="settings.webgl_enabled" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="webgl-enabled" 
        />
        <span>{{ ui.webglEnabled }}</span>
      </label>

      <div v-if="webglSupport" class="webgl-info">
        <div class="info-item">
          <span class="label">{{ ui.webgl1Available }}:</span>
          <span class="value">{{ webglSupport.webgl1_available ? '✓' : '✗' }}</span>
        </div>
        <div class="info-item">
          <span class="label">{{ ui.webgl2Available }}:</span>
          <span class="value">{{ webglSupport.webgl2_available ? '✓' : '✗' }}</span>
        </div>
        <div class="info-item">
          <span class="label">{{ ui.webglVersion }}:</span>
          <span class="value">{{ webglSupport.webgl_version }}</span>
        </div>
      </div>

      <!-- WebGPU Settings -->
      <h4>{{ ui.webgpuSection }}</h4>
      <label class="checkbox-row">
        <input 
          v-model="settings.webgpu_enabled" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="webgpu-enabled" 
        />
        <span>{{ ui.webgpuEnabled }}</span>
      </label>

      <div v-if="webgpuSupport" class="webgpu-info">
        <div class="info-item">
          <span class="label">{{ ui.webgpuAvailable }}:</span>
          <span class="value">{{ webgpuSupport.available ? '✓' : '✗' }}</span>
        </div>
        <div v-if="webgpuSupport.adapter_info" class="info-item">
          <span class="label">{{ ui.adapterInfo }}:</span>
          <span class="value">{{ webgpuSupport.adapter_info }}</span>
        </div>
        <div v-if="webgpuSupport.features.length > 0" class="info-item">
          <span class="label">{{ ui.features }}:</span>
          <span class="value">{{ webgpuSupport.features.join(', ') }}</span>
        </div>
      </div>

      <!-- ANGLE Backend -->
      <h4>{{ ui.angleSection }}</h4>
      <label>
        {{ ui.angleBackend }}
        <select 
          v-model="settings.angle_backend" 
          data-testid="angle-backend" 
          @change="() => void persist()"
        >
          <option value="default">{{ ui.angleDefault }}</option>
          <option value="gl">{{ ui.angleGl }}</option>
          <option value="d3d11">{{ ui.angleD3d11 }}</option>
          <option value="d3d9">{{ ui.angleD3d9 }}</option>
          <option value="metal">{{ ui.angleMetal }}</option>
          <option value="vulkan">{{ ui.angleVulkan }}</option>
        </select>
      </label>
      <p class="settings-hint">{{ ui.angleBackendHint }}</p>

      <!-- Advanced Settings -->
      <h4>{{ ui.advancedSection }}</h4>
      <label class="checkbox-row">
        <input 
          v-model="settings.gpu_rasterization" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="gpu-rasterization" 
        />
        <span>{{ ui.gpuRasterization }}</span>
      </label>

      <label class="checkbox-row">
        <input 
          v-model="settings.zero_copy_video" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="zero-copy-video" 
        />
        <span>{{ ui.zeroCopyVideo }}</span>
      </label>

      <label class="checkbox-row">
        <input 
          v-model="settings.ignore_gpu_blocklist" 
          type="checkbox" 
          @change="() => void persist()" 
          data-testid="ignore-gpu-blocklist" 
        />
        <span>{{ ui.ignoreGpuBlocklist }}</span>
      </label>
      <p class="settings-hint warning">{{ ui.ignoreGpuBlocklistWarning }}</p>

      <!-- Performance Metrics -->
      <h4>{{ ui.performanceSection }}</h4>
      <div v-if="performanceMetrics" class="performance-metrics">
        <div class="metric-item">
          <span class="label">{{ ui.memoryUsed }}:</span>
          <span class="value">{{ formatBytes(performanceMetrics.memory_used) }}</span>
        </div>
        <div class="metric-item">
          <span class="label">{{ ui.memoryTotal }}:</span>
          <span class="value">{{ formatBytes(performanceMetrics.memory_total) }}</span>
        </div>
        <div class="metric-item">
          <span class="label">{{ ui.gpuUtilization }}:</span>
          <span class="value">{{ (performanceMetrics.gpu_utilization * 100).toFixed(1) }}%</span>
        </div>
        <div v-if="performanceMetrics.temperature" class="metric-item">
          <span class="label">{{ ui.temperature }}:</span>
          <span class="value">{{ performanceMetrics.temperature.toFixed(1) }}°C</span>
        </div>
      </div>

      <button 
        type="button" 
        class="nav-button secondary" 
        @click="() => void refreshMetrics()" 
        data-testid="refresh-metrics"
      >
        {{ ui.refreshMetrics }}
      </button>

      <button 
        type="button" 
        class="nav-button secondary" 
        @click="() => void resetToDefaults()" 
        data-testid="gpu-reset"
      >
        {{ ui.reset }}
      </button>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — GPU and hardware acceleration settings.
 */
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { type AppLocale } from '@/lib/appLocale';
import { gpuSettingsStrings } from '@/lib/gpuSettingsUi';
import { runGpuDetection } from '@/lib/gpuDetection';

const props = defineProps<{
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
}>();

const ui = computed(() => gpuSettingsStrings(props.uiLocale));

interface GpuInfo {
  vendor: string;
  renderer: string;
  driver_version: string;
  api_type: string;
  max_texture_size: number;
  max_viewport_dims: number[];
}

interface WebGLSupport {
  webgl1_available: boolean;
  webgl2_available: boolean;
  webgl_version: string;
  max_texture_size: number;
  max_renderbuffer_size: number;
}

interface WebGPUSupport {
  available: boolean;
  adapter_info: string | null;
  features: string[];
}

interface GpuAccelerationSettings {
  enabled: boolean;
  webgl_enabled: boolean;
  webgpu_enabled: boolean;
  angle_backend: string;
  gpu_rasterization: boolean;
  zero_copy_video: boolean;
  ignore_gpu_blocklist: boolean;
}

interface GpuPerformanceMetrics {
  timestamp: number;
  memory_used: number;
  memory_total: number;
  gpu_utilization: number;
  temperature: number | null;
  power_usage: number | null;
}

const STORAGE_KEY = 'exodus-gpu-settings';

const DEFAULT_SETTINGS: GpuAccelerationSettings = {
  enabled: true,
  webgl_enabled: true,
  webgpu_enabled: true,
  angle_backend: 'default',
  gpu_rasterization: true,
  zero_copy_video: true,
  ignore_gpu_blocklist: false,
};

const loading = ref(true);
const settings = ref<GpuAccelerationSettings>({ ...DEFAULT_SETTINGS });
const gpuInfo = ref<GpuInfo | null>(null);
const webglSupport = ref<WebGLSupport | null>(null);
const webgpuSupport = ref<WebGPUSupport | null>(null);
const performanceMetrics = ref<GpuPerformanceMetrics | null>(null);

/** Load settings from localStorage and backend. */
async function load(): Promise<void> {
  loading.value = true;
  try {
    // Load from localStorage
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      settings.value = { ...DEFAULT_SETTINGS, ...JSON.parse(saved) };
    }

    // Initialize GPU detection
    await invoke('initialize_gpu_detection');

    // Run JavaScript-based GPU detection for more accurate results
    try {
      await runGpuDetection();
    } catch (error) {
      console.error('Failed to run JS GPU detection:', error);
    }

    // Get GPU info from backend
    try {
      gpuInfo.value = await invoke<GpuInfo | null>('get_gpu_info');
    } catch (error) {
      console.error('Failed to get GPU info:', error);
    }

    // Get WebGL support
    try {
      webglSupport.value = await invoke<WebGLSupport | null>('get_webgl_support');
    } catch (error) {
      console.error('Failed to get WebGL support:', error);
    }

    // Get WebGPU support
    try {
      webgpuSupport.value = await invoke<WebGPUSupport | null>('get_webgpu_support');
    } catch (error) {
      console.error('Failed to get WebGPU support:', error);
    }

    // Get current settings from backend
    try {
      const backendSettings = await invoke<GpuAccelerationSettings>('get_gpu_acceleration_settings');
      settings.value = { ...settings.value, ...backendSettings };
    } catch (error) {
      console.error('Failed to get GPU settings from backend:', error);
    }

    // Get performance metrics
    await refreshMetrics();
  } catch (error) {
    console.error('GpuSettings.load failed:', error);
  } finally {
    loading.value = false;
  }
}

/** Persist settings to localStorage and backend. */
async function persist(): Promise<void> {
  try {
    // Validate settings before applying
    try {
      await invoke('validate_gpu_settings', { settings: settings.value });
    } catch (validationError) {
      console.error('GPU settings validation failed:', validationError);
      emit('status', `Validation error: ${validationError}`);
      return;
    }

    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value));

    // Sync with backend
    if (settings.value.enabled) {
      await invoke('enable_gpu_acceleration');
    } else {
      await invoke('disable_gpu_acceleration');
    }

    await invoke('set_webgl_enabled', { enabled: settings.value.webgl_enabled });
    await invoke('set_webgpu_enabled', { enabled: settings.value.webgpu_enabled });
    await invoke('set_angle_backend', { backend: settings.value.angle_backend });

    // Apply settings to WebView (hot reload)
    try {
      await invoke('apply_gpu_settings_to_webview');
      emit('status', ui.value.saved + ' (WebView updated)');
    } catch (applyError) {
      console.error('Failed to apply GPU settings to WebView:', applyError);
      emit('status', ui.value.saved + ' (WebView update pending restart)');
    }
  } catch (error) {
    console.error('GpuSettings.persist failed:', error);
    emit('status', ui.value.saveError);
  }
}

/** Refresh performance metrics. */
async function refreshMetrics(): Promise<void> {
  try {
    performanceMetrics.value = await invoke<GpuPerformanceMetrics>('get_gpu_performance_metrics');
  } catch (error) {
    console.error('Failed to refresh metrics:', error);
  }
}

/** Reset to default settings. */
async function resetToDefaults(): Promise<void> {
  settings.value = { ...DEFAULT_SETTINGS };
  await persist();
  emit('status', ui.value.reset);
}

/** Format bytes to human readable format. */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
}

onMounted(() => {
  load();
});
</script>

<style scoped>
.gpu-info-card {
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}

.gpu-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.gpu-detail {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.webgl-info,
.webgpu-info {
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.performance-metrics {
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.metric-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.label {
  font-weight: 500;
  color: var(--text-secondary);
}

.value {
  font-family: monospace;
  color: var(--text-primary);
}

.warning {
  color: var(--color-warning);
  font-size: 0.9em;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.settings-hint {
  color: var(--text-secondary);
  font-size: 0.9em;
  margin-bottom: 16px;
}

.loading-state {
  text-align: center;
  padding: 32px;
  color: var(--text-secondary);
}
</style>
