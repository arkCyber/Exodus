/**
 * Aerospace-grade security audit logging
 * Provides comprehensive logging for security events and system operations
 */

export enum AuditLevel {
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
  CRITICAL = 'CRITICAL',
  SECURITY = 'SECURITY',
}

export enum AuditCategory {
  AUTHENTICATION = 'AUTHENTICATION',
  AUTHORIZATION = 'AUTHORIZATION',
  DATA_ACCESS = 'DATA_ACCESS',
  DATA_MODIFICATION = 'DATA_MODIFICATION',
  NETWORK = 'NETWORK',
  SYSTEM = 'SYSTEM',
  SECURITY = 'SECURITY',
  PERFORMANCE = 'PERFORMANCE',
}

export interface AuditEntry {
  timestamp: number;
  level: AuditLevel;
  category: AuditCategory;
  action: string;
  userId?: string;
  sessionId?: string;
  ipAddress?: string;
  userAgent?: string;
  details?: Record<string, unknown>;
  success: boolean;
  errorMessage?: string;
  stackTrace?: string;
}

const MAX_LOG_ENTRIES = 10000;
const MAX_LOG_SIZE_BYTES = 10 * 1024 * 1024; // 10MB

class AuditLogger {
  private logs: AuditEntry[] = [];
  private currentSize: number = 0;
  private flushInterval: number | null = null;
  private readonly FLUSH_INTERVAL_MS = 30000; // 30 seconds

  constructor() {
    this.loadLogs();
    this.startFlushInterval();
  }

  /**
   * Log an audit event
   */
  log(
    level: AuditLevel,
    category: AuditCategory,
    action: string,
    details?: Record<string, unknown>,
    success: boolean = true,
    errorMessage?: string,
    stackTrace?: string
  ): void {
    const entry: AuditEntry = {
      timestamp: Date.now(),
      level,
      category,
      action,
      details,
      success,
      errorMessage,
      stackTrace,
    };

    const entrySize = JSON.stringify(entry).length;

    // Check size limits
    if (this.currentSize + entrySize > MAX_LOG_SIZE_BYTES) {
      this.flushLogs();
    }

    if (this.logs.length >= MAX_LOG_ENTRIES) {
      this.logs.shift(); // Remove oldest entry
    }

    this.logs.push(entry);
    this.currentSize += entrySize;

    // Immediate flush for critical/security events
    if (level === AuditLevel.CRITICAL || level === AuditLevel.SECURITY) {
      this.flushLogs();
    }
  }

  /**
   * Log authentication event
   */
  logAuthentication(
    action: string,
    userId?: string,
    success: boolean = true,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.SECURITY,
      AuditCategory.AUTHENTICATION,
      action,
      { userId, ...details },
      success,
      success ? undefined : 'Authentication failed'
    );
  }

  /**
   * Log authorization event
   */
  logAuthorization(
    action: string,
    userId?: string,
    resource?: string,
    success: boolean = true
  ): void {
    this.log(
      AuditLevel.SECURITY,
      AuditCategory.AUTHORIZATION,
      action,
      { userId, resource },
      success,
      success ? undefined : 'Authorization denied'
    );
  }

  /**
   * Log data access event
   */
  logDataAccess(
    action: string,
    resource: string,
    userId?: string,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.INFO,
      AuditCategory.DATA_ACCESS,
      action,
      { resource, userId, ...details }
    );
  }

  /**
   * Log data modification event
   */
  logDataModification(
    action: string,
    resource: string,
    userId?: string,
    details?: Record<string, unknown>,
    success: boolean = true
  ): void {
    this.log(
      AuditLevel.INFO,
      AuditCategory.DATA_MODIFICATION,
      action,
      { resource, userId, ...details },
      success,
      success ? undefined : 'Data modification failed'
    );
  }

  /**
   * Log network event
   */
  logNetwork(
    action: string,
    url: string,
    method: string,
    statusCode?: number,
    duration?: number,
    success: boolean = true
  ): void {
    this.log(
      AuditLevel.INFO,
      AuditCategory.NETWORK,
      action,
      { url, method, statusCode, duration },
      success,
      success ? undefined : `Network request failed: ${statusCode}`
    );
  }

  /**
   * Log security event
   */
  logSecurity(
    action: string,
    threat: string,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.SECURITY,
      AuditCategory.SECURITY,
      action,
      { threat, ...details },
      false,
      `Security threat detected: ${threat}`
    );
  }

  /**
   * Log performance event
   */
  logPerformance(
    action: string,
    duration: number,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.INFO,
      AuditCategory.PERFORMANCE,
      action,
      { duration, ...details }
    );
  }

  /**
   * Log error event
   */
  logError(
    action: string,
    error: Error,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.ERROR,
      AuditCategory.SYSTEM,
      action,
      details,
      false,
      error.message,
      error.stack
    );
  }

  /**
   * Log critical event
   */
  logCritical(
    action: string,
    errorMessage: string,
    details?: Record<string, unknown>
  ): void {
    this.log(
      AuditLevel.CRITICAL,
      AuditCategory.SYSTEM,
      action,
      details,
      false,
      errorMessage
    );
  }

  /**
   * Get logs with filtering
   */
  getLogs(
    level?: AuditLevel,
    category?: AuditCategory,
    startTime?: number,
    endTime?: number,
    limit: number = 100
  ): AuditEntry[] {
    let filtered = this.logs;

    if (level) {
      filtered = filtered.filter(log => log.level === level);
    }

    if (category) {
      filtered = filtered.filter(log => log.category === category);
    }

    if (startTime) {
      filtered = filtered.filter(log => log.timestamp >= startTime);
    }

    if (endTime) {
      filtered = filtered.filter(log => log.timestamp <= endTime);
    }

    return filtered.slice(-limit).reverse();
  }

  /**
   * Get statistics
   */
  getStatistics(): {
    totalLogs: number;
    logsByLevel: Record<AuditLevel, number>;
    logsByCategory: Record<AuditCategory, number>;
    errorRate: number;
    currentSize: number;
  } {
    const logsByLevel: Record<AuditLevel, number> = {
      [AuditLevel.DEBUG]: 0,
      [AuditLevel.INFO]: 0,
      [AuditLevel.WARN]: 0,
      [AuditLevel.ERROR]: 0,
      [AuditLevel.CRITICAL]: 0,
      [AuditLevel.SECURITY]: 0,
    };

    const logsByCategory: Record<AuditCategory, number> = {
      [AuditCategory.AUTHENTICATION]: 0,
      [AuditCategory.AUTHORIZATION]: 0,
      [AuditCategory.DATA_ACCESS]: 0,
      [AuditCategory.DATA_MODIFICATION]: 0,
      [AuditCategory.NETWORK]: 0,
      [AuditCategory.SYSTEM]: 0,
      [AuditCategory.SECURITY]: 0,
      [AuditCategory.PERFORMANCE]: 0,
    };

    let errorCount = 0;

    for (const log of this.logs) {
      logsByLevel[log.level]++;
      logsByCategory[log.category]++;
      if (!log.success) {
        errorCount++;
      }
    }

    return {
      totalLogs: this.logs.length,
      logsByLevel,
      logsByCategory,
      errorRate: this.logs.length > 0 ? errorCount / this.logs.length : 0,
      currentSize: this.currentSize,
    };
  }

  /**
   * Clear logs
   */
  clearLogs(): void {
    this.logs = [];
    this.currentSize = 0;
    this.persistLogs();
  }

  /**
   * Export logs as JSON
   */
  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  /**
   * Persist logs to localStorage
   */
  private persistLogs(): void {
    try {
      const data = JSON.stringify({ logs: this.logs, size: this.currentSize });
      localStorage.setItem('audit-logs', data);
    } catch (e) {
      console.error('Failed to persist audit logs:', e);
    }
  }

  /**
   * Load logs from localStorage
   */
  private loadLogs(): void {
    try {
      const data = localStorage.getItem('audit-logs');
      if (data) {
        const parsed = JSON.parse(data);
        if (Array.isArray(parsed.logs)) {
          this.logs = parsed.logs;
          this.currentSize = parsed.size || 0;
        }
      }
    } catch (e) {
      console.error('Failed to load audit logs:', e);
      this.logs = [];
      this.currentSize = 0;
    }
  }

  /**
   * Flush logs to persistent storage
   */
  private flushLogs(): void {
    this.persistLogs();
  }

  /**
   * Start automatic flush interval
   */
  private startFlushInterval(): void {
    this.flushInterval = window.setInterval(() => {
      this.flushLogs();
    }, this.FLUSH_INTERVAL_MS);
  }

  /**
   * Stop automatic flush interval
   */
  private stopFlushInterval(): void {
    if (this.flushInterval) {
      clearInterval(this.flushInterval);
      this.flushInterval = null;
    }
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.stopFlushInterval();
    this.flushLogs();
  }
}

// Singleton instance
let auditLoggerInstance: AuditLogger | null = null;

export function getAuditLogger(): AuditLogger {
  if (!auditLoggerInstance) {
    auditLoggerInstance = new AuditLogger();
  }
  return auditLoggerInstance;
}

export function destroyAuditLogger(): void {
  if (auditLoggerInstance) {
    auditLoggerInstance.destroy();
    auditLoggerInstance = null;
  }
}
