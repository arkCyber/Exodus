<template>
  <div class="home-page">
    <h1>Exodus Browser</h1>
    <p>Vue 3 + Tauri migration in progress...</p>
    <p>Status: {{ status }}</p>
    <button @click="testTauri">Test Tauri</button>
    <p v-if="tauriResult">Tauri Result: {{ tauriResult }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';

const status = ref('Ready');
const tauriResult = ref<string | null>(null);

async function testTauri() {
  if (!isTauri()) {
    status.value = 'Not in Tauri environment';
    tauriResult.value = 'Dev mode - Tauri API unavailable';
    return;
  }
  status.value = 'Testing Tauri...';
  try {
    // Try to invoke a simple command
    const result = await invoke('get_app_name');
    tauriResult.value = result as string;
    status.value = 'Success';
  } catch (e) {
    tauriResult.value = `Error: ${e}`;
    status.value = 'Error';
  }
}
</script>

<style scoped>
.home-page {
  padding: 20px;
}
</style>
