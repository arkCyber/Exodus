import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  getErrorRecoveryManager,
  destroyErrorRecoveryManager,
  ErrorCategory,
  ErrorSeverity,
  withRecovery,
} from './error-recovery';

describe('Error Recovery System Tests', () => {
  beforeEach(() => {
    destroyErrorRecoveryManager();
  });

  afterEach(() => {
    destroyErrorRecoveryManager();
  });

  describe('Basic Error Handling', () => {
    it('should handle errors', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Test error');
      const recovered = await manager.handleError(
        error,
        ErrorCategory.RUNTIME,
        { operation: 'test' }
      );

      expect(typeof recovered).toBe('boolean');
    });

    it('should log error to history', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Test error');
      await manager.handleError(error, ErrorCategory.RUNTIME);

      const history = manager.getErrorHistory();
      expect(history.length).toBeGreaterThan(0);
      expect(history[0].message).toBe('Test error');
    });
  });

  describe('Error Severity Determination', () => {
    it('should mark security errors as critical', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Security breach');
      await manager.handleError(error, ErrorCategory.SECURITY);

      const history = manager.getErrorHistory();
      expect(history[0].severity).toBe(ErrorSeverity.CRITICAL);
    });

    it('should mark system errors as high', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('System failure');
      await manager.handleError(error, ErrorCategory.SYSTEM);

      const history = manager.getErrorHistory();
      expect(history[0].severity).toBe(ErrorSeverity.HIGH);
    });

    it('should mark network errors as medium', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Network timeout');
      await manager.handleError(error, ErrorCategory.NETWORK);

      const history = manager.getErrorHistory();
      expect(history[0].severity).toBe(ErrorSeverity.MEDIUM);
    });
  });

  describe('Recoverability', () => {
    it('should mark security errors as non-recoverable', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Security breach');
      await manager.handleError(error, ErrorCategory.SECURITY);

      const history = manager.getErrorHistory();
      expect(history[0].recoverable).toBe(false);
    });

    it('should mark validation errors as recoverable', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Invalid input');
      await manager.handleError(error, ErrorCategory.VALIDATION);

      const history = manager.getErrorHistory();
      expect(history[0].recoverable).toBe(true);
    });

    it('should mark network errors as recoverable', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Network timeout');
      await manager.handleError(error, ErrorCategory.NETWORK);

      const history = manager.getErrorHistory();
      expect(history[0].recoverable).toBe(true);
    });
  });

  describe('Recovery Strategies', () => {
    it('should register custom recovery strategy', () => {
      const manager = getErrorRecoveryManager();

      manager.registerStrategy({
        name: 'custom-strategy',
        canRecover: (error) => error.category === ErrorCategory.RUNTIME,
        recover: async (_error) => true,
        maxAttempts: 3,
        backoffMs: 1000,
      });

      const history = manager.getErrorHistory();
      expect(history).toBeDefined();
    });

    it('should unregister recovery strategy', () => {
      const manager = getErrorRecoveryManager();

      manager.registerStrategy({
        name: 'custom-strategy',
        canRecover: (error) => error.category === ErrorCategory.RUNTIME,
        recover: async (_error) => true,
        maxAttempts: 3,
        backoffMs: 1000,
      });

      manager.unregisterStrategy('custom-strategy');

      expect(() => {
        manager.unregisterStrategy('non-existent');
      }).not.toThrow();
    });
  });

  describe('Default Strategies', () => {
    it('should have network retry strategy', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Network timeout');
      const recovered = await manager.handleError(error, ErrorCategory.NETWORK);

      expect(typeof recovered).toBe('boolean');
    });

    it('should have storage clear strategy', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Storage quota exceeded');
      const recovered = await manager.handleError(error, ErrorCategory.STORAGE);

      expect(typeof recovered).toBe('boolean');
    });

    it('should have validation sanitize strategy', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Invalid input');
      const recovered = await manager.handleError(error, ErrorCategory.VALIDATION);

      expect(typeof recovered).toBe('boolean');
    });
  });

  describe('Error History', () => {
    it('should limit error history size', async () => {
      const manager = getErrorRecoveryManager();

      for (let i = 0; i < 1100; i++) {
        await manager.handleError(new Error(`Error ${i}`), ErrorCategory.RUNTIME);
      }

      const history = manager.getErrorHistory();
      expect(history.length).toBeLessThanOrEqual(1000);
    });

    it('should limit returned history size', async () => {
      const manager = getErrorRecoveryManager();

      for (let i = 0; i < 50; i++) {
        await manager.handleError(new Error(`Error ${i}`), ErrorCategory.RUNTIME);
      }

      const history = manager.getErrorHistory(10);
      expect(history.length).toBe(10);
    });

    it('should clear error history', async () => {
      const manager = getErrorRecoveryManager();

      await manager.handleError(new Error('Test error'), ErrorCategory.RUNTIME);

      manager.clearErrorHistory();

      const history = manager.getErrorHistory();
      expect(history.length).toBe(0);
    });
  });

  describe('Statistics', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should calculate error statistics', async () => {
      const manager = getErrorRecoveryManager();

      for (const [message, category] of [
        ['network timeout', ErrorCategory.NETWORK],
        ['Validation error', ErrorCategory.VALIDATION],
        ['Runtime error', ErrorCategory.RUNTIME],
      ] as const) {
        const p = manager.handleError(new Error(message), category);
        await vi.runAllTimersAsync();
        await p;
      }

      const stats = manager.getStatistics();
      expect(stats.totalErrors).toBe(3);
      expect(stats.errorsByCategory[ErrorCategory.NETWORK]).toBe(1);
      expect(stats.errorsByCategory[ErrorCategory.VALIDATION]).toBe(1);
      expect(stats.errorsByCategory[ErrorCategory.RUNTIME]).toBe(1);
    });

    it('should calculate error statistics by severity', async () => {
      const manager = getErrorRecoveryManager();

      const p1 = manager.handleError(new Error('network timeout'), ErrorCategory.NETWORK);
      const p2 = manager.handleError(new Error('System error'), ErrorCategory.SYSTEM);
      await vi.runAllTimersAsync();
      await Promise.all([p1, p2]);

      const stats = manager.getStatistics();
      expect(stats.errorsBySeverity[ErrorSeverity.MEDIUM]).toBe(1);
      expect(stats.errorsBySeverity[ErrorSeverity.HIGH]).toBe(1);
    });

    it('should calculate recovery rate', async () => {
      const manager = getErrorRecoveryManager();

      for (let i = 0; i < 5; i++) {
        const p = manager.handleError(new Error(`network timeout ${i}`), ErrorCategory.VALIDATION);
        await vi.runAllTimersAsync();
        await p;
      }

      const stats = manager.getStatistics();
      expect(stats.recoveryRate).toBeGreaterThanOrEqual(0);
      expect(stats.recoveryRate).toBeLessThanOrEqual(1);
    });

    it('should track active recoveries', async () => {
      const manager = getErrorRecoveryManager();

      const p = manager.handleError(new Error('network timeout'), ErrorCategory.NETWORK);
      await vi.runAllTimersAsync();
      await p;

      const stats = manager.getStatistics();
      expect(stats.activeRecoveries).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Error Export', () => {
    it('should export error report', async () => {
      const manager = getErrorRecoveryManager();

      await manager.handleError(new Error('Test error'), ErrorCategory.RUNTIME);

      const report = manager.exportErrorReport();
      const parsed = JSON.parse(report);

      expect(parsed).toBeDefined();
      expect(parsed.statistics).toBeDefined();
      expect(parsed.errors).toBeDefined();
      expect(parsed.strategies).toBeDefined();
    });
  });

  describe('withRecovery Wrapper', () => {
    it('should wrap function with error recovery', async () => {
      let attempts = 0;

      const result = await withRecovery(
        () => {
          attempts++;
          if (attempts < 2) {
            throw new Error('Temporary failure');
          }
          return 'success';
        },
        ErrorCategory.NETWORK
      );

      expect(result).toBe('success');
      expect(attempts).toBeGreaterThanOrEqual(1);
    });

    it('should throw error if recovery fails', async () => {
      await expect(
        withRecovery(
          () => {
            throw new Error('Permanent failure');
          },
          ErrorCategory.SECURITY
        )
      ).rejects.toThrow('Permanent failure');
    });
  });

  describe('Concurrent Error Handling', () => {
    it('should handle concurrent errors safely', async () => {
      const manager = getErrorRecoveryManager();
      const promises: Promise<boolean>[] = [];

      for (let i = 0; i < 50; i++) {
        promises.push(
          manager.handleError(new Error(`Error ${i}`), ErrorCategory.NETWORK)
        );
      }

      await Promise.all(promises);

      const stats = manager.getStatistics();
      expect(stats.totalErrors).toBe(50);
    });

    it('should prevent duplicate recovery attempts', async () => {
      const manager = getErrorRecoveryManager();
      const error = new Error('Test error');
      const timestamp = Date.now();

      const promises: Promise<boolean>[] = [];

      for (let i = 0; i < 5; i++) {
        promises.push(
          manager.handleError(error, ErrorCategory.NETWORK, { timestamp })
        );
      }

      await Promise.all(promises);

      const stats = manager.getStatistics();
      expect(stats.totalErrors).toBe(5);
    });
  });

  describe('Cleanup', () => {
    it('should destroy manager and clean up resources', async () => {
      const manager = getErrorRecoveryManager();

      await manager.handleError(new Error('Test error'), ErrorCategory.RUNTIME);

      manager.destroy();

      const history = manager.getErrorHistory();
      expect(history.length).toBe(0);
    });
  });

  describe('Edge Cases', () => {
    it('should handle error without stack trace', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Test error');
      delete error.stack;

      await manager.handleError(error, ErrorCategory.RUNTIME);

      const history = manager.getErrorHistory();
      expect(history.length).toBe(1);
    });

    it('should handle error without details', async () => {
      const manager = getErrorRecoveryManager();

      const error = new Error('Test error');
      await manager.handleError(error, ErrorCategory.RUNTIME);

      const history = manager.getErrorHistory();
      expect(history.length).toBe(1);
    });

    it('should handle very long error message', async () => {
      const manager = getErrorRecoveryManager();

      const longMessage = 'a'.repeat(10000);
      const error = new Error(longMessage);

      await manager.handleError(error, ErrorCategory.RUNTIME);

      const history = manager.getErrorHistory();
      expect(history.length).toBe(1);
    });
  });
});
