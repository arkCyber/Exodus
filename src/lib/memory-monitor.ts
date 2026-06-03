/**
 * Aerospace-grade memory leak detection
 * Provides comprehensive memory monitoring and leak detection
 */

export interface MemorySnapshot {
  timestamp: number;
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
  componentCount: number;
  eventListenerCount: number;
  intervalCount: number;
  timeoutCount: number;
}

export interface MemoryLeakAlert {
  timestamp: number;
  type: 'heap_growth' | 'component_leak' | 'listener_leak' | 'timer_leak';
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  details: Record<string, unknown>;
}

class MemoryMonitor {
  private snapshots: MemorySnapshot[] = [];
  private alerts: MemoryLeakAlert[] = [];
  private monitoringInterval: number | null = null;
  private readonly MONITOR_INTERVAL_MS = 5000; // 5 seconds
  private readonly MAX_SNAPSHOTS = 100;
  private baseline: MemorySnapshot | null = null;
  private componentRegistry: Set<string> = new Set();
  private listenerRegistry: Map<string, number> = new Map();
  private timerRegistry: Map<string, number> = new Map();

  constructor() {
    this.startMonitoring();
  }

  /**
   * Start memory monitoring
   */
  startMonitoring(): void {
    if (this.monitoringInterval) {
      return;
    }

    this.monitoringInterval = window.setInterval(() => {
      this.takeSnapshot();
    }, this.MONITOR_INTERVAL_MS);

    // Set baseline after first snapshot
    setTimeout(() => {
      if (this.snapshots.length > 0) {
        this.baseline = { ...this.snapshots[0] };
      }
    }, this.MONITOR_INTERVAL_MS);
  }

  /**
   * Stop memory monitoring
   */
  stopMonitoring(): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
  }

  /**
   * Take a memory snapshot
   */
  private takeSnapshot(): void {
    const perf = performance as any;
    const memory = perf.memory;

    const snapshot: MemorySnapshot = {
      timestamp: Date.now(),
      usedJSHeapSize: memory?.usedJSHeapSize || 0,
      totalJSHeapSize: memory?.totalJSHeapSize || 0,
      jsHeapSizeLimit: memory?.jsHeapSizeLimit || 0,
      componentCount: this.componentRegistry.size,
      eventListenerCount: this.getEventListenerCount(),
      intervalCount: this.getIntervalCount(),
      timeoutCount: this.getTimeoutCount(),
    };

    this.snapshots.push(snapshot);

    // Maintain max snapshots
    if (this.snapshots.length > this.MAX_SNAPSHOTS) {
      this.snapshots.shift();
    }

    // Analyze for leaks
    this.analyzeMemory(snapshot);
  }

  /**
   * Analyze memory for potential leaks
   */
  private analyzeMemory(current: MemorySnapshot): void {
    if (!this.baseline) {
      return;
    }

    const timeDiff = current.timestamp - this.baseline.timestamp;
    const heapGrowth = current.usedJSHeapSize - this.baseline.usedJSHeapSize;
    const growthRate = heapGrowth / timeDiff; // bytes per ms

    // Check for heap growth leak (growth > 1MB/min)
    if (growthRate > 16.67) {
      this.addAlert({
        timestamp: Date.now(),
        type: 'heap_growth',
        severity: growthRate > 100 ? 'critical' : growthRate > 50 ? 'high' : 'medium',
        message: `Heap memory growing at ${(growthRate * 60 * 1000 / 1024 / 1024).toFixed(2)} MB/min`,
        details: {
          baseline: this.baseline.usedJSHeapSize,
          current: current.usedJSHeapSize,
          growth: heapGrowth,
          growthRate,
        },
      });
    }

    // Check for component leak
    if (current.componentCount > this.baseline.componentCount * 2) {
      this.addAlert({
        timestamp: Date.now(),
        type: 'component_leak',
        severity: 'high',
        message: `Component count doubled: ${this.baseline.componentCount} → ${current.componentCount}`,
        details: {
          baseline: this.baseline.componentCount,
          current: current.componentCount,
        },
      });
    }

    // Check for event listener leak
    if (current.eventListenerCount > this.baseline.eventListenerCount * 3) {
      this.addAlert({
        timestamp: Date.now(),
        type: 'listener_leak',
        severity: 'high',
        message: `Event listener count tripled: ${this.baseline.eventListenerCount} → ${current.eventListenerCount}`,
        details: {
          baseline: this.baseline.eventListenerCount,
          current: current.eventListenerCount,
        },
      });
    }

    // Check for timer leak
    const timerGrowth = (current.intervalCount + current.timeoutCount) - 
                      (this.baseline.intervalCount + this.baseline.timeoutCount);
    if (timerGrowth > 50) {
      this.addAlert({
        timestamp: Date.now(),
        type: 'timer_leak',
        severity: 'medium',
        message: `Timer count increased by ${timerGrowth}`,
        details: {
          baseline: this.baseline.intervalCount + this.baseline.timeoutCount,
          current: current.intervalCount + current.timeoutCount,
          growth: timerGrowth,
        },
      });
    }
  }

  /**
   * Add a memory leak alert
   */
  private addAlert(alert: MemoryLeakAlert): void {
    this.alerts.push(alert);

    // Keep only last 100 alerts
    if (this.alerts.length > 100) {
      this.alerts.shift();
    }

    // Log critical alerts
    if (alert.severity === 'critical') {
      console.error('[MemoryMonitor] CRITICAL:', alert.message, alert.details);
    }
  }

  /**
   * Register a component
   */
  registerComponent(componentId: string): void {
    this.componentRegistry.add(componentId);
  }

  /**
   * Unregister a component
   */
  unregisterComponent(componentId: string): void {
    this.componentRegistry.delete(componentId);
  }

  /**
   * Register an event listener
   */
  registerListener(target: string, event: string): void {
    const key = `${target}:${event}`;
    this.listenerRegistry.set(key, (this.listenerRegistry.get(key) || 0) + 1);
  }

  /**
   * Unregister an event listener
   */
  unregisterListener(target: string, event: string): void {
    const key = `${target}:${event}`;
    const count = this.listenerRegistry.get(key) || 0;
    if (count > 1) {
      this.listenerRegistry.set(key, count - 1);
    } else {
      this.listenerRegistry.delete(key);
    }
  }

  /**
   * Register a timer
   */
  registerTimer(timerId: number, type: 'interval' | 'timeout'): void {
    this.timerRegistry.set(`${type}:${timerId}`, Date.now());
  }

  /**
   * Unregister a timer
   */
  unregisterTimer(timerId: number, type: 'interval' | 'timeout'): void {
    this.timerRegistry.delete(`${type}:${timerId}`);
  }

  /**
   * Get event listener count (estimated)
   */
  private getEventListenerCount(): number {
    let count = 0;
    for (const value of this.listenerRegistry.values()) {
      count += value;
    }
    return count;
  }

  /**
   * Get interval count
   */
  private getIntervalCount(): number {
    let count = 0;
    for (const key of this.timerRegistry.keys()) {
      if (key.startsWith('interval:')) {
        count++;
      }
    }
    return count;
  }

  /**
   * Get timeout count
   */
  private getTimeoutCount(): number {
    let count = 0;
    for (const key of this.timerRegistry.keys()) {
      if (key.startsWith('timeout:')) {
        count++;
      }
    }
    return count;
  }

  /**
   * Get current memory usage
   */
  getCurrentMemoryUsage(): {
    used: number;
    total: number;
    limit: number;
    percentage: number;
  } | null {
    const perf = performance as any;
    const memory = perf.memory;

    if (!memory) {
      return null;
    }

    return {
      used: memory.usedJSHeapSize,
      total: memory.totalJSHeapSize,
      limit: memory.jsHeapSizeLimit,
      percentage: (memory.usedJSHeapSize / memory.jsHeapSizeLimit) * 100,
    };
  }

  /**
   * Get memory snapshots
   */
  getSnapshots(limit: number = 50): MemorySnapshot[] {
    return this.snapshots.slice(-limit);
  }

  /**
   * Get memory leak alerts
   */
  getAlerts(limit: number = 50): MemoryLeakAlert[] {
    return this.alerts.slice(-limit);
  }

  /**
   * Get memory statistics
   */
  getStatistics(): {
    currentUsage: {
      used: number;
      total: number;
      limit: number;
      percentage: number;
    } | null;
    snapshots: number;
    alerts: number;
    componentCount: number;
    listenerCount: number;
    timerCount: number;
  } {
    return {
      currentUsage: this.getCurrentMemoryUsage(),
      snapshots: this.snapshots.length,
      alerts: this.alerts.length,
      componentCount: this.componentRegistry.size,
      listenerCount: this.getEventListenerCount(),
      timerCount: this.getIntervalCount() + this.getTimeoutCount(),
    };
  }

  /**
   * Clear alerts
   */
  clearAlerts(): void {
    this.alerts = [];
  }

  /**
   * Reset baseline
   */
  resetBaseline(): void {
    if (this.snapshots.length > 0) {
      this.baseline = { ...this.snapshots[this.snapshots.length - 1] };
    }
  }

  /**
   * Force garbage collection (if available)
   */
  forceGC(): boolean {
    if (typeof (window as any).gc === 'function') {
      (window as any).gc();
      return true;
    }
    return false;
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.stopMonitoring();
    this.snapshots = [];
    this.alerts = [];
    this.componentRegistry.clear();
    this.listenerRegistry.clear();
    this.timerRegistry.clear();
    this.baseline = null;
  }
}

// Singleton instance
let memoryMonitorInstance: MemoryMonitor | null = null;

export function getMemoryMonitor(): MemoryMonitor {
  if (!memoryMonitorInstance) {
    memoryMonitorInstance = new MemoryMonitor();
  }
  return memoryMonitorInstance;
}

export function destroyMemoryMonitor(): void {
  if (memoryMonitorInstance) {
    memoryMonitorInstance.destroy();
    memoryMonitorInstance = null;
  }
}
