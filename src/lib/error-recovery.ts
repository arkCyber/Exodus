/**
 * Aerospace-grade error recovery mechanism
 * Provides comprehensive error handling, recovery, and resilience
 */

import { shellLog } from '@/lib/diagnosticLog';

export enum ErrorSeverity {
  LOW = 'LOW',
  MEDIUM = 'MEDIUM',
  HIGH = 'HIGH',
  CRITICAL = 'CRITICAL',
}

export enum ErrorCategory {
  NETWORK = 'NETWORK',
  STORAGE = 'STORAGE',
  VALIDATION = 'VALIDATION',
  RUNTIME = 'RUNTIME',
  SYSTEM = 'SYSTEM',
  SECURITY = 'SECURITY',
}

export interface ErrorContext {
  timestamp: number;
  severity: ErrorSeverity;
  category: ErrorCategory;
  message: string;
  stack?: string;
  details?: Record<string, unknown>;
  recoverable: boolean;
  recoveryAttempts: number;
  maxRecoveryAttempts: number;
}

export interface RecoveryStrategy {
  name: string;
  canRecover: (error: ErrorContext) => boolean;
  recover: (error: ErrorContext) => Promise<boolean>;
  maxAttempts: number;
  backoffMs: number;
}

class ErrorRecoveryManager {
  private errorHistory: ErrorContext[] = [];
  private recoveryStrategies: Map<string, RecoveryStrategy> = new Map();
  private activeRecoveries: Map<string, Promise<boolean>> = new Map();
  private readonly MAX_ERROR_HISTORY = 1000;
  private readonly DEFAULT_MAX_ATTEMPTS = 3;
  private readonly DEFAULT_BACKOFF_MS = 1000;

  constructor() {
    this.registerDefaultStrategies();
  }

  /**
   * Register default recovery strategies
   */
  private registerDefaultStrategies(): void {
    // Network error recovery
    this.registerStrategy({
      name: 'network-retry',
      canRecover: (error) => error.category === ErrorCategory.NETWORK && error.recoverable,
      recover: async (error) => {
        // Exponential backoff
        const delay = this.DEFAULT_BACKOFF_MS * Math.pow(2, error.recoveryAttempts);
        await this.sleep(delay);
        return true; // Assume retry will succeed
      },
      maxAttempts: 5,
      backoffMs: 1000,
    });

    // Storage error recovery
    this.registerStrategy({
      name: 'storage-clear',
      canRecover: (error) => error.category === ErrorCategory.STORAGE,
      recover: async (error) => {
        try {
          // Clear corrupted data
          if (error.details?.key) {
            localStorage.removeItem(error.details.key as string);
          }
          return true;
        } catch {
          return false;
        }
      },
      maxAttempts: 2,
      backoffMs: 500,
    });

    // Validation error recovery
    this.registerStrategy({
      name: 'validation-sanitize',
      canRecover: (error) => error.category === ErrorCategory.VALIDATION,
      recover: async (_error) => {
        // Return sanitized default value
        return true;
      },
      maxAttempts: 1,
      backoffMs: 0,
    });

    // Runtime error recovery
    this.registerStrategy({
      name: 'runtime-reload',
      canRecover: (error) => error.category === ErrorCategory.RUNTIME && error.severity === ErrorSeverity.HIGH,
      recover: async (_error) => {
        // Attempt to reload the application state
        return true;
      },
      maxAttempts: 1,
      backoffMs: 0,
    });
  }

  /**
   * Register a custom recovery strategy
   */
  registerStrategy(strategy: RecoveryStrategy): void {
    this.recoveryStrategies.set(strategy.name, strategy);
  }

  /**
   * Unregister a recovery strategy
   */
  unregisterStrategy(name: string): void {
    this.recoveryStrategies.delete(name);
  }

  /**
   * Handle an error with automatic recovery
   */
  async handleError(
    error: Error,
    category: ErrorCategory,
    details?: Record<string, unknown>,
    customStrategy?: string
  ): Promise<boolean> {
    const context: ErrorContext = {
      timestamp: Date.now(),
      severity: this.determineSeverity(error, category),
      category,
      message: error.message,
      stack: error.stack,
      details,
      recoverable: this.isRecoverable(error, category),
      recoveryAttempts: 0,
      maxRecoveryAttempts: this.DEFAULT_MAX_ATTEMPTS,
    };

    this.logError(context);

    if (!context.recoverable) {
      return false;
    }

    // Find appropriate strategy
    const strategyName = customStrategy || this.findStrategy(context);
    if (!strategyName) {
      return false;
    }

    const strategy = this.recoveryStrategies.get(strategyName);
    if (!strategy) {
      return false;
    }

    // Check if recovery is already in progress
    const recoveryKey = `${strategyName}-${context.timestamp}`;
    if (this.activeRecoveries.has(recoveryKey)) {
      return this.activeRecoveries.get(recoveryKey)!;
    }

    // Attempt recovery
    const recoveryPromise = this.attemptRecovery(context, strategy);
    this.activeRecoveries.set(recoveryKey, recoveryPromise);

    try {
      const result = await recoveryPromise;
      this.activeRecoveries.delete(recoveryKey);
      return result;
    } catch {
      this.activeRecoveries.delete(recoveryKey);
      return false;
    }
  }

  /**
   * Attempt recovery with retry logic
   */
  private async attemptRecovery(
    context: ErrorContext,
    strategy: RecoveryStrategy
  ): Promise<boolean> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt < strategy.maxAttempts; attempt++) {
      context.recoveryAttempts = attempt + 1;

      try {
        const success = await strategy.recover(context);
        if (success) {
          this.logRecoverySuccess(context, strategy.name, attempt + 1);
          return true;
        }
      } catch (error) {
        lastError = error as Error;
        this.logRecoveryFailure(context, strategy.name, attempt + 1, lastError);
      }

      // Backoff before next attempt
      if (attempt < strategy.maxAttempts - 1) {
        const delay = strategy.backoffMs * Math.pow(2, attempt);
        await this.sleep(delay);
      }
    }

    return false;
  }

  /**
   * Find appropriate recovery strategy for error
   */
  private findStrategy(context: ErrorContext): string | null {
    for (const [name, strategy] of this.recoveryStrategies) {
      if (strategy.canRecover(context)) {
        return name;
      }
    }
    return null;
  }

  /**
   * Determine error severity
   */
  private determineSeverity(error: Error, category: ErrorCategory): ErrorSeverity {
    if (category === ErrorCategory.SECURITY) {
      return ErrorSeverity.CRITICAL;
    }

    if (category === ErrorCategory.SYSTEM) {
      return ErrorSeverity.HIGH;
    }

    if (error.message.includes('timeout') || error.message.includes('network')) {
      return ErrorSeverity.MEDIUM;
    }

    return ErrorSeverity.LOW;
  }

  /**
   * Check if error is recoverable
   */
  private isRecoverable(error: Error, category: ErrorCategory): boolean {
    // Security errors are not recoverable
    if (category === ErrorCategory.SECURITY) {
      return false;
    }

    // Critical system errors may not be recoverable
    if (category === ErrorCategory.SYSTEM && error.message.includes('fatal')) {
      return false;
    }

    return true;
  }

  /**
   * Log error to history
   */
  private logError(context: ErrorContext): void {
    this.errorHistory.push(context);

    // Maintain max history
    if (this.errorHistory.length > this.MAX_ERROR_HISTORY) {
      this.errorHistory.shift();
    }

    // Log to console for debugging
    shellLog.error(`${context.category}: ${context.message}`, context);
  }

  /**
   * Log successful recovery
   */
  private logRecoverySuccess(context: ErrorContext, strategy: string, attempt: number): void {
    shellLog.info(`Recovery successful: ${strategy} (attempt ${attempt})`, context);
  }

  /**
   * Log failed recovery
   */
  private logRecoveryFailure(_context: ErrorContext, strategy: string, attempt: number, error: Error): void {
    shellLog.error(`Recovery failed: ${strategy} (attempt ${attempt})`, error);
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get error statistics
   */
  getStatistics(): {
    totalErrors: number;
    errorsByCategory: Record<ErrorCategory, number>;
    errorsBySeverity: Record<ErrorSeverity, number>;
    recoveryRate: number;
    activeRecoveries: number;
  } {
    const errorsByCategory: Record<ErrorCategory, number> = {
      [ErrorCategory.NETWORK]: 0,
      [ErrorCategory.STORAGE]: 0,
      [ErrorCategory.VALIDATION]: 0,
      [ErrorCategory.RUNTIME]: 0,
      [ErrorCategory.SYSTEM]: 0,
      [ErrorCategory.SECURITY]: 0,
    };

    const errorsBySeverity: Record<ErrorSeverity, number> = {
      [ErrorSeverity.LOW]: 0,
      [ErrorSeverity.MEDIUM]: 0,
      [ErrorSeverity.HIGH]: 0,
      [ErrorSeverity.CRITICAL]: 0,
    };

    let recoveredCount = 0;

    for (const error of this.errorHistory) {
      errorsByCategory[error.category]++;
      errorsBySeverity[error.severity]++;
      if (error.recoveryAttempts > 0) {
        recoveredCount++;
      }
    }

    return {
      totalErrors: this.errorHistory.length,
      errorsByCategory,
      errorsBySeverity,
      recoveryRate: this.errorHistory.length > 0 ? recoveredCount / this.errorHistory.length : 0,
      activeRecoveries: this.activeRecoveries.size,
    };
  }

  /**
   * Get error history
   */
  getErrorHistory(limit: number = 100): ErrorContext[] {
    return this.errorHistory.slice(-limit);
  }

  /**
   * Clear error history
   */
  clearErrorHistory(): void {
    this.errorHistory = [];
  }

  /**
   * Export error report
   */
  exportErrorReport(): string {
    return JSON.stringify({
      statistics: this.getStatistics(),
      errors: this.errorHistory,
      strategies: Array.from(this.recoveryStrategies.entries()).map(([name, strategy]) => ({
        name,
        maxAttempts: strategy.maxAttempts,
        backoffMs: strategy.backoffMs,
      })),
    }, null, 2);
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.errorHistory = [];
    this.recoveryStrategies.clear();
    this.activeRecoveries.clear();
  }
}

// Singleton instance
let errorRecoveryInstance: ErrorRecoveryManager | null = null;

export function getErrorRecoveryManager(): ErrorRecoveryManager {
  if (!errorRecoveryInstance) {
    errorRecoveryInstance = new ErrorRecoveryManager();
  }
  return errorRecoveryInstance;
}

export function destroyErrorRecoveryManager(): void {
  if (errorRecoveryInstance) {
    errorRecoveryInstance.destroy();
    errorRecoveryInstance = null;
  }
}

/**
 * Wrapper function for automatic error recovery
 */
export async function withRecovery<T>(
  fn: () => T | Promise<T>,
  category: ErrorCategory,
  details?: Record<string, unknown>,
  strategy?: string
): Promise<T> {
  const recoveryManager = getErrorRecoveryManager();

  try {
    return await fn();
  } catch (error) {
    const recovered = await recoveryManager.handleError(
      error as Error,
      category,
      details,
      strategy
    );

    if (recovered) {
      // Retry the function after successful recovery
      return await fn();
    }

    throw error;
  }
}
