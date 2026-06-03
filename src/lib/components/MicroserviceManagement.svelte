<script lang="ts">
  /**
   * Exodus Browser — Microservice Management UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface Microservice {
    id: string;
    name: string;
    status: 'running' | 'stopped' | 'error';
    port: number;
    health_status: 'healthy' | 'unhealthy' | 'unknown';
    cpu_usage: number;
    memory_usage: number;
    uptime: number;
    last_restart: number;
  }

  let services: Microservice[] = [];
  let showAddServiceDialog = false;
  let selectedService: Microservice | null = null;

  // Add service form
  let serviceName = '';
  let servicePort = 3000;
  let serviceBinary = '';

  async function loadServices() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    services = [];
  }

  async function addService() {
    if (!serviceName) return;

    try {
      const service: Microservice = {
        id: crypto.randomUUID(),
        name: serviceName,
        status: 'stopped',
        port: servicePort,
        health_status: 'unknown',
        cpu_usage: 0,
        memory_usage: 0,
        uptime: 0,
        last_restart: Date.now() / 1000,
      };

      // In a real implementation, this would add via the backend
      console.log('Adding microservice:', service);
      
      services.push(service);
      showAddServiceDialog = false;
      serviceName = '';
      servicePort = 3000;
      serviceBinary = '';
    } catch (error) {
      console.error('Failed to add service:', error);
    }
  }

  async function startService(id: string) {
    try {
      const service = services.find((s) => s.id === id);
      if (service) {
        service.status = 'running';
        // In a real implementation, this would start via the backend
        console.log('Starting service:', id);
      }
    } catch (error) {
      console.error('Failed to start service:', error);
    }
  }

  async function stopService(id: string) {
    if (!confirm('Are you sure you want to stop this service?')) return;

    try {
      const service = services.find((s) => s.id === id);
      if (service) {
        service.status = 'stopped';
        // In a real implementation, this would stop via the backend
        console.log('Stopping service:', id);
      }
    } catch (error) {
      console.error('Failed to stop service:', error);
    }
  }

  async function restartService(id: string) {
    try {
      const service = services.find((s) => s.id === id);
      if (service) {
        service.status = 'running';
        service.last_restart = Date.now() / 1000;
        // In a real implementation, this would restart via the backend
        console.log('Restarting service:', id);
      }
    } catch (error) {
      console.error('Failed to restart service:', error);
    }
  }

  async function deleteService(id: string) {
    if (!confirm('Are you sure you want to delete this service?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting service:', id);
      
      services = services.filter((s) => s.id !== id);
      
      if (selectedService && selectedService.id === id) {
        selectedService = null;
      }
    } catch (error) {
      console.error('Failed to delete service:', error);
    }
  }

  function selectService(service: Microservice) {
    selectedService = service;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running':
        return '#059669';
      case 'stopped':
        return '#6b7280';
      case 'error':
        return '#dc2626';
      default:
        return '#888';
    }
  }

  function getHealthColor(health: string): string {
    switch (health) {
      case 'healthy':
        return '#059669';
      case 'unhealthy':
        return '#dc2626';
      default:
        return '#d97706';
    }
  }

  function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  // Load services on mount
  loadServices();
</script>

<div class="microservice-management">
  <div class="header">
    <h2>Microservice Management</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showAddServiceDialog = true)}>
        Add Service
      </button>
    </div>
  </div>

  <div class="service-container">
    <div class="service-list">
      {#if services.length === 0}
        <div class="empty-state">
          <div class="empty-icon">⚙️</div>
          <p>No microservices</p>
        </div>
      {:else}
        {#each services as service (service.id)}
          <div
            class="service-item {selectedService?.id === service.id ? 'active' : ''}"
            on:click={() => selectService(service)}
          >
            <div class="service-icon">
              <div class="status-indicator {service.status}"></div>
            </div>
            <div class="service-info">
              <div class="name">{service.name}</div>
              <div class="port">Port: {service.port}</div>
              <div class="status" style="color: {getStatusColor(service.status)}">
                {service.status}
              </div>
              <div class="health" style="color: {getHealthColor(service.health_status)}">
                Health: {service.health_status}
              </div>
            </div>
            <div class="service-actions">
              {#if service.status === 'running'}
                <button class="btn-icon" title="Stop" on:click|stopPropagation={() => stopService(service.id)}>
                  ⏸️
                </button>
                <button class="btn-icon" title="Restart" on:click|stopPropagation={() => restartService(service.id)}>
                  🔄
                </button>
              {:else}
                <button class="btn-icon" title="Start" on:click|stopPropagation={() => startService(service.id)}>
                  ▶️
                </button>
              {/if}
              <button class="btn-icon" title="Delete" on:click|stopPropagation={() => deleteService(service.id)}>
                🗑️
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="service-details">
      {#if selectedService}
        <div class="details-header">
          <div class="service-header-info">
            <div class="status-indicator large {selectedService.status}"></div>
            <div>
              <h3>{selectedService.name}</h3>
              <div class="status" style="color: {getStatusColor(selectedService.status)}">
                {selectedService.status}
              </div>
            </div>
          </div>
          <button class="btn-icon" on:click={() => (selectedService = null)}>✕</button>
        </div>
        <div class="details-content">
          <div class="detail-section">
            <h4>Port</h4>
            <div class="port">{selectedService.port}</div>
          </div>
          <div class="detail-section">
            <h4>Health Status</h4>
            <div class="health" style="color: {getHealthColor(selectedService.health_status)}">
              {selectedService.health_status}
            </div>
          </div>
          <div class="detail-section">
            <h4>Resource Usage</h4>
            <div class="resource-usage">
              <div class="usage-item">
                <span class="label">CPU:</span>
                <span class="value">{selectedService.cpu_usage}%</span>
              </div>
              <div class="usage-item">
                <span class="label">Memory:</span>
                <span class="value">{selectedService.memory_usage} MB</span>
              </div>
            </div>
          </div>
          <div class="detail-section">
            <h4>Uptime</h4>
            <div class="uptime">{formatUptime(selectedService.uptime)}</div>
          </div>
          <div class="detail-section">
            <h4>Last Restart</h4>
            <div class="last-restart">{formatDate(selectedService.last_restart)}</div>
          </div>
          <div class="detail-section">
            <h4>Actions</h4>
            <div class="service-actions-detail">
              {#if selectedService.status === 'running'}
                <button class="btn btn-secondary" on:click={() => stopService(selectedService.id)}>
                  Stop Service
                </button>
                <button class="btn btn-secondary" on:click={() => restartService(selectedService.id)}>
                  Restart Service
                </button>
              {:else}
                <button class="btn btn-primary" on:click={() => startService(selectedService.id)}>
                  Start Service
                </button>
              {/if}
              <button class="btn btn-danger" on:click={() => deleteService(selectedService.id)}>
                Delete Service
              </button>
            </div>
          </div>
        </div>
      {:else}
        <div class="no-selection">
          <div class="no-selection-icon">⚙️</div>
          <p>Select a service to view details</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- Add Service Dialog -->
  {#if showAddServiceDialog}
    <div class="dialog-overlay" on:click={() => (showAddServiceDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>Add Microservice</h3>
        <form on:submit|preventDefault={addService}>
          <div class="form-group">
            <label>Service Name</label>
            <input type="text" bind:value={serviceName} required />
          </div>
          <div class="form-group">
            <label>Port</label>
            <input type="number" bind:value={servicePort} min="1" max="65535" required />
          </div>
          <div class="form-group">
            <label>Binary Path (optional)</label>
            <input type="text" bind:value={serviceBinary} placeholder="/path/to/binary" />
          </div>
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showAddServiceDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Add Service</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .microservice-management {
    padding: 20px;
    height: 100%;
    display: flex;
    flex-direction: column;
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

  .service-container {
    flex: 1;
    display: flex;
    gap: 20px;
    min-height: 0;
  }

  .service-list {
    width: 350px;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #888;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 20px;
  }

  .service-item {
    display: flex;
    align-items: center;
    gap: 15px;
    padding: 15px;
    background: #444;
    border-radius: 6px;
    margin-bottom: 10px;
    cursor: pointer;
    position: relative;
    transition: background 0.2s;
  }

  .service-item:hover {
    background: #555;
  }

  .service-item.active {
    background: #6366f1;
    border-color: #4f46e5;
  }

  .service-icon {
    position: relative;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
  }

  .status-indicator.large {
    width: 24px;
    height: 24px;
  }

  .status-indicator.running {
    background: #059669;
    box-shadow: 0 0 8px #059669;
  }

  .status-indicator.stopped {
    background: #6b7280;
  }

  .status-indicator.error {
    background: #dc2626;
  }

  .service-info {
    flex: 1;
  }

  .name {
    font-weight: bold;
    color: #eee;
    margin-bottom: 3px;
  }

  .port {
    color: #aaa;
    font-size: 12px;
    margin-bottom: 2px;
  }

  .status {
    font-size: 12px;
    font-weight: bold;
    text-transform: capitalize;
    margin-bottom: 2px;
  }

  .health {
    font-size: 12px;
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

  .service-details {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #444;
  }

  .service-header-info {
    display: flex;
    align-items: center;
    gap: 15px;
  }

  .service-header-info h3 {
    margin: 0;
  }

  .details-content {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }

  .detail-section {
    margin-bottom: 25px;
  }

  .detail-section h4 {
    margin: 0 0 10px 0;
    color: #aaa;
  }

  .detail-section .port,
  .detail-section .uptime,
  .detail-section .last-restart {
    color: #eee;
    font-size: 16px;
  }

  .resource-usage {
    display: flex;
    gap: 20px;
  }

  .usage-item {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .usage-item .label {
    color: #888;
    font-size: 12px;
  }

  .usage-item .value {
    color: #eee;
    font-size: 18px;
    font-weight: bold;
  }

  .service-actions-detail {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .no-selection {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    color: #888;
  }

  .no-selection-icon {
    font-size: 64px;
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
  .form-group input[type='number'] {
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

  .btn-danger {
    background: #dc2626;
    color: white;
  }

  .btn-danger:hover {
    background: #b91c1c;
  }

  .btn-icon {
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 8px;
    font-size: 16px;
  }

  .btn-icon:hover {
    color: #eee;
  }
</style>
