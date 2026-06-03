<template>
  <div style="padding: 40px; font-size: 24px; color: white; background: #1a1a2e; min-height: 100vh;">
    <h1>Test Page</h1>
    <p>Vue is working!</p>
    <p>ActiveTabId: {{ activeTabId }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const activeTabId = ref('test-123');

onMounted(() => {
  console.log('TestPage mounted successfully');
});
</script>
