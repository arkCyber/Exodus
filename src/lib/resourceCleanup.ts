/**
 * Resource cleanup utilities for preventing memory leaks
 */

import { shellLog } from '@/lib/diagnosticLog';
import { onUnmounted } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

/**
 * Resource manager for tracking and cleaning up resources
 */
export class ResourceManager {
  private eventListeners: UnlistenFn[] = [];
  private timers: number[] = [];
  private intervals: number[] = [];
  private abortControllers: AbortController[] = [];
  private cleanupFunctions: (() => void)[] = [];

  /**
   * Register an event listener for cleanup
   */
  addListener(unlisten: UnlistenFn): void {
    this.eventListeners.push(unlisten);
  }

  /**
   * Register a timeout for cleanup
   */
  addTimeout(timerId: number): void {
    this.timers.push(timerId);
  }

  /**
   * Register an interval for cleanup
   */
  addInterval(intervalId: number): void {
    this.intervals.push(intervalId);
  }

  /**
   * Register an AbortController for cleanup
   */
  addAbortController(controller: AbortController): void {
    this.abortControllers.push(controller);
  }

  /**
   * Register a custom cleanup function
   */
  addCleanup(fn: () => void): void {
    this.cleanupFunctions.push(fn);
  }

  /**
   * Create a managed setTimeout
   */
  setTimeout(callback: () => void, delay: number): number {
    const timerId = window.setTimeout(callback, delay);
    this.addTimeout(timerId);
    return timerId;
  }

  /**
   * Create a managed setInterval
   */
  setInterval(callback: () => void, delay: number): number {
    const intervalId = window.setInterval(callback, delay);
    this.addInterval(intervalId);
    return intervalId;
  }

  /**
   * Create a managed AbortController
   */
  createAbortController(): AbortController {
    const controller = new AbortController();
    this.addAbortController(controller);
    return controller;
  }

  /**
   * Clean up all registered resources
   */
  cleanup(): void {
    // Clean up event listeners
    for (const unlisten of this.eventListeners) {
      try {
        unlisten();
      } catch (e) {
        shellLog.error('Failed to unlisten', e);
      }
    }
    this.eventListeners.length = 0;

    // Clear all timers
    for (const timerId of this.timers) {
      try {
        clearTimeout(timerId);
      } catch (e) {
        shellLog.error('Failed to clear timeout', e);
      }
    }
    this.timers.length = 0;

    // Clear all intervals
    for (const intervalId of this.intervals) {
      try {
        clearInterval(intervalId);
      } catch (e) {
        shellLog.error('Failed to clear interval', e);
      }
    }
    this.intervals.length = 0;

    // Abort all controllers
    for (const controller of this.abortControllers) {
      try {
        if (!controller.signal.aborted) {
          controller.abort();
        }
      } catch (e) {
        shellLog.error('Failed to abort controller', e);
      }
    }
    this.abortControllers.length = 0;

    // Run custom cleanup functions
    for (const fn of this.cleanupFunctions) {
      try {
        fn();
      } catch (e) {
        shellLog.error('Cleanup function failed', e);
      }
    }
    this.cleanupFunctions.length = 0;
  }

  /**
   * Get cleanup statistics
   */
  getStats(): {
    listeners: number;
    timers: number;
    intervals: number;
    controllers: number;
    cleanups: number;
  } {
    return {
      listeners: this.eventListeners.length,
      timers: this.timers.length,
      intervals: this.intervals.length,
      controllers: this.abortControllers.length,
      cleanups: this.cleanupFunctions.length,
    };
  }
}

/**
 * Create a resource manager that automatically cleans up on component unmount
 */
export function useResourceManager(): ResourceManager {
  const manager = new ResourceManager();
  
  onUnmounted(() => {
    const stats = manager.getStats();
    shellLog.info('Cleaning up resources', stats);
    manager.cleanup();
  });

  return manager;
}

/**
 * Helper to create a managed timeout that auto-cleans on unmount
 */
export function useManagedTimeout(callback: () => void, delay: number): void {
  const manager = useResourceManager();
  manager.setTimeout(callback, delay);
}

/**
 * Helper to create a managed interval that auto-cleans on unmount
 */
export function useManagedInterval(callback: () => void, delay: number): void {
  const manager = useResourceManager();
  manager.setInterval(callback, delay);
}
