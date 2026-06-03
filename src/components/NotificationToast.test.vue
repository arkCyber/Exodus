/**
 * Exodus Browser — NotificationToast component tests.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount } from '@vue/test-utils';
import NotificationToast from './NotificationToast.vue';

vi.mock('@/composables/useNotifications', () => ({
  useNotifications: () => ({
    notifications: [
      { id: '1', type: 'success', title: 'Success', message: 'Operation completed' },
      { id: '2', type: 'error', title: 'Error', message: 'Something went wrong' },
      { id: '3', type: 'warning', title: 'Warning', message: 'Be careful' },
      { id: '4', type: 'info', title: 'Info', message: 'Information' },
    ],
    removeNotification: vi.fn(),
  }),
}));

describe('NotificationToast', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders notification container', () => {
    const wrapper = mount(NotificationToast);
    
    expect(wrapper.find('.notification-container').exists()).toBe(true);
  });

  it('renders all notification types', () => {
    const wrapper = mount(NotificationToast);
    
    expect(wrapper.findAll('.notification').length).toBe(4);
    expect(wrapper.find('.notification-success').exists()).toBe(true);
    expect(wrapper.find('.notification-error').exists()).toBe(true);
    expect(wrapper.find('.notification-warning').exists()).toBe(true);
    expect(wrapper.find('.notification-info').exists()).toBe(true);
  });

  it('displays correct icon for success notification', () => {
    const wrapper = mount(NotificationToast);
    
    const successNotification = wrapper.find('.notification-success');
    expect(successNotification.find('.notification-icon').text()).toBe('✓');
  });

  it('displays correct icon for error notification', () => {
    const wrapper = mount(NotificationToast);
    
    const errorNotification = wrapper.find('.notification-error');
    expect(errorNotification.find('.notification-icon').text()).toBe('✕');
  });

  it('displays correct icon for warning notification', () => {
    const wrapper = mount(NotificationToast);
    
    const warningNotification = wrapper.find('.notification-warning');
    expect(warningNotification.find('.notification-icon').text()).toBe('⚠');
  });

  it('displays correct icon for info notification', () => {
    const wrapper = mount(NotificationToast);
    
    const infoNotification = wrapper.find('.notification-info');
    expect(infoNotification.find('.notification-icon').text()).toBe('ℹ');
  });

  it('displays notification title and message', () => {
    const wrapper = mount(NotificationToast);
    
    const successNotification = wrapper.find('.notification-success');
    expect(successNotification.find('h4').text()).toBe('Success');
    expect(successNotification.find('p').text()).toBe('Operation completed');
  });

  it('calls removeNotification when notification is clicked', async () => {
    const { useNotifications } = await import('@/composables/useNotifications');
    const { removeNotification } = useNotifications();
    
    const wrapper = mount(NotificationToast);
    await wrapper.find('.notification-success').trigger('click');
    
    expect(removeNotification).toHaveBeenCalledWith('1');
  });

  it('calls removeNotification when close button is clicked', async () => {
    const { useNotifications } = await import('@/composables/useNotifications');
    const { removeNotification } = useNotifications();
    
    const wrapper = mount(NotificationToast);
    await wrapper.find('.notification-success .notification-close').trigger('click');
    
    expect(removeNotification).toHaveBeenCalledWith('1');
  });

  it('renders close button for each notification', () => {
    const wrapper = mount(NotificationToast);
    
    expect(wrapper.findAll('.notification-close').length).toBe(4);
  });
});
