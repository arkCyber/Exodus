<template>
  <div v-if="error" class="error-boundary">
    <div class="error-content">
      <h2>Something went wrong</h2>
      <p>{{ error.message }}</p>
      <div v-if="error.stack" class="error-stack">
        <pre>{{ error.stack }}</pre>
      </div>
      <button @click="resetError" class="retry-button">Retry</button>
    </div>
  </div>
  <slot v-else />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue';

const error = ref<Error | null>(null);

onErrorCaptured((err: Error) => {
  error.value = err;
  console.error('ErrorBoundary caught:', err);
  // Return false to prevent error from propagating further
  return false;
});

function resetError() {
  error.value = null;
}
</script>

<style scoped>
.error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: #1a1a1a;
  color: #ffffff;
  padding: 2rem;
}

.error-content {
  max-width: 600px;
  text-align: center;
}

.error-content h2 {
  font-size: 1.5rem;
  margin-bottom: 1rem;
  color: #ff6b6b;
}

.error-content p {
  margin-bottom: 1.5rem;
  color: #cccccc;
}

.error-stack {
  background: #2a2a2a;
  border-radius: 8px;
  padding: 1rem;
  margin-bottom: 1.5rem;
  text-align: left;
  overflow-x: auto;
}

.error-stack pre {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 0.85rem;
  color: #ff6b6b;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.retry-button {
  background: #007aff;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  font-size: 1rem;
  cursor: pointer;
  transition: background 0.2s;
}

.retry-button:hover {
  background: #0056b3;
}
</style>
