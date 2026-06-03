import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  getMemoryMonitor,
  destroyMemoryMonitor,
} from './memory-monitor';

describe('Memory Monitor System Tests', () => {
  beforeEach(() => {
    destroyMemoryMonitor();
  });

  afterEach(() => {
    destroyMemoryMonitor();
  });

  describe('Basic Monitoring', () => {
    it('should start and stop monitoring', () => {
      const monitor = getMemoryMonitor();

      const stats = monitor.getStatistics();
      expect(stats).toBeDefined();
      expect(stats.currentUsage).toBeDefined();

      monitor.stopMonitoring();

      const statsAfterStop = monitor.getStatistics();
      expect(statsAfterStop).toBeDefined();
    });

    it('should get current memory usage', () => {
      const monitor = getMemoryMonitor();

      const usage = monitor.getCurrentMemoryUsage();
      if (usage) {
        expect(usage).toHaveProperty('used');
        expect(usage).toHaveProperty('total');
        expect(usage).toHaveProperty('limit');
        expect(usage).toHaveProperty('percentage');
      } else {
        expect(usage).toBeNull();
      }
    });
  });

  describe('Component Registration', () => {
    it('should register components', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('test-component-1');
      monitor.registerComponent('test-component-2');

      const stats = monitor.getStatistics();
      expect(stats.componentCount).toBe(2);
    });

    it('should unregister components', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('test-component-1');
      monitor.registerComponent('test-component-2');

      monitor.unregisterComponent('test-component-1');

      const stats = monitor.getStatistics();
      expect(stats.componentCount).toBe(1);
    });

    it('should handle duplicate registration', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('test-component');
      monitor.registerComponent('test-component');

      const stats = monitor.getStatistics();
      expect(stats.componentCount).toBe(1);
    });

    it('should handle unregistering non-existent component', () => {
      const monitor = getMemoryMonitor();

      expect(() => {
        monitor.unregisterComponent('non-existent');
      }).not.toThrow();
    });
  });

  describe('Event Listener Registration', () => {
    it('should register event listeners', () => {
      const monitor = getMemoryMonitor();

      monitor.registerListener('window', 'click');
      monitor.registerListener('document', 'scroll');

      const stats = monitor.getStatistics();
      expect(stats.listenerCount).toBe(2);
    });

    it('should unregister event listeners', () => {
      const monitor = getMemoryMonitor();

      monitor.registerListener('window', 'click');
      monitor.registerListener('window', 'click');

      monitor.unregisterListener('window', 'click');

      const stats = monitor.getStatistics();
      expect(stats.listenerCount).toBe(1);
    });

    it('should track multiple listeners for same target/event', () => {
      const monitor = getMemoryMonitor();

      monitor.registerListener('window', 'click');
      monitor.registerListener('window', 'click');
      monitor.registerListener('window', 'click');

      const stats = monitor.getStatistics();
      expect(stats.listenerCount).toBe(3);
    });
  });

  describe('Timer Registration', () => {
    it('should register intervals', () => {
      const monitor = getMemoryMonitor();

      const intervalId = setInterval(() => {}, 1000) as unknown as number;
      monitor.registerTimer(intervalId, 'interval');

      const stats = monitor.getStatistics();
      expect(stats.timerCount).toBe(1);

      clearInterval(intervalId);
    });

    it('should register timeouts', () => {
      const monitor = getMemoryMonitor();

      const timeoutId = setTimeout(() => {}, 1000) as unknown as number;
      monitor.registerTimer(timeoutId, 'timeout');

      const stats = monitor.getStatistics();
      expect(stats.timerCount).toBe(1);

      clearTimeout(timeoutId);
    });

    it('should unregister timers', () => {
      const monitor = getMemoryMonitor();

      const intervalId = setInterval(() => {}, 1000) as unknown as number;
      monitor.registerTimer(intervalId, 'interval');

      monitor.unregisterTimer(intervalId, 'interval');

      const stats = monitor.getStatistics();
      expect(stats.timerCount).toBe(0);

      clearInterval(intervalId);
    });
  });

  describe('Memory Snapshots', () => {
    it('should take memory snapshots', async () => {
      vi.useFakeTimers();
      const monitor = getMemoryMonitor();
      vi.advanceTimersByTime(5000);
      const snapshots = monitor.getSnapshots(10);
      expect(snapshots.length).toBeGreaterThan(0);
      expect(snapshots[0]).toHaveProperty('timestamp');
      expect(snapshots[0]).toHaveProperty('usedJSHeapSize');
      expect(snapshots[0]).toHaveProperty('totalJSHeapSize');
      vi.useRealTimers();
    });

    it('should limit number of snapshots', () => {
      const monitor = getMemoryMonitor();

      const snapshots = monitor.getSnapshots(5);
      expect(snapshots.length).toBeLessThanOrEqual(5);
    });
  });

  describe('Memory Leak Alerts', () => {
    it('should track memory leak alerts', () => {
      const monitor = getMemoryMonitor();

      // Register many components to trigger potential leak detection
      for (let i = 0; i < 100; i++) {
        monitor.registerComponent(`component-${i}`);
      }

      const alerts = monitor.getAlerts();
      expect(Array.isArray(alerts)).toBe(true);
    });

    it('should clear alerts', () => {
      const monitor = getMemoryMonitor();

      monitor.clearAlerts();

      const alerts = monitor.getAlerts();
      expect(alerts.length).toBe(0);
    });

    it('should limit number of alerts', () => {
      const monitor = getMemoryMonitor();

      const alerts = monitor.getAlerts(10);
      expect(alerts.length).toBeLessThanOrEqual(10);
    });
  });

  describe('Statistics', () => {
    it('should calculate comprehensive statistics', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('comp-1');
      monitor.registerComponent('comp-2');
      monitor.registerListener('window', 'click');

      const stats = monitor.getStatistics();
      expect(stats).toBeDefined();
      expect(stats.currentUsage).toBeDefined();
      expect(stats.snapshots).toBeGreaterThanOrEqual(0);
      expect(stats.alerts).toBeGreaterThanOrEqual(0);
      expect(stats.componentCount).toBe(2);
      expect(stats.listenerCount).toBe(1);
      expect(stats.timerCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Baseline Management', () => {
    it('should reset baseline', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('comp-1');

      monitor.resetBaseline();

      const stats = monitor.getStatistics();
      expect(stats).toBeDefined();
    });
  });

  describe('Garbage Collection', () => {
    it('should attempt garbage collection if available', () => {
      const monitor = getMemoryMonitor();

      const result = monitor.forceGC();
      expect(typeof result).toBe('boolean');
    });
  });

  describe('Cleanup', () => {
    it('should destroy monitor and clean up resources', () => {
      const monitor = getMemoryMonitor();

      monitor.registerComponent('comp-1');
      monitor.registerListener('window', 'click');

      monitor.destroy();

      const stats = monitor.getStatistics();
      expect(stats.componentCount).toBe(0);
      expect(stats.listenerCount).toBe(0);
      expect(stats.timerCount).toBe(0);
    });
  });

  describe('Concurrent Operations', () => {
    it('should handle concurrent component registration', async () => {
      const monitor = getMemoryMonitor();
      const promises: Promise<void>[] = [];

      for (let i = 0; i < 100; i++) {
        promises.push(
          new Promise(resolve => {
            setTimeout(() => {
              monitor.registerComponent(`component-${i}`);
              resolve();
            }, Math.random() * 10);
          })
        );
      }

      await Promise.all(promises);

      const stats = monitor.getStatistics();
      expect(stats.componentCount).toBe(100);
    });

    it('should handle concurrent listener registration', async () => {
      const monitor = getMemoryMonitor();
      const promises: Promise<void>[] = [];

      for (let i = 0; i < 50; i++) {
        promises.push(
          new Promise(resolve => {
            setTimeout(() => {
              monitor.registerListener('window', 'click');
              resolve();
            }, Math.random() * 10);
          })
        );
      }

      await Promise.all(promises);

      const stats = monitor.getStatistics();
      expect(stats.listenerCount).toBe(50);
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty component ID', () => {
      const monitor = getMemoryMonitor();

      expect(() => {
        monitor.registerComponent('');
      }).not.toThrow();
    });

    it('should handle special characters in component ID', () => {
      const monitor = getMemoryMonitor();

      expect(() => {
        monitor.registerComponent('component-with-special-chars-!@#$%');
      }).not.toThrow();
    });

    it('should handle very long component ID', () => {
      const monitor = getMemoryMonitor();

      const longId = 'a'.repeat(1000);
      expect(() => {
        monitor.registerComponent(longId);
      }).not.toThrow();
    });
  });
});
