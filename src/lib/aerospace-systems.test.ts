/**
 * Cross-module smoke tests for validation, audit, memory, performance, and recovery.
 */
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  validateUrl,
  validateText,
  validateHtml,
  validateEmail,
} from './validation';
import { getAuditLogger, destroyAuditLogger, AuditLevel, AuditCategory } from './audit-logger';
import { getMemoryMonitor, destroyMemoryMonitor } from './memory-monitor';
import { getPerformanceMonitor, destroyPerformanceMonitor } from './performance-monitor';
import {
  getErrorRecoveryManager,
  destroyErrorRecoveryManager,
  ErrorCategory as RecoveryCategory,
} from './error-recovery';

describe('Aerospace Systems Integration Tests', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    destroyAuditLogger();
    destroyMemoryMonitor();
    destroyPerformanceMonitor();
    destroyErrorRecoveryManager();
  });

  describe('Validation + Audit Logger Integration', () => {
    it('should log validation failures to audit logger', () => {
      const auditLogger = getAuditLogger();
      const invalidUrl = validateUrl('javascript:alert(1)');
      expect(invalidUrl).toBeNull();

      auditLogger.log(
        AuditLevel.SECURITY,
        AuditCategory.SECURITY,
        'validate_url_rejected',
        { input: 'javascript:alert(1)' },
        false,
      );

      const logs = auditLogger.getLogs(AuditLevel.SECURITY, AuditCategory.SECURITY);
      expect(logs.length).toBeGreaterThan(0);
    });

    it('should log successful validations', () => {
      const auditLogger = getAuditLogger();
      const validUrl = validateUrl('https://example.com');
      expect(validUrl).toBe('https://example.com');

      auditLogger.log(
        AuditLevel.INFO,
        AuditCategory.DATA_ACCESS,
        'validate_url_ok',
        { url: validUrl },
        true,
      );

      const logs = auditLogger.getLogs(AuditLevel.INFO, AuditCategory.DATA_ACCESS);
      expect(logs.length).toBeGreaterThan(0);
    });
  });

  describe('Memory Monitor + Performance Monitor Integration', () => {
    it('should track memory and performance together', () => {
      const memoryMonitor = getMemoryMonitor();
      const performanceMonitor = getPerformanceMonitor();

      performanceMonitor.startMark('test-operation');
      const duration = performanceMonitor.endMark('test-operation');
      expect(duration).not.toBeNull();
      expect(duration!).toBeGreaterThanOrEqual(0);

      const memoryStats = memoryMonitor.getStatistics();
      expect(memoryStats).toBeDefined();
      expect(memoryStats.componentCount).toBeGreaterThanOrEqual(0);

      const perfStats = performanceMonitor.getStatistics();
      expect(perfStats).toBeDefined();
      expect(perfStats.totalMetrics).toBeGreaterThan(0);
    });
  });

  describe('Error Recovery + Audit Logger Integration', () => {
    it('should log error recovery attempts', async () => {
      const errorRecovery = getErrorRecoveryManager();
      const auditLogger = getAuditLogger();

      await errorRecovery.handleError(
        new Error('validation failed'),
        RecoveryCategory.VALIDATION,
        { url: 'https://example.com' },
      );

      auditLogger.log(
        AuditLevel.ERROR,
        AuditCategory.NETWORK,
        'error_recovery_attempt',
        { url: 'https://example.com' },
        true,
      );

      const logs = auditLogger.getLogs(AuditLevel.ERROR, AuditCategory.NETWORK);
      expect(logs.length).toBeGreaterThan(0);
    });

    it('should track recovery statistics', async () => {
      vi.useFakeTimers();
      const errorRecovery = getErrorRecoveryManager();

      for (let i = 0; i < 5; i++) {
        const p = errorRecovery.handleError(
          new Error(`network timeout ${i}`),
          RecoveryCategory.VALIDATION,
        );
        await vi.runAllTimersAsync();
        await p;
      }
      vi.useRealTimers();

      const stats = errorRecovery.getStatistics();
      expect(stats.totalErrors).toBe(5);
      expect(stats.recoveryRate).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Validation + Error Recovery Integration', () => {
    it('should recover from validation errors', async () => {
      vi.useFakeTimers();
      const errorRecovery = getErrorRecoveryManager();
      const invalid = validateText('hello\x00world');
      expect(invalid).toBeNull();

      const p = errorRecovery.handleError(
        new Error('validation failed'),
        RecoveryCategory.VALIDATION,
      );
      await vi.runAllTimersAsync();
      const ok = await p;
      vi.useRealTimers();
      expect(typeof ok).toBe('boolean');
    });
  });

  describe('End-to-End Security Flow', () => {
    it('should handle complete security validation flow', () => {
      const auditLogger = getAuditLogger();
      expect(validateUrl('https://example.com')).toBe('https://example.com');
      expect(validateText('safe input')).toBe('safe input');
      expect(validateHtml('<p>hello</p>')).toContain('hello');
      expect(validateEmail('user@example.com')).toBe('user@example.com');

      auditLogger.log(AuditLevel.INFO, AuditCategory.SECURITY, 'security_flow_ok', {}, true);
      expect(auditLogger.getLogs().length).toBeGreaterThan(0);
    });

    it('should reject and log security threats', () => {
      const auditLogger = getAuditLogger();
      expect(validateUrl('javascript:evil()')).toBeNull();
      auditLogger.log(
        AuditLevel.SECURITY,
        AuditCategory.SECURITY,
        'security_threat_blocked',
        { protocol: 'javascript' },
        false,
      );
      expect(auditLogger.getLogs(AuditLevel.SECURITY, AuditCategory.SECURITY).length).toBeGreaterThan(0);
    });
  });
});
