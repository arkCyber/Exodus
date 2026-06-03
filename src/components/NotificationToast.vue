<template>
  <div class="notification-container">
    <TransitionGroup name="notification">
      <div
        v-for="notification in notifications"
        :key="notification.id"
        :class="['notification', `notification-${notification.type}`]"
        @click="removeNotification(notification.id)"
      >
        <div class="notification-icon">
          <span v-if="notification.type === 'success'">✓</span>
          <span v-else-if="notification.type === 'error'">✕</span>
          <span v-else-if="notification.type === 'warning'">⚠</span>
          <span v-else>ℹ</span>
        </div>
        <div class="notification-content">
          <h4>{{ notification.title }}</h4>
          <p v-if="notification.message">{{ notification.message }}</p>
        </div>
        <button class="notification-close" @click.stop="removeNotification(notification.id)">
          ×
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { useNotifications } from '@/composables/useNotifications';

const { notifications, removeNotification } = useNotifications();
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 400px;
}

.notification {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 8px;
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  transition: all 0.2s ease;
}

@media (prefers-color-scheme: dark) {
  .notification {
    background: #2d2e30;
    border-color: #5f6368;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
}

.notification:hover {
  transform: translateX(-2px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.notification-success {
  border-left: 3px solid #137333;
}

.notification-error {
  border-left: 3px solid #d93025;
}

.notification-warning {
  border-left: 3px solid #b06000;
}

.notification-info {
  border-left: 3px solid var(--color-primary, #1a73e8);
}

.notification-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  font-size: 1rem;
}

.notification-success .notification-icon {
  color: #137333;
}

.notification-error .notification-icon {
  color: #d93025;
}

.notification-warning .notification-icon {
  color: #b06000;
}

.notification-info .notification-icon {
  color: var(--color-primary, #1a73e8);
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-content h4 {
  margin: 0 0 4px 0;
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

@media (prefers-color-scheme: dark) {
  .notification-content h4 {
    color: #e8eaed;
  }
}

.notification-content p {
  margin: 0;
  font-size: 0.8rem;
  color: var(--chrome-tab-text, #5f6368);
  line-height: 1.4;
}

@media (prefers-color-scheme: dark) {
  .notification-content p {
    color: #9aa0a6;
  }
}

.notification-close {
  flex-shrink: 0;
  background: transparent;
  border: none;
  font-size: 1.2rem;
  color: var(--chrome-tab-text, #5f6368);
  cursor: pointer;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: background-color 0.15s ease;
}

@media (prefers-color-scheme: dark) {
  .notification-close {
    color: #9aa0a6;
  }
}

.notification-close:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .notification-close:hover {
    background: rgba(255, 255, 255, 0.1);
  }
}

.notification-enter-active,
.notification-leave-active {
  transition: all 0.2s ease;
}

.notification-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.notification-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
