<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
      @click.stop
    >
      <div
        v-for="item in items"
        :key="item.id"
        :class="['context-menu-item', { disabled: item.disabled, separator: item.separator }]"
        @click="handleItemClick(item)"
      >
        <span v-if="item.icon" class="menu-icon">{{ item.icon }}</span>
        <span class="menu-text">{{ item.label }}</span>
        <span v-if="item.shortcut" class="menu-shortcut">{{ item.shortcut }}</span>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  action?: () => void;
}

interface Props {
  visible?: boolean;
  x?: number;
  y?: number;
  items?: ContextMenuItem[];
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  x: 0,
  y: 0,
  items: () => [],
});

const emit = defineEmits<{
  close: [];
}>();

function handleItemClick(item: ContextMenuItem) {
  if (item.disabled || item.separator) return;
  if (item.action) item.action();
  emit('close');
}

function handleClickOutside(_e: MouseEvent) {
  if (props.visible) {
    emit('close');
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible) {
    emit('close');
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  z-index: 10000;
  min-width: 220px;
  padding: 4px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  animation: fadeIn 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .context-menu {
    background: #2d2e30;
    border-color: #5f6368;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  gap: 12px;
  font-size: 13px;
  color: var(--chrome-tab-text-active, #202124);
  transition: background-color 0.15s ease;
  user-select: none;
}

@media (prefers-color-scheme: dark) {
  .context-menu-item {
    color: #e8eaed;
  }
}

.context-menu-item:hover:not(.disabled):not(.separator) {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .context-menu-item:hover:not(.disabled):not(.separator) {
    background: rgba(255, 255, 255, 0.08);
  }
}

.context-menu-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.context-menu-item.separator {
  padding: 4px 0;
  cursor: default;
}

.context-menu-item.separator::after {
  content: '';
  display: block;
  height: 1px;
  background: var(--chrome-divider, #dadce0);
  width: calc(100% - 24px);
  margin: 0 auto;
}

.menu-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
  color: var(--chrome-tab-text, #5f6368);
}

@media (prefers-color-scheme: dark) {
  .menu-icon {
    color: #9aa0a6;
  }
}

.menu-text {
  flex: 1;
}

.menu-shortcut {
  color: var(--chrome-tab-text, #5f6368);
  font-size: 12px;
}

@media (prefers-color-scheme: dark) {
  .menu-shortcut {
    color: #9aa0a6;
  }
}
</style>
