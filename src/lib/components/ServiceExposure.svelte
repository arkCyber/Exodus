<script lang="ts">
  /**
   * Exodus Browser — Service Exposure UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface ExposedService {
    id: string;
    name: string;
    local_port: number;
    external_port: number;
    protocol: 'http' | 'https' | 'tcp' | 'udp';
    status: 'active' | 'inactive' | 'error';
    access_url: string;
    created_at: number;
    last_accessed: number;
    access_count: number;
  }

  let services: ExposedService[] = [];
  let showExposeDialog = false;

  // Expose form
  let serviceName = '';
  let serviceLocalPort = 8080;
  let serviceProtocol: 'http' | 'https' | 'tcp' | 'udp' = 'http';

  async function loadServices() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    services = [];
  }

  async function exposeService() {
    if (!serviceName) return;

    try {
      const service: ExposedService = {
        id: crypto.randomUUID(),
        name: serviceName,
        local_port: serviceLocalPort,
        external_port: Math.floor(Math.random() * 10000) + 10000,
        protocol: serviceProtocol,
        status: 'active',
        access_url: '',
        created_at: Date.now() / 1000,
        last_accessed: Date.now() / 1000,
        access_count: 0,
      };

      // In a real implementation, this would expose via the backend
      console.log('Exposing service:', service);
      
      services.push(service);
      showExposeDialog = false;
      serviceName = '';
      serviceLocalPort = 8080;
    } catch (error) {
      console.error('Failed to expose service:', error);
    }
  }

  async function stopService(id: string) {
    if (!confirm('Are you sure you want to stop this service?')) return;

    try {
      // In a real implementation, this would stop via the backend
      console.log('Stopping service:', id);
      
      const service = services.find((s) => s.id === id);
      if (service) {
        service.status = 'inactive';
      }
    } catch (error) {
      console.error('Failed to stop service:', error);
    }
  }

  async function deleteService(id: string) {
    if (!confirm('Are you sure you want to delete this service?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting service:', id);
      
      services = services.filter((s) => s.id !== id);
    } catch (error) {
      console.error('Failed to delete service:', error);
    }
  }

  async function copyAccessUrl(url: string) {
    try {
      await navigator.clipboard.writeText(url);
    } catch (error) {
      console.error('Failed to copy URL:', error);
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

  function getProtocolIcon(protocol: string): string {
    switch (protocol) {
      case 'http':
        return '🌐';
      case 'https':
        return '🔒';
      case 'tcp':
        return '📡';
      case 'udp':
        return '📤';
      default:
        return '🔌';
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  // Load services on mount
  loadServices();
</script>

<div class="service-exposure">
  <div class="header">
    <h2>Service Exposure</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showExposeDialog = true)}>
        Expose Service
      </button>
    </div>
  </div>

  <div class="service-list">
    {#if services.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🌐</div>
        <p>No exposed services</p>
      </div>
    {:else}
      {#each services as service (service.id)}
        <div class="service-item">
          <div class="service-icon">{getProtocolIcon(service.protocol)}</div>
          <div class="service-info">
            <div class="name">{service.name}</div>
            <div class="ports">
              Local: {service.local_port} → External: {service.external_port}
            </div>
            <div class="protocol">Protocol: {service.protocol.toUpperCase()}</div>
            {#if service.access_url}
              <div class="access-url">
                <input type="text" value={service.access_url} readonly />
                <button
                  class="btn-icon"
                  title="Copy URL"
                  on:click={() => copyAccessUrl(service.access_url)}
                >
                  📋
                </button>
              </div>
            {/if}
            <div class="meta">
              <span class="status" style="color: {getStatusColor(service.status)}">
                {service.status}
              </span>
              <span class="access-count">{service.access_count} accesses</span>
              <span class="last-accessed">Last: {formatDate(service.last_accessed)}</span>
            </div>
          </div>
          <div class="service-actions">
            {#if service.status === 'active'}
              <button class="btn-icon" title="Stop" on:click={() => stopService(service.id)}>
                ⏸️
              </button>
            {/if}
            <button class="btn-icon" title="Delete" on:click={() => deleteService(service.id)}>
              🗑️
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Expose Service Dialog -->
  {#if showExposeDialog}
    <div class="dialog-overlay" on:click={() => (showExposeDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Expose Service</h3>
        <form on:submit|preventDefault={exposeService}>
          <div class="form-group">
            <label>Service Name</label>
            <input type="text" bind:value={serviceName} required />
          </div>
          <div class="form-group">
            <label>Local Port</label>
            <input type="number" bind:value={serviceLocalPort} min="1" max="65535" required />
          </div>
          <div class="form-group">
            <label>Protocol</label>
            <select bind:value={serviceProtocol}>
              <option value="http">HTTP</option>
              <option value="https">HTTPS</option>
              <option value="tcp">TCP</option>
              <option value="udp">UDP</option>
            </select>
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showExposeDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Expose</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .service-exposure {
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

  .service-list {
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

  .service-item {
    display: flex;
    align-items: flex-start;
    gap: 15px;
    padding: 15px;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .service-icon {
    font-size: 32px;
  }

  .service-info {
    flex: 1;
  }

  .name {
    font-weight: bold;
    color: #eee;
    margin-bottom: 5px;
  }

  .ports {
    color: #aaa;
    font-size: 14px;
    margin-bottom: 3px;
  }

  .protocol {
    color: #888;
    font-size: 12px;
    margin-bottom: 8px;
  }

  .access-url {
    display: flex;
    gap: 5px;
    margin-bottom: 8px;
  }

  .access-url input {
    flex: 1;
    padding: 6px;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
    font-family: monospace;
    font-size: 12px;
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

  .service-actions {
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
    min-width: 400px;
  }

  .dialog h3 {
    margin: 0 0 20px 0;
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
