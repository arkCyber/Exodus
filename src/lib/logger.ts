/**
 * Exodus Browser - Unified logging utility
 * Provides consistent, structured logging across the application.
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  component: string;
  message: string;
  data?: unknown;
}

class Logger {
  private static instance: Logger;
  private logLevel: LogLevel = LogLevel.DEBUG;
  private logs: LogEntry[] = [];
  private maxLogs = 1000;

  private constructor() {}

  static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  setLogLevel(level: LogLevel): void {
    this.logLevel = level;
  }

  private formatTimestamp(): string {
    return new Date().toISOString();
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= this.logLevel;
  }

  private addLog(level: LogLevel, component: string, message: string, data?: unknown): void {
    const entry: LogEntry = {
      timestamp: this.formatTimestamp(),
      level,
      component,
      message,
      data,
    };

    this.logs.push(entry);
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }
  }

  private formatLog(level: LogLevel, component: string, message: string): string {
    const levelStr = LogLevel[level];
    return `[${levelStr}] [${component}] ${message}`;
  }

  debug(component: string, message: string, data?: unknown): void {
    if (!this.shouldLog(LogLevel.DEBUG)) return;
    this.addLog(LogLevel.DEBUG, component, message, data);
    const formatted = this.formatLog(LogLevel.DEBUG, component, message);
    if (data !== undefined) {
      console.debug(formatted, data);
    } else {
      console.debug(formatted);
    }
  }

  info(component: string, message: string, data?: unknown): void {
    if (!this.shouldLog(LogLevel.INFO)) return;
    this.addLog(LogLevel.INFO, component, message, data);
    const formatted = this.formatLog(LogLevel.INFO, component, message);
    if (data !== undefined) {
      console.info(formatted, data);
    } else {
      console.info(formatted);
    }
  }

  warn(component: string, message: string, data?: unknown): void {
    if (!this.shouldLog(LogLevel.WARN)) return;
    this.addLog(LogLevel.WARN, component, message, data);
    const formatted = this.formatLog(LogLevel.WARN, component, message);
    if (data !== undefined) {
      console.warn(formatted, data);
    } else {
      console.warn(formatted);
    }
  }

  error(component: string, message: string, data?: unknown): void {
    if (!this.shouldLog(LogLevel.ERROR)) return;
    this.addLog(LogLevel.ERROR, component, message, data);
    const formatted = this.formatLog(LogLevel.ERROR, component, message);
    if (data !== undefined) {
      console.error(formatted, data);
    } else {
      console.error(formatted);
    }
    try {
      void import('@/lib/startupLog').then(({ logStartupError }) => {
        logStartupError(`[${component}] ${message}`, data ?? message);
      });
    } catch {
      /* startup log optional during early boot */
    }
  }

  getLogs(): LogEntry[] {
    return [...this.logs];
  }

  clearLogs(): void {
    this.logs = [];
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Convenience functions for common logging patterns
export function logDebug(component: string, message: string, data?: unknown): void {
  logger.debug(component, message, data);
}

export function logInfo(component: string, message: string, data?: unknown): void {
  logger.info(component, message, data);
}

export function logWarn(component: string, message: string, data?: unknown): void {
  logger.warn(component, message, data);
}

export function logError(component: string, message: string, data?: unknown): void {
  logger.error(component, message, data);
}
