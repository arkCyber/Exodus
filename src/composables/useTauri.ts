import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}

export function useTauri() {
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<T>(command, args);
      return result;
    } catch (e) {
      error.value = e as string;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    loading,
    error,
    invokeCommand,
  };
}
