<template>
  <div v-if="open" class="find-bar" role="search">
    <input
      :value="findQuery"
      type="text"
      class="find-input"
      @input="onInput"
      @keydown.enter="$emit('find', 'next')"
      placeholder="Find in page..."
      aria-label="Find in page"
    />
    <span class="find-count" aria-live="polite">
      {{ findResults > 0 ? `${currentFindIndex}/${findResults}` : '0/0' }}
    </span>
    <button 
      type="button" 
      class="find-btn" 
      @click="$emit('find', 'prev')" 
      title="Previous" 
      aria-label="Previous match"
    >
      ▲
    </button>
    <button 
      type="button" 
      class="find-btn" 
      @click="$emit('find', 'next')" 
      title="Next" 
      aria-label="Next match"
    >
      ▼
    </button>
    <button 
      type="button" 
      class="find-btn close" 
      @click="$emit('close')" 
      title="Close" 
      aria-label="Close find bar"
    >
      ×
    </button>
  </div>
</template>

<script setup lang="ts">

interface Props {
  open?: boolean;
  findQuery?: string;
  findResults?: number;
  currentFindIndex?: number;
}

withDefaults(defineProps<Props>(), {
  open: false,
  findQuery: '',
  findResults: 0,
  currentFindIndex: 0,
});

const emit = defineEmits<{
  findInput: [];
  'update:findQuery': [value: string];
  find: [direction?: 'next' | 'prev'];
  close: [];
}>();

function onInput(e: Event): void {
  const value = (e.target as HTMLInputElement).value;
  emit('update:findQuery', value);
  emit('findInput');
}
</script>

<style scoped>
.find-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--chrome-toolbar-bg, #dee1e6);
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

@media (prefers-color-scheme: dark) {
  .find-bar {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

.find-input {
  flex: 1;
  background: var(--chrome-omnibox-bg, #ffffff);
  border: 1px solid var(--chrome-omnibox-border, #dadce0);
  color: var(--chrome-tab-text-active, #202124);
  padding: 6px 10px;
  border-radius: 16px;
  font-size: 13px;
  min-height: 28px;
}

@media (prefers-color-scheme: dark) {
  .find-input {
    background: #292a2d;
    border-color: #5f6368;
    color: #e8eaed;
  }
}

.find-input:focus {
  outline: none;
  border-color: var(--color-primary, #1a73e8);
  box-shadow: 0 1px 6px rgba(32, 33, 36, 0.28);
}

.find-input::placeholder {
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .find-input::placeholder {
    color: #9aa0a6;
  }
}

.find-count {
  color: var(--chrome-tab-text, #5f6368);
  font-size: 12px;
  min-width: 60px;
  text-align: center;
}

@media (prefers-color-scheme: dark) {
  .find-count {
    color: #9aa0a6;
  }
}

.find-btn {
  background: transparent;
  border: none;
  color: var(--chrome-tab-text, #5f6368);
  width: 28px;
  height: 28px;
  border-radius: 50%;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .find-btn {
    color: #9aa0a6;
  }
}

.find-btn:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .find-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

.find-btn.close {
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .find-btn.close {
    color: #9aa0a6;
  }
}

.find-btn.close:hover {
  background: rgba(234, 67, 53, 0.1);
  color: #ea4335;
}
</style>
