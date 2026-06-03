<template>
  <div class="loading-spinner" :class="size">
    <div class="spinner"></div>
    <p v-if="message" class="loading-message">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  size?: 'small' | 'medium' | 'large';
  message?: string;
}

withDefaults(defineProps<Props>(), {
  size: 'medium',
  message: '',
});
</script>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.spinner {
  border: 3px solid var(--chrome-divider, #dadce0);
  border-top-color: var(--color-primary, #1a73e8);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@media (prefers-color-scheme: dark) {
  .spinner {
    border-color: #5f6368;
    border-top-color: var(--color-primary, #8ab4f8);
  }
}

.loading-spinner.small .spinner {
  width: 20px;
  height: 20px;
  border-width: 2px;
}

.loading-spinner.medium .spinner {
  width: 40px;
  height: 40px;
  border-width: 3px;
}

.loading-spinner.large .spinner {
  width: 60px;
  height: 60px;
  border-width: 4px;
}

.loading-message {
  color: var(--chrome-tab-text, #5f6368);
  font-size: 0.9rem;
  margin: 0;
}

@media (prefers-color-scheme: dark) {
  .loading-message {
    color: #9aa0a6;
  }
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
