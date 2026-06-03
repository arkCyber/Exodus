/**
 * Exodus Browser — GPU detection utilities for WebGL and WebGPU.
 * 
 * This module provides functions to detect actual WebGL and WebGPU capabilities
 * from within the browser context, which can then be sent to the backend.
 */

/**
 * Detect WebGL capabilities from the browser context.
 */
export function detectWebGLCapabilities(): {
  webgl1_available: boolean;
  webgl2_available: boolean;
  webgl_version: string;
  max_texture_size: number;
  max_renderbuffer_size: number;
} {
  const result = {
    webgl1_available: false,
    webgl2_available: false,
    webgl_version: '0.0',
    max_texture_size: 0,
    max_renderbuffer_size: 0,
  };

  // Try WebGL 2 first
  try {
    const canvas = document.createElement('canvas');
    const gl2 = canvas.getContext('webgl2');
    if (gl2) {
      result.webgl2_available = true;
      result.webgl_version = '2.0';
      result.max_texture_size = gl2.getParameter(gl2.MAX_TEXTURE_SIZE);
      result.max_renderbuffer_size = gl2.getParameter(gl2.MAX_RENDERBUFFER_SIZE);
    }
  } catch (e) {
    // WebGL 2 not available
  }

  // Try WebGL 1
  try {
    const canvas = document.createElement('canvas');
    const gl1 = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
    if (gl1) {
      result.webgl1_available = true;
      if (!result.webgl2_available) {
        result.webgl_version = '1.0';
        result.max_texture_size = gl1.getParameter(gl1.MAX_TEXTURE_SIZE);
        result.max_renderbuffer_size = gl1.getParameter(gl1.MAX_RENDERBUFFER_SIZE);
      }
    }
  } catch (e) {
    // WebGL 1 not available
  }

  return result;
}

/**
 * Detect WebGPU capabilities from the browser context.
 */
export async function detectWebGPUCapabilities(): Promise<{
  available: boolean;
  adapter_info: string | null;
  features: string[];
}> {
  const result = {
    available: false,
    adapter_info: null as string | null,
    features: [] as string[],
  };

  try {
    if (!navigator.gpu) {
      return result;
    }

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      return result;
    }

    result.available = true;
    result.adapter_info = adapter.info?.description || 'Unknown adapter';

    // Get supported features
    const features: string[] = [];
    if (adapter.features.has('timestamp-query')) {
      features.push('timestamp-query');
    }
    if (adapter.features.has('pipeline-statistics-query')) {
      features.push('pipeline-statistics-query');
    }
    if (adapter.features.has('texture-compression-bc')) {
      features.push('texture-compression-bc');
    }
    if (adapter.features.has('texture-compression-etc2')) {
      features.push('texture-compression-etc2');
    }
    if (adapter.features.has('texture-compression-astc')) {
      features.push('texture-compression-astc');
    }
    result.features = features;
  } catch (e) {
    // WebGPU not available or error occurred
    console.error('WebGPU detection failed:', e);
  }

  return result;
}

/**
 * Estimate GPU performance metrics.
 * Note: This is a rough estimation as browsers don't provide direct GPU metrics.
 */
export function estimateGpuMetrics(): {
  timestamp: number;
  memory_used: number;
  memory_total: number;
  gpu_utilization: number;
  temperature: number | null;
  power_usage: number | null;
} {
  // This is a placeholder implementation
  // Real GPU metrics would require platform-specific APIs or extensions
  return {
    timestamp: Date.now(),
    memory_used: 0,
    memory_total: 0,
    gpu_utilization: 0,
    temperature: null,
    power_usage: null,
  };
}

/**
 * Run all GPU detections and send results to backend.
 */
export async function runGpuDetection(): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');

  // Detect WebGL
  const webglCapabilities = detectWebGLCapabilities();
  try {
    await invoke('detect_webgl_from_js', { webglInfo: webglCapabilities });
  } catch (e) {
    console.error('Failed to send WebGL detection:', e);
  }

  // Detect WebGPU
  const webgpuCapabilities = await detectWebGPUCapabilities();
  try {
    await invoke('detect_webgpu_from_js', { webgpuInfo: webgpuCapabilities });
  } catch (e) {
    console.error('Failed to send WebGPU detection:', e);
  }

  // Estimate GPU metrics
  const gpuMetrics = estimateGpuMetrics();
  try {
    await invoke('collect_performance_metrics_from_js', { metrics: gpuMetrics });
  } catch (e) {
    console.error('Failed to send GPU metrics:', e);
  }
}
