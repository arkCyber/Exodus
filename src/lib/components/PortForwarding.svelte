<script lang="ts">
  /**
   * Exodus Browser — Port Forwarding UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface PortForwardingRule {
    id: string;
    name: string;
    local_port: number;
    remote_host: string;
    remote_port: number;
    protocol: 'tcp' | 'udp';
    status: 'active' | 'inactive' | 'error';
    bytes_transferred: number;
    created_at: number;
    last_activity: number;
  }

  let rules: PortForwardingRule[] = [];
  let showAddRuleDialog = false;

  // Add rule form
  let ruleName = '';
  let ruleLocalPort = 8080;
  let ruleRemoteHost = 'localhost';
  let ruleRemotePort = 3000;
  let ruleProtocol: 'tcp' | 'udp' = 'tcp';

  async function loadRules() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    rules = [];
  }

  async function addRule() {
    if (!ruleName) return;

    try {
      const rule: PortForwardingRule = {
        id: crypto.randomUUID(),
        name: ruleName,
        local_port: ruleLocalPort,
        remote_host: ruleRemoteHost,
        remote_port: ruleRemotePort,
        protocol: ruleProtocol,
        status: 'active',
        bytes_transferred: 0,
        created_at: Date.now() / 1000,
        last_activity: Date.now() / 1000,
      };

      // In a real implementation, this would add via the backend
      console.log('Adding port forwarding rule:', rule);
      
      rules.push(rule);
      showAddRuleDialog = false;
      ruleName = '';
      ruleLocalPort = 8080;
      ruleRemoteHost = 'localhost';
      ruleRemotePort = 3000;
    } catch (error) {
      console.error('Failed to add rule:', error);
    }
  }

  async function toggleRule(id: string) {
    try {
      const rule = rules.find((r) => r.id === id);
      if (rule) {
        rule.status = rule.status === 'active' ? 'inactive' : 'active';
        // In a real implementation, this would toggle via the backend
        console.log('Toggling rule:', id, 'to', rule.status);
      }
    } catch (error) {
      console.error('Failed to toggle rule:', error);
    }
  }

  async function deleteRule(id: string) {
    if (!confirm('Are you sure you want to delete this rule?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting rule:', id);
      
      rules = rules.filter((r) => r.id !== id);
    } catch (error) {
      console.error('Failed to delete rule:', error);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'active':
        return '#059669';
      case 'inactive':
        return '#6b7280';
      case 'error':
        return '#dc2626';
      default:
        return '#888';
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  // Load rules on mount
  loadRules();
</script>

<div class="port-forwarding">
  <div class="header">
    <h2>Port Forwarding</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showAddRuleDialog = true)}>
        Add Rule
      </button>
    </div>
  </div>

  <div class="rule-list">
    {#if rules.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔌</div>
        <p>No port forwarding rules</p>
      </div>
    {:else}
      {#each rules as rule (rule.id)}
        <div class="rule-item">
          <div class="rule-info">
            <div class="name">{rule.name}</div>
            <div class="mapping">
              {rule.local_port} → {rule.remote_host}:{rule.remote_port}
            </div>
            <div class="protocol">Protocol: {rule.protocol.toUpperCase()}</div>
            <div class="meta">
              <span class="status" style="color: {getStatusColor(rule.status)}">
                {rule.status}
              </span>
              <span class="bytes">Transferred: {formatBytes(rule.bytes_transferred)}</span>
              <span class="last-activity">Last: {formatDate(rule.last_activity)}</span>
            </div>
          </div>
          <div class="rule-actions">
            <button
              class="btn-icon {rule.status === 'active' ? 'active' : 'inactive'}"
              title="Toggle"
              on:click={() => toggleRule(rule.id)}
            >
              {rule.status === 'active' ? '⏸️' : '▶️'}
            </button>
            <button class="btn-icon" title="Delete" on:click={() => deleteRule(rule.id)}>
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Add Rule Dialog -->
  {#if showAddRuleDialog}
    <div class="dialog-overlay" on:click={() => (showAddRuleDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Add Port Forwarding Rule</h3>
        <form on:submit|preventDefault={addRule}>
          <div class="form-group">
            <label>Rule Name</label>
            <input type="text" bind:value={ruleName} required />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Local Port</label>
              <input type="number" bind:value={ruleLocalPort} min="1" max="65535" required />
            </div>
            <div class="form-group">
              <label>Remote Host</label>
              <input type="text" bind:value={ruleRemoteHost} required />
            </div>
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Remote Port</label>
              <input type="number" bind:value={ruleRemotePort} min="1" max="65535" required />
            </div>
            <div class="form-group">
              <label>Protocol</label>
              <select bind:value={ruleProtocol}>
                <option value="tcp">TCP</option>
                <option value="udp">UDP</option>
              </select>
            </div>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showAddRuleDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Add Rule</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .port-forwarding {
    padding: 20px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .header h2 {
    margin: 0;
  }

  .rule-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #888;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 20px;
  }

  .rule-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .rule-info {
    flex: 1;
  }

  .name {
    font-weight: bold;
    color: #eee;
    margin-bottom: 5px;
  }

  .mapping {
    color: #aaa;
    font-size: 14px;
    margin-bottom: 3px;
    font-family: monospace;
  }

  .protocol {
    color: #888;
    font-size: 12px;
    margin-bottom: 8px;
  }

  .meta {
    display: flex;
    gap: 15px;
    font-size: 12px;
    color: #888;
  }

  .status {
    font-weight: bold;
    text-transform: capitalize;
  }

  .rule-actions {
    display: flex;
    gap: 5px;
  }

  .btn-icon {
    background: #444;
    border: 1px solid #555;
    color: #eee;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-icon:hover {
    background: #555;
  }

  .btn-icon.active {
    background: #059669;
    border-color: #065f46;
  }

  .btn-icon.inactive {
    background: #d97706;
    border-color: #b45309;
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #333;
    border: 1px solid #555;
    border-radius: 8px;
    padding: 20px;
    min-width: 500px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
  }

  .form-row {
    display: flex;
    gap: 15px;
  }

  .form-row .form-group {
    flex: 1;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    color: #aaa;
  }

  .form-group input[type='text'],
  .form-group input[type='number'],
  .form-group select {
    width: 100%;
    padding: 8px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    border: none;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #444;
    color: #eee;
  }

  .btn-secondary:hover {
    background: #555;
  }
</style>
