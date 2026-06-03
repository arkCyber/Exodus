<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import MetricsChart from './MetricsChart.svelte';

  interface DataPoint {
    timestamp: number;
    value: number;
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

  let memoryData = $state<DataPoint[]>([]);
  let cpuData = $state<DataPoint[]>([]);
  let tabCountData = $state<DataPoint[]>([]);
  let requestsData = $state<DataPoint[]>([]);
  
  let tabSleepStats = $state<TabSleepStats | null>(null);
  let autoRefresh = $state(true);
  let refreshInterval = $state(2000); // 2 seconds for real-time
  let refreshTimer: ReturnType<typeof setInterval> | undefined;

  async function collectMetrics() {
    const now = Date.now();

    try {
      // Get tab sleep stats
      const stats = await invoke<TabSleepStats>('tab_sleep_get_stats').catch(() => null);
      
      if (stats) {
        tabSleepStats = stats;

        // Add memory data point
        memoryData = [
          ...memoryData.slice(-49),
          { timestamp: now, value: stats.active_memory_mb }
        ];

        // Add tab count data point
        tabCountData = [
          ...tabCountData.slice(-49),
          { timestamp: now, value: stats.total_tabs }
        ];
      }

      // Simulate CPU usage (in real app, get from system)
      const cpuUsage = Math.random() * 20 + 5; // 5-25%
      cpuData = [
        ...cpuData.slice(-49),
        { timestamp: now, value: cpuUsage }
      ];

      // Record a metric counter for requests
      await invoke('metrics_counter', {
        name: 'page_views_total',
        value: 1.0,
        labels: { source: 'monitor' }
      });

      // Get request count (simulated)
      const requestCount = Math.floor(Math.random() * 50) + 100;
      requestsData = [
        ...requestsData.slice(-49),
        { timestamp: now, value: requestCount }
      ];

    } catch (error) {
      console.error('Failed to collect metrics:', error);
    }
  }

  function startAutoRefresh() {
    if (refreshTimer) clearInterval(refreshTimer);
    if (autoRefresh) {
      refreshTimer = setInterval(() => {
        void collectMetrics();
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

  onMount(() => {
    void collectMetrics();
    startAutoRefresh();

    return () => {
      if (refreshTimer) clearInterval(refreshTimer);
    };
  });

  $effect(() => {
    startAutoRefresh();
  });
</script>

<div class="advanced-monitor">
  <div class="monitor-header">
    <h2>📊 实时性能监控</h2>
    <div class="controls">
      <label>
        <input type="checkbox" bind:checked={autoRefresh} />
        自动刷新
      </label>
      <select bind:value={refreshInterval} disabled={!autoRefresh}>
        <option value={1000}>1秒</option>
        <option value={2000}>2秒</option>
        <option value={5000}>5秒</option>
      </select>
      <button onclick={() => void collectMetrics()}>🔄 刷新</button>
    </div>
  </div>

  <div class="charts-grid">
    <MetricsChart
      title="内存使用 (MB)"
      data={memoryData}
      color="#22c55e"
      height={180}
      unit=" MB"
    />

    <MetricsChart
      title="CPU 使用率 (%)"
      data={cpuData}
      color="#3b82f6"
      height={180}
      unit="%"
    />

    <MetricsChart
      title="标签页数量"
      data={tabCountData}
      color="#eab308"
      height={180}
    />

    <MetricsChart
      title="请求数/分钟"
      data={requestsData}
      color="#ef4444"
      height={180}
    />
  </div>

  {#if tabSleepStats}
    <div class="stats-summary">
      <div class="stat-card">
        <div class="stat-icon">💤</div>
        <div class="stat-content">
          <div class="stat-label">休眠标签页</div>
          <div class="stat-value">{tabSleepStats.sleeping_tabs}</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">⚡</div>
        <div class="stat-content">
          <div class="stat-label">活跃标签页</div>
          <div class="stat-value">{tabSleepStats.active_tabs}</div>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">📌</div>
        <div class="stat-content">
          <div class="stat-label">固定标签页</div>
          <div class="stat-value">{tabSleepStats.pinned_tabs}</div>
        </div>
      </div>

      <div class="stat-card highlight">
        <div class="stat-icon">💾</div>
        <div class="stat-content">
          <div class="stat-label">节省内存</div>
          <div class="stat-value">
            {formatBytes(tabSleepStats.saved_memory_mb * 1024 * 1024)}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .advanced-monitor {
    padding: 1.5rem;
    background: var(--bg-primary, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    min-height: 100vh;
  }

  .monitor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--border-color, #333);
  }

  .monitor-header h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
  }

  .controls {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .controls label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .controls input[type="checkbox"] {
    cursor: pointer;
  }

  .controls select {
    padding: 0.5rem;
    background: var(--bg-secondary, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #444);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
  }

  .controls select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .controls button {
    padding: 0.5rem 1rem;
    background: var(--accent-color, #007bff);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    font-size: 0.875rem;
    transition: background 0.2s;
  }

  .controls button:hover {
    background: var(--accent-hover, #0056b3);
  }

  .charts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .stats-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.5rem;
    background: var(--bg-secondary, #2a2a2a);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .stat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .stat-card.highlight {
    background: linear-gradient(135deg, #1e3a8a 0%, #3b82f6 100%);
    border-color: #3b82f6;
  }

  .stat-icon {
    font-size: 2rem;
    line-height: 1;
  }

  .stat-content {
    flex: 1;
  }

  .stat-label {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.7);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 0.25rem;
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 700;
    color: white;
  }

  @media (max-width: 768px) {
    .charts-grid {
      grid-template-columns: 1fr;
    }

    .stats-summary {
      grid-template-columns: 1fr;
    }

    .monitor-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;
    }

    .controls {
      width: 100%;
      flex-wrap: wrap;
    }
  }
</style>
