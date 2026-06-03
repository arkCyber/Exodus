import { ref } from 'vue';

export type NotificationType = 'success' | 'error' | 'warning' | 'info';

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
  timestamp: number;
}

const notifications = ref<Notification[]>([]);

export function useNotifications() {
  function addNotification(
    type: NotificationType,
    title: string,
    message?: string,
    duration: number = 3000
  ) {
    const notification: Notification = {
      id: `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      title,
      message,
      duration,
      timestamp: Date.now(),
    };

    notifications.value.push(notification);

    // Auto-remove after duration
    if (duration > 0) {
      setTimeout(() => {
        removeNotification(notification.id);
      }, duration);
    }

    return notification.id;
  }

  function removeNotification(id: string) {
    const index = notifications.value.findIndex(n => n.id === id);
    if (index > -1) {
      notifications.value.splice(index, 1);
    }
  }

  function clearNotifications() {
    notifications.value = [];
  }

  function success(title: string, message?: string, duration?: number) {
    return addNotification('success', title, message, duration);
  }

  function error(title: string, message?: string, duration?: number) {
    return addNotification('error', title, message, duration);
  }

  function warning(title: string, message?: string, duration?: number) {
    return addNotification('warning', title, message, duration);
  }

  function info(title: string, message?: string, duration?: number) {
    return addNotification('info', title, message, duration);
  }

  return {
    notifications,
    addNotification,
    removeNotification,
    clearNotifications,
    success,
    error,
    warning,
    info,
  };
}
