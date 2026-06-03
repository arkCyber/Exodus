/**
 * Exodus Browser — useNotifications composable tests.
 */
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useNotifications } from './useNotifications';

describe('useNotifications', () => {
  beforeEach(() => {
    // Clear notifications before each test
    const { clearNotifications } = useNotifications();
    clearNotifications();
  });

  it('adds a notification', () => {
    const { addNotification, notifications } = useNotifications();
    const id = addNotification('success', 'Test Title', 'Test Message');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].id).toBe(id);
    expect(notifications.value[0].type).toBe('success');
    expect(notifications.value[0].title).toBe('Test Title');
    expect(notifications.value[0].message).toBe('Test Message');
  });

  it('adds notification without message', () => {
    const { addNotification, notifications } = useNotifications();
    addNotification('info', 'Info Only');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].message).toBeUndefined();
  });

  it('removes a notification', () => {
    const { addNotification, removeNotification, notifications } = useNotifications();
    const id = addNotification('success', 'Test');
    
    removeNotification(id);
    
    expect(notifications.value.length).toBe(0);
  });

  it('clears all notifications', () => {
    const { addNotification, clearNotifications, notifications } = useNotifications();
    addNotification('success', 'Test 1');
    addNotification('error', 'Test 2');
    addNotification('warning', 'Test 3');
    
    clearNotifications();
    
    expect(notifications.value.length).toBe(0);
  });

  it('success helper adds success notification', () => {
    const { success, notifications } = useNotifications();
    success('Success Title', 'Success Message');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].type).toBe('success');
  });

  it('error helper adds error notification', () => {
    const { error, notifications } = useNotifications();
    error('Error Title', 'Error Message');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].type).toBe('error');
  });

  it('warning helper adds warning notification', () => {
    const { warning, notifications } = useNotifications();
    warning('Warning Title', 'Warning Message');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].type).toBe('warning');
  });

  it('info helper adds info notification', () => {
    const { info, notifications } = useNotifications();
    info('Info Title', 'Info Message');
    
    expect(notifications.value.length).toBe(1);
    expect(notifications.value[0].type).toBe('info');
  });

  it('auto-removes notification after duration', async () => {
    vi.useFakeTimers();
    const { addNotification, notifications } = useNotifications();
    addNotification('success', 'Test', 'Message', 1000);
    
    expect(notifications.value.length).toBe(1);
    
    await vi.advanceTimersByTimeAsync(1000);
    
    expect(notifications.value.length).toBe(0);
    vi.useRealTimers();
  });

  it('does not auto-remove notification with duration 0', () => {
    vi.useFakeTimers();
    const { addNotification, notifications } = useNotifications();
    addNotification('success', 'Test', 'Message', 0);
    
    vi.advanceTimersByTime(5000);
    
    expect(notifications.value.length).toBe(1);
    vi.useRealTimers();
  });

  it('generates unique IDs for notifications', () => {
    const { addNotification, notifications } = useNotifications();
    const id1 = addNotification('success', 'Test 1');
    const id2 = addNotification('success', 'Test 2');
    
    expect(id1).not.toBe(id2);
    expect(notifications.value.length).toBe(2);
  });

  it('includes timestamp in notification', () => {
    const { addNotification, notifications } = useNotifications();
    const before = Date.now();
    addNotification('success', 'Test');
    const after = Date.now();
    
    expect(notifications.value[0].timestamp).toBeGreaterThanOrEqual(before);
    expect(notifications.value[0].timestamp).toBeLessThanOrEqual(after);
  });
});
