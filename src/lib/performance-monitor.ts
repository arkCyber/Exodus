/**
 * Aerospace-grade performance monitoring
 * Provides comprehensive performance metrics and analysis
 */

export interface PerformanceMetric {
  name: string;
  value: number;
  unit: string;
  timestamp: number;
  category: 'render' | 'network' | 'memory' | 'custom';
}

export interface PerformanceMark {
  name: string;
  startTime: number;
  duration?: number;
  metadata?: Record<string, unknown>;
}

class PerformanceMonitor {
  private metrics: PerformanceMetric[] = [];
  private marks: Map<string, PerformanceMark> = new Map();
  private observers: PerformanceObserver[] = [];
  private readonly MAX_METRICS = 1000;

  constructor() {
    this.setupObservers();
  }

  /**
   * Setup performance observers
   */
  private setupObservers(): void {
    // Observe paint timing
    if ('PerformanceObserver' in window) {
      try {
        const paintObserver = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            this.recordMetric({
              name: entry.name,
              value: entry.startTime,
              unit: 'ms',
              timestamp: Date.now(),
              category: 'render',
            });
          }
        });
        paintObserver.observe({ entryTypes: ['paint'] });
        this.observers.push(paintObserver);
      } catch (e) {
        console.warn('Failed to setup paint observer:', e);
      }

      // Observe layout shift
      try {
        const layoutObserver = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            this.recordMetric({
              name: 'layout-shift',
              value: (entry as any).value || 0,
              unit: 'score',
              timestamp: Date.now(),
              category: 'render',
            });
          }
        });
        layoutObserver.observe({ entryTypes: ['layout-shift'] });
        this.observers.push(layoutObserver);
      } catch (e) {
        console.warn('Failed to setup layout shift observer:', e);
      }

      // Observe largest contentful paint
      try {
        const lcpObserver = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            this.recordMetric({
              name: 'largest-contentful-paint',
              value: entry.startTime,
              unit: 'ms',
              timestamp: Date.now(),
              category: 'render',
            });
          }
        });
        lcpObserver.observe({ entryTypes: ['largest-contentful-paint'] });
        this.observers.push(lcpObserver);
      } catch (e) {
        console.warn('Failed to setup LCP observer:', e);
      }
    }
  }

  /**
   * Start a performance mark
   */
  startMark(name: string, metadata?: Record<string, unknown>): void {
    this.marks.set(name, {
      name,
      startTime: performance.now(),
      metadata,
    });
  }

  /**
   * End a performance mark
   */
  endMark(name: string): number | null {
    const mark = this.marks.get(name);
    if (!mark) {
      console.warn(`Mark "${name}" not found`);
      return null;
    }

    const duration = performance.now() - mark.startTime;
    mark.duration = duration;

    this.recordMetric({
      name,
      value: duration,
      unit: 'ms',
      timestamp: Date.now(),
      category: 'custom',
    });

    this.marks.delete(name);
    return duration;
  }

  /**
   * Record a custom metric
   */
  recordMetric(metric: PerformanceMetric): void {
    this.metrics.push(metric);

    // Maintain max metrics
    if (this.metrics.length > this.MAX_METRICS) {
      this.metrics.shift();
    }
  }

  /**
   * Measure function execution time
   */
  async measure<T>(
    name: string,
    fn: () => T | Promise<T>,
    metadata?: Record<string, unknown>
  ): Promise<T> {
    this.startMark(name, metadata);
    try {
      const result = await fn();
      this.endMark(name);
      return result;
    } catch (error) {
      this.endMark(name);
      throw error;
    }
  }

  /**
   * Get metrics by category
   */
  getMetricsByCategory(category: PerformanceMetric['category']): PerformanceMetric[] {
    return this.metrics.filter(m => m.category === category);
  }

  /**
   * Get metrics by name
   */
  getMetricsByName(name: string): PerformanceMetric[] {
    return this.metrics.filter(m => m.name === name);
  }

  /**
   * Get performance statistics
   */
  getStatistics(): {
    totalMetrics: number;
    metricsByCategory: Record<PerformanceMetric['category'], number>;
    averageRenderTime: number;
    averageNetworkTime: number;
    averageCustomTime: number;
    slowestOperations: Array<{ name: string; duration: number }>;
  } {
    const metricsByCategory: Record<PerformanceMetric['category'], number> = {
      render: 0,
      network: 0,
      memory: 0,
      custom: 0,
    };

    let totalRenderTime = 0;
    let renderCount = 0;
    let totalNetworkTime = 0;
    let networkCount = 0;
    let totalCustomTime = 0;
    let customCount = 0;

    const operations: Array<{ name: string; duration: number }> = [];

    for (const metric of this.metrics) {
      metricsByCategory[metric.category]++;

      if (metric.category === 'render') {
        totalRenderTime += metric.value;
        renderCount++;
      } else if (metric.category === 'network') {
        totalNetworkTime += metric.value;
        networkCount++;
      } else if (metric.category === 'custom') {
        totalCustomTime += metric.value;
        customCount++;
        operations.push({ name: metric.name, duration: metric.value });
      }
    }

    // Sort operations by duration
    operations.sort((a, b) => b.duration - a.duration);

    return {
      totalMetrics: this.metrics.length,
      metricsByCategory,
      averageRenderTime: renderCount > 0 ? totalRenderTime / renderCount : 0,
      averageNetworkTime: networkCount > 0 ? totalNetworkTime / networkCount : 0,
      averageCustomTime: customCount > 0 ? totalCustomTime / customCount : 0,
      slowestOperations: operations.slice(0, 10),
    };
  }

  /**
   * Get Web Vitals
   */
  getWebVitals(): {
    fcp?: number; // First Contentful Paint
    lcp?: number; // Largest Contentful Paint
    cls?: number; // Cumulative Layout Shift
    ttfb?: number; // Time to First Byte
  } {
    const vitals: {
      fcp?: number;
      lcp?: number;
      cls?: number;
      ttfb?: number;
    } = {};

    const paintEntries = performance.getEntriesByName('first-contentful-paint');
    if (paintEntries.length > 0) {
      vitals.fcp = paintEntries[0].startTime;
    }

    const lcpEntries = this.getMetricsByName('largest-contentful-paint');
    if (lcpEntries.length > 0) {
      vitals.lcp = lcpEntries[lcpEntries.length - 1].value;
    }

    const clsEntries = this.getMetricsByName('layout-shift');
    if (clsEntries.length > 0) {
      vitals.cls = clsEntries.reduce((sum, entry) => sum + entry.value, 0);
    }

    // Calculate TTFB from navigation timing
    const navEntry = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    if (navEntry) {
      vitals.ttfb = navEntry.responseStart - navEntry.requestStart;
    }

    return vitals;
  }

  /**
   * Get performance score (0-100)
   */
  getPerformanceScore(): number {
    const vitals = this.getWebVitals();
    let score = 100;

    // FCP penalty (good: <1.8s, needs improvement: <3s)
    if (vitals.fcp !== undefined) {
      if (vitals.fcp > 3000) {
        score -= 20;
      } else if (vitals.fcp > 1800) {
        score -= 10;
      }
    }

    // LCP penalty (good: <2.5s, needs improvement: <4s)
    if (vitals.lcp !== undefined) {
      if (vitals.lcp > 4000) {
        score -= 25;
      } else if (vitals.lcp > 2500) {
        score -= 15;
      }
    }

    // CLS penalty (good: <0.1, needs improvement: <0.25)
    if (vitals.cls !== undefined) {
      if (vitals.cls > 0.25) {
        score -= 25;
      } else if (vitals.cls > 0.1) {
        score -= 15;
      }
    }

    // TTFB penalty (good: <600ms, needs improvement: <1s)
    if (vitals.ttfb !== undefined) {
      if (vitals.ttfb > 1000) {
        score -= 15;
      } else if (vitals.ttfb > 600) {
        score -= 10;
      }
    }

    return Math.max(0, Math.min(100, score));
  }

  /**
   * Clear metrics
   */
  clearMetrics(): void {
    this.metrics = [];
  }

  /**
   * Export metrics as JSON
   */
  exportMetrics(): string {
    return JSON.stringify({
      metrics: this.metrics,
      statistics: this.getStatistics(),
      webVitals: this.getWebVitals(),
      performanceScore: this.getPerformanceScore(),
    }, null, 2);
  }

  /**
   * Cleanup
   */
  destroy(): void {
    for (const observer of this.observers) {
      observer.disconnect();
    }
    this.observers = [];
    this.metrics = [];
    this.marks.clear();
  }
}

// Singleton instance
let performanceMonitorInstance: PerformanceMonitor | null = null;

export function getPerformanceMonitor(): PerformanceMonitor {
  if (!performanceMonitorInstance) {
    performanceMonitorInstance = new PerformanceMonitor();
  }
  return performanceMonitorInstance;
}

export function destroyPerformanceMonitor(): void {
  if (performanceMonitorInstance) {
    performanceMonitorInstance.destroy();
    performanceMonitorInstance = null;
  }
}
