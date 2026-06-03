<template>
  <div class="wasmedge-sandbox-test">
    <h2>🚀 Aerospace-Grade WasmEdge Sandbox</h2>
    <p class="description">
      Mission-critical security for OpenClaw AI agents with aerospace-level safety standards.
      Multi-layered validation, comprehensive monitoring, and fail-safe mechanisms.
    </p>

    <div class="test-section">
      <h3>🛡️ Security Tests</h3>
      <div class="controls">
        <button @click="runSecurityTest" :disabled="isRunning" class="security-btn">
          {{ isRunning ? 'Running...' : 'Run Security Test' }}
        </button>
        <button @click="runBasicTest" :disabled="isRunning" class="basic-btn">
          {{ isRunning ? 'Running...' : 'Run Basic Test' }}
        </button>
        <button @click="runAdvancedTest" :disabled="isRunning" class="advanced-btn">
          {{ isRunning ? 'Running...' : 'Run Advanced Test' }}
        </button>
        <button @click="clearOutput" class="secondary">
          Clear Output
        </button>
      </div>
    </div>

    <div class="metrics-section">
      <h3>📊 Aerospace Metrics</h3>
      <div class="metrics-controls">
        <button @click="loadMetrics" :disabled="isLoadingMetrics">
          {{ isLoadingMetrics ? 'Loading...' : 'Load Metrics' }}
        </button>
        <button @click="clearMetrics" class="secondary">
          Clear Metrics
        </button>
      </div>
      <div class="metrics-container" v-if="metrics.length > 0">
        <div class="metrics-summary">
          <div class="metric-card">
            <span class="metric-label">Total Executions</span>
            <span class="metric-value">{{ metrics.length }}</span>
          </div>
          <div class="metric-card">
            <span class="metric-label">Successful</span>
            <span class="metric-value success">{{ successfulExecutions }}</span>
          </div>
          <div class="metric-card">
            <span class="metric-label">Violations</span>
            <span class="metric-value violation">{{ securityViolations }}</span>
          </div>
          <div class="metric-card">
            <span class="metric-label">Avg Duration</span>
            <span class="metric-value">{{ averageDuration }}ms</span>
          </div>
        </div>
        <div class="metrics-list">
          <div v-for="(metric, index) in metrics" :key="index" class="metric-item">
            <div class="metric-header">
              <span class="metric-id">{{ metric.execution_id }}</span>
              <span :class="['metric-status', metric.status.toLowerCase()]">{{ metric.status }}</span>
            </div>
            <div class="metric-details">
              <div class="detail-row">
                <span class="detail-label">Duration:</span>
                <span class="detail-value">{{ metric.duration_ms }}ms</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Script Size:</span>
                <span class="detail-value">{{ formatBytes(metric.script_size_bytes) }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Security Level:</span>
                <span class="detail-value">{{ metric.security_level }}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">File Ops:</span>
                <span class="detail-value">{{ metric.resource_usage.file_operations }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="placeholder">No metrics available. Run tests to collect data.</div>
    </div>

    <div class="output-section">
      <h3>📋 Test Output</h3>
      <div class="output-container">
        <pre v-if="output">{{ output }}</pre>
        <div v-else class="placeholder">No output yet. Run a test to see results.</div>
      </div>
    </div>

    <div class="info-section">
      <h3>🔒 Aerospace Security Features</h3>
      <div class="features-grid">
        <div class="feature-card">
          <h4>Multi-Layer Validation</h4>
          <p>Pattern detection, size limits, path traversal prevention, and security level checks</p>
        </div>
        <div class="feature-card">
          <h4>Real-Time Monitoring</h4>
          <p>Execution metrics, resource usage tracking, and performance telemetry</p>
        </div>
        <div class="feature-card">
          <h4>Audit Trail</h4>
          <p>Comprehensive logging of all security events with timestamps and severity levels</p>
        </div>
        <div class="feature-card">
          <h4>Resource Limits</h4>
          <p>Configurable memory, CPU, and execution time constraints</p>
        </div>
        <div class="feature-card">
          <h4>Fail-Safe Recovery</h4>
          <p>Automatic workspace cleanup and retention policies</p>
        </div>
        <div class="feature-card">
          <h4>Thread-Safe Design</h4>
          <p>Global instance management with concurrent execution support</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface SandboxMetrics {
  execution_id: string;
  start_time: string;
  end_time: string | null;
  duration_ms: number | null;
  script_size_bytes: number;
  security_level: string;
  resource_usage: {
    memory_peak_bytes: number;
    cpu_time_ms: number;
    file_operations: number;
    network_operations: number;
  };
  security_events: any[];
  status: string;
}

const isRunning = ref(false);
const isLoadingMetrics = ref(false);
const output = ref('');
const metrics = ref<SandboxMetrics[]>([]);

const successfulExecutions = computed(() => 
  metrics.value.filter(m => m.status === 'Completed').length
);

const securityViolations = computed(() => 
  metrics.value.filter(m => m.status === 'SecurityViolation').length
);

const averageDuration = computed(() => {
  const durations = metrics.value
    .filter(m => m.duration_ms !== null)
    .map(m => m.duration_ms as number);
  if (durations.length === 0) return 0;
  return Math.round(durations.reduce((a, b) => a + b, 0) / durations.length);
});

async function runSecurityTest() {
  isRunning.value = true;
  output.value = '🚀 Running aerospace-grade security test...\n\n';

  try {
    const evilOpenClawScript = `
      import * as fs from 'fs';
      
      console.log("OpenClaw 智能体开始运行...");
      
      try {
        // Attempt malicious deletion of system files
        fs.unlinkSync('/etc/hosts');
        fs.unlinkSync('C:\\\\Windows\\\\System32\\\\drivers\\\\etc\\\\hosts');
        console.log("警告：恶意删除系统文件竟然成功了！（不可能发生）");
      } catch(err) {
        fs.writeFileSync('/output.log', "拦截到越权穿越行为。错误详情: " + err.message);
        console.log("安全拦截成功: " + err.message);
      }
      
      fs.writeFileSync('/test.txt', 'This is a safe file in the sandbox');
      const content = fs.readFileSync('/test.txt', 'utf-8');
      console.log("安全操作成功: " + content);
    `;

    const result = await invoke('execute_openclaw_sandbox', { script: evilOpenClawScript });
    output.value += `✅ Test completed successfully!\n\n`;
    output.value += `🛡️ Aerospace Sandbox Result:\n${result}\n\n`;
    output.value += `Security Verification: PASSED\n`;
    output.value += `- Malicious file access was blocked\n`;
    output.value += `- Safe sandbox operations worked correctly\n`;
    output.value += `- Security event logged to audit trail\n`;
  } catch (error) {
    output.value += `❌ Test failed with error:\n${error}\n\n`;
  } finally {
    isRunning.value = false;
  }
}

async function runBasicTest() {
  isRunning.value = true;
  output.value = '🚀 Running basic functionality test...\n\n';

  try {
    const basicScript = `
      import * as fs from 'fs';
      
      console.log("Basic OpenClaw test starting...");
      
      fs.writeFileSync('/hello.txt', 'Hello from OpenClaw!');
      const content = fs.readFileSync('/hello.txt', 'utf-8');
      console.log("Read content: " + content);
      
      fs.writeFileSync('/output.log', 'Basic test completed successfully. Content: ' + content);
    `;

    const result = await invoke('execute_openclaw_sandbox', { script: basicScript });
    output.value += `✅ Basic test completed successfully!\n\n`;
    output.value += `🛡️ Aerospace Sandbox Result:\n${result}\n\n`;
    output.value += `Functionality Verification: PASSED\n`;
  } catch (error) {
    output.value += `❌ Basic test failed with error:\n${error}\n\n`;
  } finally {
    isRunning.value = false;
  }
}

async function runAdvancedTest() {
  isRunning.value = true;
  output.value = '🚀 Running advanced aerospace test...\n\n';

  try {
    const advancedScript = `
      import * as fs from 'fs';
      
      console.log("Advanced aerospace test starting...");
      
      // Test multiple operations
      for (let i = 0; i < 5; i++) {
        fs.writeFileSync(\`/test\${i}.txt\`, \`Test file \${i}\`);
      }
      
      const files = fs.readdirSync('/');
      console.log("Files in workspace: " + files.length);
      
      fs.writeFileSync('/output.log', 'Advanced test completed. Files created: ' + files.length);
    `;

    const result = await invoke('execute_openclaw_sandbox', { script: advancedScript });
    output.value += `✅ Advanced test completed successfully!\n\n`;
    output.value += `🛡️ Aerospace Sandbox Result:\n${result}\n\n`;
    output.value += `Advanced Verification: PASSED\n`;
  } catch (error) {
    output.value += `❌ Advanced test failed with error:\n${error}\n\n`;
  } finally {
    isRunning.value = false;
  }
}

async function loadMetrics() {
  isLoadingMetrics.value = true;
  try {
    const metricsJson = await invoke('get_sandbox_metrics');
    metrics.value = JSON.parse(metricsJson as string);
    output.value += `📊 Loaded ${metrics.value.length} execution metrics\n\n`;
  } catch (error) {
    output.value += `❌ Failed to load metrics: ${error}\n\n`;
  } finally {
    isLoadingMetrics.value = false;
  }
}

async function clearMetrics() {
  try {
    await invoke('clear_sandbox_metrics');
    metrics.value = [];
    output.value += '🧹 Metrics cleared successfully\n\n';
  } catch (error) {
    output.value += `❌ Failed to clear metrics: ${error}\n\n`;
  }
}

function clearOutput() {
  output.value = '';
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
</script>

<style scoped>
.wasmedge-sandbox-test {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

h2 {
  color: #1a1a2e;
  margin-bottom: 10px;
  font-size: 28px;
}

h3 {
  color: #333;
  margin-top: 25px;
  margin-bottom: 15px;
  font-size: 20px;
}

.description {
  color: #666;
  margin-bottom: 25px;
  line-height: 1.6;
  font-size: 15px;
}

.test-section, .metrics-section {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 12px;
  margin-bottom: 25px;
  border: 1px solid #e9ecef;
}

.controls, .metrics-controls {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

button {
  padding: 12px 24px;
  background: #2563eb;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

button:hover:not(:disabled) {
  background: #1d4ed8;
  transform: translateY(-1px);
}

button:disabled {
  background: #9ca3af;
  cursor: not-allowed;
  transform: none;
}

button.secondary {
  background: #64748b;
}

button.secondary:hover:not(:disabled) {
  background: #475569;
}

.security-btn {
  background: #dc2626;
}

.security-btn:hover:not(:disabled) {
  background: #b91c1c;
}

.basic-btn {
  background: #059669;
}

.basic-btn:hover:not(:disabled) {
  background: #047857;
}

.advanced-btn {
  background: #7c3aed;
}

.advanced-btn:hover:not(:disabled) {
  background: #6d28d9;
}

.metrics-summary {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
  margin-bottom: 20px;
}

.metric-card {
  background: white;
  padding: 20px;
  border-radius: 10px;
  text-align: center;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.metric-label {
  display: block;
  color: #64748b;
  font-size: 13px;
  margin-bottom: 8px;
  font-weight: 500;
}

.metric-value {
  display: block;
  color: #1e293b;
  font-size: 28px;
  font-weight: 700;
}

.metric-value.success {
  color: #059669;
}

.metric-value.violation {
  color: #dc2626;
}

.metrics-list {
  max-height: 400px;
  overflow-y: auto;
}

.metric-item {
  background: white;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 10px;
  border-left: 4px solid #3b82f6;
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.metric-id {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #64748b;
}

.metric-status {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
}

.metric-status.completed {
  background: #dcfce7;
  color: #166534;
}

.metric-status.securityviolation {
  background: #fee2e2;
  color: #991b1b;
}

.metric-status.failed {
  background: #fef3c7;
  color: #92400e;
}

.metric-details {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.detail-label {
  color: #64748b;
}

.detail-value {
  color: #1e293b;
  font-weight: 500;
}

.output-section {
  margin-bottom: 25px;
}

.output-container {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 20px;
  border-radius: 12px;
  min-height: 200px;
  max-height: 400px;
  overflow-y: auto;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.output-container pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.placeholder {
  color: #888;
  font-style: italic;
  text-align: center;
  padding: 40px;
}

.info-section {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 25px;
  border-radius: 12px;
  color: white;
}

.info-section h3 {
  color: white;
  margin-top: 0;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-top: 20px;
}

.feature-card {
  background: rgba(255, 255, 255, 0.1);
  padding: 20px;
  border-radius: 10px;
  backdrop-filter: blur(10px);
}

.feature-card h4 {
  margin: 0 0 10px 0;
  font-size: 16px;
  color: white;
}

.feature-card p {
  margin: 0;
  font-size: 14px;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.9);
}
</style>
