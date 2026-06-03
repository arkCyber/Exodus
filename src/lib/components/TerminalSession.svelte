<script lang="ts">
  /**
   * Exodus Browser — Terminal Session UI
   */

  import { invoke } from '@tauri-apps/api/core';

  interface TerminalSession {
    id: string;
    name: string;
    host: string;
    port: number;
    status: 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error';
    created_at: number;
    last_activity: number;
  }

  let sessions: TerminalSession[] = [];
  let activeSession: TerminalSession | null = null;
  let showNewSessionDialog = false;
  let terminalOutput: string[] = [];
  let currentCommand = '';

  // New session form
  let newSessionName = '';
  let newSessionHost = 'localhost';
  let newSessionPort = 22;
  let newSessionType: 'ssh' | 'local' = 'ssh';

  async function loadSessions() {
    // In a real implementation, this would load from the backend
    // For now, we'll use a placeholder
    sessions = [];
  }

  async function createSession() {
    if (!newSessionName) return;

    try {
      const session: TerminalSession = {
        id: crypto.randomUUID(),
        name: newSessionName,
        host: newSessionHost,
        port: newSessionPort,
        status: 'idle',
        created_at: Date.now() / 1000,
        last_activity: Date.now() / 1000,
      };

      // In a real implementation, this would save via the backend
      console.log('Creating session:', session);
      
      sessions.push(session);
      showNewSessionDialog = false;
      newSessionName = '';
      newSessionHost = 'localhost';
      newSessionPort = 22;
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  }

  async function connectSession(session: TerminalSession) {
    activeSession = session;
    session.status = 'connecting';
    terminalOutput = [`Connecting to ${session.host}:${session.port}...`];

    try {
      // In a real implementation, this would connect via the backend
      console.log('Connecting to session:', session);
      
      // Simulate connection
      setTimeout(() => {
        session.status = 'connected';
        terminalOutput.push('Connected successfully');
        terminalOutput.push(`Welcome to ${session.name}`);
        terminalOutput.push('$ ');
      }, 1000);
    } catch (error) {
      console.error('Failed to connect:', error);
      session.status = 'error';
      terminalOutput.push(`Connection failed: ${error}`);
    }
  }

  async function disconnectSession() {
    if (!activeSession) return;

    try {
      // In a real implementation, this would disconnect via the backend
      console.log('Disconnecting from session:', activeSession);
      
      activeSession.status = 'disconnected';
      terminalOutput.push('Disconnected');
      activeSession = null;
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  }

  async function executeCommand() {
    if (!currentCommand || !activeSession) return;

    const command = currentCommand;
    currentCommand = '';
    terminalOutput.push(`$ ${command}`);

    try {
      // In a real implementation, this would execute via the backend
      console.log('Executing command:', command);
      
      // Simulate command output
      setTimeout(() => {
        const output = `Output for: ${command}`;
        terminalOutput.push(output);
        terminalOutput.push('$ ');
        activeSession.last_activity = Date.now() / 1000;
      }, 500);
    } catch (error) {
      console.error('Failed to execute command:', error);
      terminalOutput.push(`Error: ${error}`);
      terminalOutput.push('$ ');
    }
  }

  async function deleteSession(id: string) {
    if (!confirm('Are you sure you want to delete this session?')) return;

    try {
      // In a real implementation, this would delete via the backend
      console.log('Deleting session:', id);
      
      sessions = sessions.filter((s) => s.id !== id);
      
      if (activeSession && activeSession.id === id) {
        activeSession = null;
        terminalOutput = [];
      }
    } catch (error) {
      console.error('Failed to delete session:', error);
    }
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'connected':
        return '#059669';
      case 'connecting':
        return '#d97706';
      case 'disconnected':
        return '#6b7280';
      case 'error':
        return '#dc2626';
      default:
        return '#888';
    }
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      executeCommand();
    }
  }

  // Load sessions on mount
  loadSessions();
</script>

<div class="terminal-session">
  <div class="header">
    <h2>Terminal Sessions</h2>
    <div class="actions">
      <button class="btn btn-primary" on:click={() => (showNewSessionDialog = true)}>
        New Session
      </button>
    </div>
  </div>

  <div class="terminal-container">
    <div class="sidebar">
      <div class="sidebar-header">
        <h3>Sessions</h3>
      </div>
      <div class="session-list">
        {#if sessions.length === 0}
          <div class="empty-state">
            <p>No sessions</p>
          </div>
        {:else}
          {#each sessions as session (session.id)}
            <div
              class="session-item {activeSession?.id === session.id ? 'active' : ''}"
              on:click={() => connectSession(session)}
            >
              <div class="session-info">
                <div class="session-name">{session.name}</div>
                <div class="session-host">{session.host}:{session.port}</div>
                <div class="session-status" style="color: {getStatusColor(session.status)}">
                  {session.status}
                </div>
              </div>
              <button
                class="btn-icon delete"
                title="Delete"
                on:click|stopPropagation={() => deleteSession(session.id)}
              >
                🗑️
              </button>
            </div>
          {/each}
        {/if}
      </div>
    </div>

    <div class="terminal">
      {#if activeSession}
        <div class="terminal-header">
          <div class="session-info">
            <h3>{activeSession.name}</h3>
            <div class="meta">
              <span>{activeSession.host}:{activeSession.port}</span>
              <span>Status: {activeSession.status}</span>
            </div>
          </div>
          <div class="terminal-actions">
            <button class="btn btn-danger" on:click={disconnectSession}>
              Disconnect
            </button>
          </div>
        </div>

        <div class="terminal-output">
          {#each terminalOutput as line}
            <div class="output-line">{line}</div>
          {/each}
        </div>

        <div class="terminal-input">
          <span class="prompt">$</span>
          <input
            type="text"
            bind:value={currentCommand}
            on:keydown={handleKeydown}
            placeholder="Enter command..."
            class="command-input"
            disabled={activeSession.status !== 'connected'}
          />
        </div>
      {:else}
        <div class="no-session">
          <div class="no-session-icon">💻</div>
          <p>Select a session to connect</p>
        </div>
      {/if}
    </div>
  </div>

  <!-- New Session Dialog -->
  {#if showNewSessionDialog}
    <div class="dialog-overlay" on:click={() => (showNewSessionDialog = false)}>
      <div class="dialog" on:click|stopPropagation>
        <h3>New Terminal Session</h3>
        <form on:submit|preventDefault={createSession}>
          <div class="form-group">
            <label>Session Name</label>
            <input type="text" bind:value={newSessionName} required />
          </div>
          <div class="form-group">
            <label>Type</label>
            <select bind:value={newSessionType}>
              <option value="ssh">SSH</option>
              <option value="local">Local</option>
            </select>
          </div>
          {#if newSessionType === 'ssh'}
            <div class="form-group">
              <label>Host</label>
              <input type="text" bind:value={newSessionHost} />
            </div>
            <div class="form-group">
              <label>Port</label>
              <input type="number" bind:value={newSessionPort} />
            </div>
          {/if}
          <div class="form-actions">
            <button type="button" class="btn btn-secondary" on:click={() => (showNewSessionDialog = false)}>
              Cancel
            </button>
            <button type="submit" class="btn btn-primary">Create</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .terminal-session {
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

  .terminal-container {
    flex: 1;
    display: flex;
    gap: 20px;
    min-height: 0;
  }

  .sidebar {
    width: 300px;
    display: flex;
    flex-direction: column;
    background: #333;
    border-radius: 8px;
    border: 1px solid #444;
  }

  .sidebar-header {
    padding: 15px;
    border-bottom: 1px solid #444;
  }

  .sidebar-header h3 {
    margin: 0;
  }

  .session-list {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #888;
  }

  .session-item {
    padding: 15px;
    background: #444;
    border-radius: 6px;
    margin-bottom: 10px;
    cursor: pointer;
    position: relative;
    transition: background 0.2s;
  }

  .session-item:hover {
    background: #555;
  }

  .session-item.active {
    background: #6366f1;
    border-color: #4f46e5;
  }

  .session-info {
    margin-bottom: 5px;
  }

  .session-name {
    font-weight: bold;
    color: #eee;
    margin-bottom: 3px;
  }

  .session-host {
    font-size: 12px;
    color: #aaa;
    margin-bottom: 3px;
  }

  .session-status {
    font-size: 12px;
    font-weight: bold;
    text-transform: capitalize;
  }

  .session-item .btn-icon {
    position: absolute;
    top: 10px;
    right: 10px;
    background: transparent;
    border: none;
    color: #aaa;
    cursor: pointer;
    padding: 4px;
  }

  .session-item .btn-icon:hover {
    color: #dc2626;
  }

  .terminal {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    border-radius: 8px;
    border: 1px solid #444;
    min-height: 0;
  }

  .terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    border-bottom: 1px solid #444;
    background: #333;
  }

  .session-info h3 {
    margin: 0 0 5px 0;
  }

  .session-info .meta {
    font-size: 12px;
    color: #aaa;
  }

  .session-info .meta span {
    margin-right: 15px;
  }

  .terminal-actions {
    display: flex;
    gap: 10px;
  }

  .terminal-output {
    flex: 1;
    padding: 15px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    color: #eee;
  }

  .output-line {
    margin-bottom: 5px;
    white-space: pre-wrap;
  }

  .terminal-input {
    display: flex;
    align-items: center;
    padding: 15px;
    border-top: 1px solid #444;
    background: #333;
  }

  .prompt {
    color: #059669;
    margin-right: 10px;
    font-weight: bold;
  }

  .command-input {
    flex: 1;
    background: #444;
    border: 1px solid #555;
    border-radius: 4px;
    color: #eee;
    padding: 8px;
    font-family: 'Courier New', monospace;
  }

  .command-input:focus {
    outline: none;
    border-color: #6366f1;
  }

  .command-input:disabled {
    opacity: 0.6;
  }

  .no-session {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    color: #888;
  }

  .no-session-icon {
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

  .btn-danger {
    background: #dc2626;
    color: white;
  }

  .btn-danger:hover {
    background: #b91c1c;
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
</style>
