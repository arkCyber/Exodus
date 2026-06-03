<template>
  <div class="site-isolation-test">
    <h2>Site Isolation Test</h2>
    <div class="test-section">
      <h3>Test Commands</h3>
      <button @click="testGetStats">Get Isolation Stats</button>
      <button @click="testGetOrCreateSite">Get or Create Site</button>
      <button @click="testNavigationAllowed">Test Navigation Allowed</button>
      <button @click="testBoundaryPolicy">Get Boundary Policy</button>
    </div>
    <div class="results">
      <h3>Results</h3>
      <pre>{{ results }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const results = ref<any>(null);

async function testGetStats() {
  try {
    const stats = await invoke('get_isolation_stats');
    results.value = { command: 'get_isolation_stats', data: stats };
  } catch (error) {
    results.value = { command: 'get_isolation_stats', error: String(error) };
  }
}

async function testGetOrCreateSite() {
  try {
    const site = await invoke('get_or_create_site', { url: 'https://example.com' });
    results.value = { command: 'get_or_create_site', data: site };
  } catch (error) {
    results.value = { command: 'get_or_create_site', error: String(error) };
  }
}

async function testNavigationAllowed() {
  try {
    const allowed = await invoke('is_navigation_allowed', { 
      fromUrl: 'https://example.com', 
      toUrl: 'https://google.com' 
    });
    results.value = { command: 'is_navigation_allowed', data: allowed };
  } catch (error) {
    results.value = { command: 'is_navigation_allowed', error: String(error) };
  }
}

async function testBoundaryPolicy() {
  try {
    const policy = await invoke('get_boundary_policy');
    results.value = { command: 'get_boundary_policy', data: policy };
  } catch (error) {
    results.value = { command: 'get_boundary_policy', error: String(error) };
  }
}
</script>

<style scoped>
.site-isolation-test {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
}

.test-section {
  margin: 20px 0;
}

.test-section button {
  margin-right: 10px;
  padding: 8px 16px;
  cursor: pointer;
}

.results {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
