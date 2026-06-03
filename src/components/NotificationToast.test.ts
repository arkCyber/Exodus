/**
 * Exodus Browser — NotificationToast component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ref } from 'vue';
import { mount } from '@vue/test-utils';
import NotificationToast from './NotificationToast.vue';

const removeNotificationMock = vi.fn();
const notificationsMock = ref([
  { id: '1', type: 'success' as const, title: 'Success', message: 'Operation completed', timestamp: 0 },
  { id: '2', type: 'error' as const, title: 'Error', message: 'Something went wrong', timestamp: 0 },
  { id: '3', type: 'warning' as const, title: 'Warning', message: 'Please check', timestamp: 0 },
  { id: '4', type: 'info' as const, title: 'Info', message: 'Information', timestamp: 0 },
]);

vi.mock('@/composables/useNotifications', () => ({
  useNotifications: () => ({
    notifications: notificationsMock,
    removeNotification: removeNotificationMock,
  }),
}));

describe('NotificationToast', () => {
  beforeEach(() => {
    removeNotificationMock.mockClear();
  });

  it('renders notification container', () => {
    const wrapper = mount(NotificationToast);
    expect(wrapper.find('.notification-container').exists()).toBe(true);
  });

  it('renders all notification types', () => {
    const wrapper = mount(NotificationToast);
    const notifications = wrapper.findAll('.notification');
    expect(notifications.length).toBe(4);
  });

  it('renders success notification with correct class', () => {
    const wrapper = mount(NotificationToast);
    const successNotification = wrapper.findAll('.notification')[0];
    expect(successNotification.classes()).toContain('notification-success');
  });

  it('renders error notification with correct class', () => {
    const wrapper = mount(NotificationToast);
    const errorNotification = wrapper.findAll('.notification')[1];
    expect(errorNotification.classes()).toContain('notification-error');
  });

  it('renders warning notification with correct class', () => {
    const wrapper = mount(NotificationToast);
    const warningNotification = wrapper.findAll('.notification')[2];
    expect(warningNotification.classes()).toContain('notification-warning');
  });

  it('renders info notification with correct class', () => {
    const wrapper = mount(NotificationToast);
    const infoNotification = wrapper.findAll('.notification')[3];
    expect(infoNotification.classes()).toContain('notification-info');
  });

  it('displays notification title', () => {
    const wrapper = mount(NotificationToast);
    const titles = wrapper.findAll('.notification-content h4');
    expect(titles[0].text()).toBe('Success');
    expect(titles[1].text()).toBe('Error');
  });

  it('displays notification message', () => {
    const wrapper = mount(NotificationToast);
    const messages = wrapper.findAll('.notification-content p');
    expect(messages[0].text()).toBe('Operation completed');
  });

  it('renders notification icons', () => {
    const wrapper = mount(NotificationToast);
    const icons = wrapper.findAll('.notification-icon span');
    expect(icons[0].text()).toBe('✓');
    expect(icons[1].text()).toBe('✕');
    expect(icons[2].text()).toBe('⚠');
    expect(icons[3].text()).toBe('ℹ');
  });

  it('renders close button', () => {
    const wrapper = mount(NotificationToast);
    const closeButtons = wrapper.findAll('.notification-close');
    expect(closeButtons.length).toBe(4);
  });

  it('calls removeNotification when notification is clicked', async () => {
    const wrapper = mount(NotificationToast);
    const notification = wrapper.find('.notification');
    await notification.trigger('click');

    expect(removeNotificationMock).toHaveBeenCalledWith('1');
  });

  it('calls removeNotification when close button is clicked', async () => {
    const wrapper = mount(NotificationToast);
    const closeButton = wrapper.find('.notification-close');
    await closeButton.trigger('click');

    expect(removeNotificationMock).toHaveBeenCalledWith('1');
  });
});
