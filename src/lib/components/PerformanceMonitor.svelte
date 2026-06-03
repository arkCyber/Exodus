<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface MetricValue {
    metric_type: 'Counter' | 'Gauge' | 'Histogram' | 'Summary';
    value: number;
    timestamp: number;
    labels: Record<string, string>;
  }

  interface Metric {
    name: string;
    metric_type: 'Counter' | 'Gauge' | 'Histogram' | 'Summary';
    help: string;
    values: MetricValue[];
  }

  interface MetricsStats {
    total_metrics: number;
    total_histograms: number;
    total_data_points: number;
  }

  interface TabSleepStats {
    total_tabs: number;
    active_tabs: number;
    sleeping_tabs: number;
    pinned_tabs: number;
    total_memory_mb: number;
    active_memory_mb: number;
    saved_memory_mb: number;
  }

  interface CircuitBreakerStats {
    state: 'Closed' | 'Open' | 'HalfOpen';
    failure_count: number;
    success_count: number;
    last_failure_time: number | null;
    last_state_change: number;
    total_calls: number;
    rejected_calls: number;
  }

  interface ServiceDiscoveryStats {
    total_services: number;
    total_endpoints: number;
    active_endpoints: number;
    stale_endpoints: number;
  }

  interface TracingStats {
    total_traces: number;
    total_spans: number;
    avg_duration_ms: number;
  }

  let metrics = $state<Metric[]>([]);
  let metricsStats = $state<MetricsStats | null>(null);
  let tabSleepStats = $state<TabSleepStats | null>(null);
  let circuitBreakerStats = $state<CircuitBreakerStats | null>(null);
  let serviceDiscoveryStats = $state<ServiceDiscoveryStats | null>(null);
  let tracingStats = $state<TracingStats | null>(null);
  let isLoading = $state(true);
  let autoRefresh = $state(true);
  let refreshInterval = $state(5000); // 5 seconds
  let selectedTab = $state<'overview' | 'metrics' | 'services' | 'traces' | 'tabs'>('overview');

  let refreshTimer: ReturnType<typeof setInterval> | undefined;

  async function loadAllStats() {
    isLoading = true;
    try {
      const [mStats, tStats] = await Promise.all([
        invoke<MetricsStats>('metrics_get_stats').catch(() => null),
        invoke<TabSleepStats>('tab_sleep_get_stats').catch(() => null),
      ]);

      metricsStats = mStats;
      tabSleepStats = tStats;
    } catch (error) {
      console.error('Failed to load stats:', error);
    } finally {
      isLoading = false;
    }
  }

  async function loadMetrics() {
    try {
      const allMetrics = await invoke<Metric[]>('metrics_get_all');
      metrics = allMetrics;
    } catch (error) {
      console.error('Failed to load metrics:', error);
    }
  }

  function startAutoRefresh() {
    if (refreshTimer) clearInterval(refreshTimer);
    if (autoRefresh) {
      refreshTimer = setInterval(() => {
        void loadAllStats();
        if (selectedTab === 'metrics') {
          void loadMetrics();
        }
      }, refreshInterval);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms.toFixed(0)}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  }

  function getStateColor(state: string): string {
    switch (state) {
      case 'Closed': return 'text-green-500';
      case 'Open': return 'text-red-500';
      case 'HalfOpen': return 'text-yellow-500';
      default: return 'text-gray-500';
    }
  }

  onMount(() => {
    void loadAllStats();
    startAutoRefresh();

    return () => {
      if (refreshTimer) clearInterval(refreshTimer);
    };
  });

  $effect(() => {
    startAutoRefresh();
  });
</script>

<div class="performance-monitor">
  <div class="monitor-header">
    <h2>性能监控</h2>
    <div class="header-controls">
      <label class="auto-refresh">
        <input type="checkbox" bind:checked={autoRefresh} />
        自动刷新
      </label>
      <select bind:value={refreshInterval} disabled={!autoRefresh}>
        <option value={1000}>1秒</option>
        <option value={5000}>5秒</option>
        <option value={10000}>10秒</option>
        <option value={30000}>30秒</option>
      </select>
      <button class="btn-refresh" onclick={() => void loadAllStats()}>
        刷新
      </button>
    </div>
  </div>

  <div class="monitor-tabs">
    <button
      class="tab-btn"
      class:active={selectedTab === 'overview'}
      onclick={() => selectedTab = 'overview'}
    >
      概览
    </button>
    <button
      class="tab-btn"
      class:active={selectedTab === 'metrics'}
      onclick={() => { selectedTab = 'metrics'; void loadMetrics(); }}
    >
      指标
    </button>
    <button
      class="tab-btn"
      class:active={selectedTab === 'services'}
      onclick={() => selectedTab = 'services'}
    >
      服务
    </button>
    <button
      class="tab-btn"
      class:active={selectedTab === 'traces'}
      onclick={() => selectedTab = 'traces'}
    >
      追踪
    </button>
    <button
      class="tab-btn"
      class:active={selectedTab === 'tabs'}
      onclick={() => selectedTab = 'tabs'}
    >
      标签页
    </button>
  </div>

  <div class="monitor-content">
    {#if isLoading}
      <div class="loading">加载中...</div>
    {:else if selectedTab === 'overview'}
      <div class="overview-grid">
        <!-- Metrics Card -->
        {#if metricsStats}
          <div class="stat-card">
            <h3>📊 指标统计</h3>
            <div class="stat-row">
              <span>总指标数:</span>
              <strong>{metricsStats.total_metrics}</strong>
            </div>
            <div class="stat-row">
              <span>直方图:</span>
              <strong>{metricsStats.total_histograms}</strong>
            </div>
            <div class="stat-row">
              <span>数据点:</span>
              <strong>{metricsStats.total_data_points.toLocaleString()}</strong>
            </div>
          </div>
        {/if}

        <!-- Tab Sleep Card -->
        {#if tabSleepStats}
          <div class="stat-card">
            <h3>💤 标签页休眠</h3>
            <div class="stat-row">
              <span>总标签页:</span>
              <strong>{tabSleepStats.total_tabs}</strong>
            </div>
            <div class="stat-row">
              <span>活跃:</span>
              <strong class="text-green-500">{tabSleepStats.active_tabs}</strong>
            </div>
            <div class="stat-row">
              <span>休眠:</span>
              <strong class="text-blue-500">{tabSleepStats.sleeping_tabs}</strong>
            </div>
            <div class="stat-row">
              <span>固定:</span>
              <strong class="text-yellow-500">{tabSleepStats.pinned_tabs}</strong>
            </div>
            <div class="stat-row highlight">
              <span>节省内存:</span>
              <strong class="text-green-600">
                {formatBytes(tabSleepStats.saved_memory_mb * 1024 * 1024)}
              </strong>
            </div>
          </div>
        {/if}

        <!-- Circuit Breaker Card -->
        {#if circuitBreakerStats}
          <div class="stat-card">
            <h3>⚡ 熔断器</h3>
            <div class="stat-row">
              <span>状态:</span>
              <strong class={getStateColor(circuitBreakerStats.state)}>
                {circuitBreakerStats.state}
              </strong>
            </div>
            <div class="stat-row">
              <span>总调用:</span>
              <strong>{circuitBreakerStats.total_calls}</strong>
            </div>
            <div class="stat-row">
              <span>拒绝:</span>
              <strong class="text-red-500">{circuitBreakerStats.rejected_calls}</strong>
            </div>
            <div class="stat-row">
              <span>失败:</span>
              <strong>{circuitBreakerStats.failure_count}</strong>
            </div>
          </div>
        {/if}

        <!-- Service Discovery Card -->
        {#if serviceDiscoveryStats}
          <div class="stat-card">
            <h3>🔍 服务发现</h3>
            <div class="stat-row">
              <span>服务数:</span>
              <strong>{serviceDiscoveryStats.total_services}</strong>
            </div>
            <div class="stat-row">
              <span>端点数:</span>
              <strong>{serviceDiscoveryStats.total_endpoints}</strong>
            </div>
            <div class="stat-row">
              <span>活跃:</span>
              <strong class="text-green-500">{serviceDiscoveryStats.active_endpoints}</strong>
            </div>
            <div class="stat-row">
              <span>过期:</span>
              <strong class="text-red-500">{serviceDiscoveryStats.stale_endpoints}</strong>
            </div>
          </div>
        {/if}

        <!-- Tracing Card -->
        {#if tracingStats}
          <div class="stat-card">
            <h3>🔬 分布式追踪</h3>
            <div class="stat-row">
              <span>追踪数:</span>
              <strong>{tracingStats.total_traces}</strong>
            </div>
            <div class="stat-row">
              <span>Span 数:</span>
              <strong>{tracingStats.total_spans}</strong>
            </div>
            <div class="stat-row">
              <span>平均耗时:</span>
              <strong>{formatDuration(tracingStats.avg_duration_ms)}</strong>
            </div>
          </div>
        {/if}
      </div>
    {:else if selectedTab === 'metrics'}
      <div class="metrics-list">
        {#if metrics.length === 0}
          <p class="empty-state">暂无指标数据</p>
        {:else}
          {#each metrics as metric}
            <div class="metric-card">
              <div class="metric-header">
                <h4>{metric.name}</h4>
                <span class="metric-type">{metric.metric_type}</span>
              </div>
              <p class="metric-help">{metric.help}</p>
              <div class="metric-values">
                {#each metric.values.slice(-5) as value}
                  <div class="metric-value">
                    <span class="value">{value.value.toFixed(2)}</span>
                    <span class="timestamp">
                      {new Date(value.timestamp * 1000).toLocaleTimeString()}
                    </span>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {:else if selectedTab === 'tabs'}
      {#if tabSleepStats}
        <div class="tabs-detail">
          <div class="memory-chart">
            <h3>内存使用情况</h3>
            <div class="chart-bar">
              <div
                class="bar active"
                style="width: {(tabSleepStats.active_memory_mb / tabSleepStats.total_memory_mb) * 100}%"
              >
                <span>活跃: {formatBytes(tabSleepStats.active_memory_mb * 1024 * 1024)}</span>
              </div>
              <div
                class="bar saved"
                style="width: {(tabSleepStats.saved_memory_mb / tabSleepStats.total_memory_mb) * 100}%"
              >
                <span>节省: {formatBytes(tabSleepStats.saved_memory_mb * 1024 * 1024)}</span>
              </div>
            </div>
          </div>

          <div class="tab-distribution">
            <h3>标签页分布</h3>
            <div class="pie-stats">
              <div class="pie-item">
                <div class="pie-color active"></div>
                <span>活跃: {tabSleepStats.active_tabs}</span>
              </div>
              <div class="pie-item">
                <div class="pie-color sleeping"></div>
                <span>休眠: {tabSleepStats.sleeping_tabs}</span>
              </div>
              <div class="pie-item">
                <div class="pie-color pinned"></div>
                <span>固定: {tabSleepStats.pinned_tabs}</span>
              </div>
            </div>
          </div>
        </div>
      {/if}
    {:else}
      <p class="empty-state">功能开发中...</p>
    {/if}
  </div>
</div>

<style>
  .performance-monitor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
  }

  .monitor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .monitor-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .header-controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }

  .auto-refresh {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
  }

  .auto-refresh input[type="checkbox"] {
    cursor: pointer;
  }

  select {
    padding: 0.375rem 0.75rem;
    background: var(--bg-secondary, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    cursor: pointer;
  }

  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-refresh {
    padding: 0.375rem 1rem;
    background: var(--accent-color, #007bff);
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
  }

  .btn-refresh:hover {
    background: var(--accent-hover, #0056b3);
  }

  .monitor-tabs {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: var(--bg-secondary, #2a2a2a);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .tab-btn {
    padding: 0.5rem 1rem;
    background: transparent;
    color: var(--text-secondary, #999);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    background: var(--bg-hover, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .tab-btn.active {
    background: var(--accent-color, #007bff);
    color: white;
  }

  .monitor-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: var(--text-secondary, #999);
  }

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-secondary, #999);
  }

  .overview-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
  }

  .stat-card {
    background: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
  }

  .stat-card h3 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .stat-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color, #444);
  }

  .stat-row:last-child {
    border-bottom: none;
  }

  .stat-row.highlight {
    background: rgba(0, 123, 255, 0.1);
    padding: 0.75rem;
    margin: 0.5rem -1rem -1rem -1rem;
    border-radius: 0 0 8px 8px;
  }

  .stat-row span {
    color: var(--text-secondary, #999);
  }

  .stat-row strong {
    font-weight: 600;
  }

  .text-green-500 { color: #22c55e; }
  .text-green-600 { color: #16a34a; }
  .text-blue-500 { color: #3b82f6; }
  .text-yellow-500 { color: #eab308; }
  .text-red-500 { color: #ef4444; }
  .text-gray-500 { color: #6b7280; }

  .metrics-list {
    display: grid;
    gap: 1rem;
  }

  .metric-card {
    background: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
  }

  .metric-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .metric-header h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .metric-type {
    padding: 0.25rem 0.5rem;
    background: var(--accent-color, #007bff);
    color: white;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .metric-help {
    margin: 0 0 1rem 0;
    color: var(--text-secondary, #999);
    font-size: 0.875rem;
  }

  .metric-values {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .metric-value {
    display: flex;
    flex-direction: column;
    padding: 0.5rem;
    background: var(--bg-tertiary, #1a1a1a);
    border-radius: 4px;
    min-width: 100px;
  }

  .metric-value .value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--accent-color, #007bff);
  }

  .metric-value .timestamp {
    font-size: 0.75rem;
    color: var(--text-secondary, #999);
  }

  .tabs-detail {
    display: grid;
    gap: 2rem;
  }

  .memory-chart h3,
  .tab-distribution h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .chart-bar {
    display: flex;
    height: 60px;
    background: var(--bg-secondary, #2a2a2a);
    border-radius: 8px;
    overflow: hidden;
  }

  .chart-bar .bar {
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: 500;
    font-size: 0.875rem;
    transition: width 0.3s;
  }

  .chart-bar .bar.active {
    background: #22c55e;
  }

  .chart-bar .bar.saved {
    background: #3b82f6;
  }

  .pie-stats {
    display: flex;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .pie-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .pie-color {
    width: 16px;
    height: 16px;
    border-radius: 4px;
  }

  .pie-color.active {
    background: #22c55e;
  }

  .pie-color.sleeping {
    background: #3b82f6;
  }

  .pie-color.pinned {
    background: #eab308;
  }
</style>
