import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  getAuditLogger,
  destroyAuditLogger,
  AuditLevel,
  AuditCategory,
} from './audit-logger';

describe('Audit Logger System Tests', () => {
  beforeEach(() => {
    localStorage.clear();
    destroyAuditLogger();
  });

  afterEach(() => {
    destroyAuditLogger();
  });

  describe('Basic Logging', () => {
    it('should log basic events', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'test-event');

      const logs = logger.getLogs();
      expect(logs.length).toBe(1);
      expect(logs[0].action).toBe('test-event');
      expect(logs[0].level).toBe(AuditLevel.INFO);
      expect(logs[0].category).toBe(AuditCategory.SYSTEM);
    });

    it('should log events with details', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'test-event', { key: 'value' });

      const logs = logger.getLogs();
      expect(logs[0].details).toEqual({ key: 'value' });
    });

    it('should log events with error information', () => {
      const logger = getAuditLogger();

      logger.log(
        AuditLevel.ERROR,
        AuditCategory.SYSTEM,
        'test-event',
        { key: 'value' },
        false,
        'Test error message',
        'Error stack trace'
      );

      const logs = logger.getLogs();
      expect(logs[0].success).toBe(false);
      expect(logs[0].errorMessage).toBe('Test error message');
      expect(logs[0].stackTrace).toBe('Error stack trace');
    });
  });

  describe('Specialized Logging Methods', () => {
    it('should log authentication events', () => {
      const logger = getAuditLogger();

      logger.logAuthentication('login', 'user-123', true);

      const logs = logger.getLogs(AuditLevel.SECURITY, AuditCategory.AUTHENTICATION);
      expect(logs.length).toBe(1);
      expect(logs[0].action).toBe('login');
      expect(logs[0].details?.userId).toBe('user-123');
    });

    it('should log authorization events', () => {
      const logger = getAuditLogger();

      logger.logAuthorization('access-resource', 'user-123', '/api/data');

      const logs = logger.getLogs(AuditLevel.SECURITY, AuditCategory.AUTHORIZATION);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.resource).toBe('/api/data');
    });

    it('should log data access events', () => {
      const logger = getAuditLogger();

      logger.logDataAccess('read-record', 'users', 'user-123');

      const logs = logger.getLogs(AuditLevel.INFO, AuditCategory.DATA_ACCESS);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.resource).toBe('users');
    });

    it('should log data modification events', () => {
      const logger = getAuditLogger();

      logger.logDataModification('update-record', 'users', 'user-123', { field: 'name' });

      const logs = logger.getLogs(AuditLevel.INFO, AuditCategory.DATA_MODIFICATION);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.field).toBe('name');
    });

    it('should log network events', () => {
      const logger = getAuditLogger();

      logger.logNetwork('http-request', 'https://api.example.com', 'GET', 200, 150);

      const logs = logger.getLogs(AuditLevel.INFO, AuditCategory.NETWORK);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.url).toBe('https://api.example.com');
      expect(logs[0].details?.statusCode).toBe(200);
    });

    it('should log security events', () => {
      const logger = getAuditLogger();

      logger.logSecurity('xss-attempt', 'script-injection');

      const logs = logger.getLogs(AuditLevel.SECURITY, AuditCategory.SECURITY);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.threat).toBe('script-injection');
    });

    it('should log performance events', () => {
      const logger = getAuditLogger();

      logger.logPerformance('database-query', 50, { query: 'SELECT * FROM users' });

      const logs = logger.getLogs(AuditLevel.INFO, AuditCategory.PERFORMANCE);
      expect(logs.length).toBe(1);
      expect(logs[0].details?.duration).toBe(50);
    });

    it('should log error events', () => {
      const logger = getAuditLogger();

      const error = new Error('Test error');
      logger.logError('operation-failed', error, { operation: 'test' });

      const logs = logger.getLogs(AuditLevel.ERROR, AuditCategory.SYSTEM);
      expect(logs.length).toBe(1);
      expect(logs[0].errorMessage).toBe('Test error');
    });

    it('should log critical events', () => {
      const logger = getAuditLogger();

      logger.logCritical('system-failure', 'Database connection lost');

      const logs = logger.getLogs(AuditLevel.CRITICAL, AuditCategory.SYSTEM);
      expect(logs.length).toBe(1);
      expect(logs[0].errorMessage).toBe('Database connection lost');
    });
  });

  describe('Log Filtering', () => {
    it('should filter logs by level', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'info-event');
      logger.log(AuditLevel.ERROR, AuditCategory.SYSTEM, 'error-event');
      logger.log(AuditLevel.SECURITY, AuditCategory.SECURITY, 'security-event');

      const errorLogs = logger.getLogs(AuditLevel.ERROR);
      expect(errorLogs.length).toBe(1);
      expect(errorLogs[0].action).toBe('error-event');
    });

    it('should filter logs by category', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'system-event');
      logger.log(AuditLevel.INFO, AuditCategory.NETWORK, 'network-event');
      logger.log(AuditLevel.INFO, AuditCategory.SECURITY, 'security-event');

      const networkLogs = logger.getLogs(undefined, AuditCategory.NETWORK);
      expect(networkLogs.length).toBe(1);
      expect(networkLogs[0].action).toBe('network-event');
    });

    it('should filter logs by time range', () => {
      const logger = getAuditLogger();

      const now = Date.now();
      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'event-1');

      // Wait a bit
      const later = now + 1000;
      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'event-2');

      const logs = logger.getLogs(undefined, undefined, now, later);
      expect(logs.length).toBe(2);
    });

    it('should limit number of returned logs', () => {
      const logger = getAuditLogger();

      for (let i = 0; i < 10; i++) {
        logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, `event-${i}`);
      }

      const logs = logger.getLogs(undefined, undefined, undefined, undefined, 5);
      expect(logs.length).toBe(5);
    });
  });

  describe('Statistics', () => {
    it('should calculate statistics correctly', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'info-event');
      logger.log(AuditLevel.ERROR, AuditCategory.SYSTEM, 'error-event', {}, false);
      logger.log(AuditLevel.SECURITY, AuditCategory.SECURITY, 'security-event', {}, false);

      const stats = logger.getStatistics();
      expect(stats.totalLogs).toBe(3);
      expect(stats.logsByLevel[AuditLevel.INFO]).toBe(1);
      expect(stats.logsByLevel[AuditLevel.ERROR]).toBe(1);
      expect(stats.logsByLevel[AuditLevel.SECURITY]).toBe(1);
      expect(stats.errorRate).toBe(2 / 3);
    });

    it('should track logs by category', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'system-event');
      logger.log(AuditLevel.INFO, AuditCategory.NETWORK, 'network-event');
      logger.log(AuditLevel.INFO, AuditCategory.SECURITY, 'security-event');

      const stats = logger.getStatistics();
      expect(stats.logsByCategory[AuditCategory.SYSTEM]).toBe(1);
      expect(stats.logsByCategory[AuditCategory.NETWORK]).toBe(1);
      expect(stats.logsByCategory[AuditCategory.SECURITY]).toBe(1);
    });
  });

  describe('Persistence', () => {
    it('should persist logs to localStorage', () => {
      const logger1 = getAuditLogger();

      logger1.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'persistent-event');

      destroyAuditLogger();

      const logger2 = getAuditLogger();
      const logs = logger2.getLogs();

      expect(logs.length).toBe(1);
      expect(logs[0].action).toBe('persistent-event');
    });

    it('should handle localStorage quota errors gracefully', () => {
      const logger = getAuditLogger();

      for (let i = 0; i < 200; i++) {
        logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, `event-${i}`, { data: 'x'.repeat(200) });
      }

      const stats = logger.getStatistics();
      expect(stats).toBeDefined();
      expect(stats.totalLogs).toBeGreaterThan(0);
    });
  });

  describe('Log Management', () => {
    it('should clear logs', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'event-1');
      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'event-2');

      logger.clearLogs();

      const logs = logger.getLogs();
      expect(logs.length).toBe(0);
    });

    it('should export logs as JSON', () => {
      const logger = getAuditLogger();

      logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, 'test-event');

      const exported = logger.exportLogs();
      const parsed = JSON.parse(exported);

      expect(Array.isArray(parsed)).toBe(true);
      expect(parsed.length).toBe(1);
      expect(parsed[0].action).toBe('test-event');
    });

    it('should maintain max log entries', () => {
      const logger = getAuditLogger();

      // Add more than max entries
      for (let i = 0; i < 11000; i++) {
        logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, `event-${i}`);
      }

      const stats = logger.getStatistics();
      expect(stats.totalLogs).toBeLessThanOrEqual(10000);
    });
  });

  describe('Critical Event Handling', () => {
    it('should immediately flush critical events', () => {
      const logger = getAuditLogger();

      logger.logCritical('critical-failure', 'System crash');

      // Critical events should be persisted immediately
      const logs = logger.getLogs(AuditLevel.CRITICAL);
      expect(logs.length).toBe(1);
    });

    it('should immediately flush security events', () => {
      const logger = getAuditLogger();

      logger.logSecurity('security-breach', 'Unauthorized access');

      // Security events should be persisted immediately
      const logs = logger.getLogs(AuditLevel.SECURITY, AuditCategory.SECURITY);
      expect(logs.length).toBe(1);
    });
  });

  describe('Concurrent Logging', () => {
    it('should handle concurrent logging safely', async () => {
      const logger = getAuditLogger();
      const promises: Promise<void>[] = [];

      for (let i = 0; i < 100; i++) {
        promises.push(
          new Promise(resolve => {
            setTimeout(() => {
              logger.log(AuditLevel.INFO, AuditCategory.SYSTEM, `event-${i}`);
              resolve();
            }, Math.random() * 10);
          })
        );
      }

      await Promise.all(promises);

      const stats = logger.getStatistics();
      expect(stats.totalLogs).toBe(100);
    });
  });
});
