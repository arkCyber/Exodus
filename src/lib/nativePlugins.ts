//! Exodus Browser Native Plugin API
//! 
//! TypeScript bindings for the native Rust plugin system

import { invoke } from '@tauri-apps/api/core';

export interface PluginMetadata {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  permissions: string[];
  apiVersion: string;
}

export interface PluginCommandParams {
  [key: string]: unknown;
}

export interface PluginCommandResult {
  success: boolean;
  data?: unknown;
  error?: string;
}

/**
 * Initialize the native plugin manager
 */
export async function initNativePluginManager(): Promise<string> {
  const result = await invoke<string>('init_native_plugin_manager');
  return result;
}

/**
 * Load a native plugin from a file path
 */
export async function loadNativePlugin(path: string): Promise<PluginMetadata> {
  const result = await invoke<PluginMetadata>('load_native_plugin', { path });
  return result;
}

/**
 * Unload a native plugin
 */
export async function unloadNativePlugin(id: string): Promise<void> {
  await invoke('unload_native_plugin', { id });
}

/**
 * List all loaded native plugins
 */
export async function listNativePlugins(): Promise<PluginMetadata[]> {
  const result = await invoke<PluginMetadata[]>('list_native_plugins');
  return result;
}

/**
 * Get metadata for a specific plugin
 */
export async function getNativePlugin(id: string): Promise<PluginMetadata | null> {
  const result = await invoke<PluginMetadata | null>('get_native_plugin', { id });
  return result;
}

/**
 * Execute a command on a native plugin
 */
export async function executePluginCommand(
  id: string,
  command: string,
  params: PluginCommandParams = {}
): Promise<unknown> {
  const result = await invoke('execute_plugin_command', {
    id,
    command,
    params,
  });
  return result;
}

/**
 * Enable or disable a native plugin
 */
export async function setNativePluginEnabled(id: string, enabled: boolean): Promise<void> {
  await invoke('set_native_plugin_enabled', { id, enabled });
}

/**
 * Scan and load all plugins from the plugins directory
 */
export async function scanNativePlugins(): Promise<number> {
  const result = await invoke<number>('scan_native_plugins');
  return result;
}

/**
 * Check for plugin file changes and reload if needed
 */
export async function reloadChangedPlugins(): Promise<string[]> {
  const result = await invoke<string[]>('reload_changed_plugins');
  return result;
}

/**
 * Resource usage statistics for a plugin
 */
export interface ResourceStats {
  command_count: number;
  network_request_count: number;
  max_concurrent_commands: number;
  max_network_requests_per_minute: number;
}

/**
 * Get resource statistics for a plugin
 */
export async function getPluginResourceStats(id: string): Promise<ResourceStats> {
  const result = await invoke<ResourceStats>('get_plugin_resource_stats', { id });
  return result;
}

/**
 * Audit log entry for plugin operations
 */
export interface AuditLogEntry {
  timestamp: number;
  plugin_id: string;
  operation: string;
  status: string;
  details: string;
  user_id?: string;
}

/**
 * Get audit log entries
 */
export async function getAuditLog(pluginId?: string): Promise<AuditLogEntry[]> {
  const result = await invoke<AuditLogEntry[]>('get_audit_log', { pluginId });
  return result;
}

/**
 * Clear old audit log entries
 */
export async function clearOldAuditLogs(olderThanSeconds: number): Promise<void> {
  await invoke('clear_old_audit_logs', { olderThanSeconds });
}

/**
 * Enable sandbox isolation for plugins
 */
export async function enablePluginSandbox(
  enableSeccomp: boolean,
  allowNetwork: boolean,
  allowFilesystem: boolean,
  maxMemoryMb: number
): Promise<void> {
  await invoke('enable_plugin_sandbox', {
    enableSeccomp,
    allowNetwork,
    allowFilesystem,
    maxMemoryMb,
  });
}

/**
 * Disable sandbox isolation
 */
export async function disablePluginSandbox(): Promise<void> {
  await invoke('disable_plugin_sandbox');
}

/**
 * Get sandbox status and configuration
 */
export async function getSandboxStatus(): Promise<SandboxStatus> {
  return await invoke('get_sandbox_status');
}

/**
 * Sandbox status for frontend
 */
export interface SandboxStatus {
  enabled: boolean;
  config: SandboxConfig;
}

/**
 * Sandbox configuration
 */
export interface SandboxConfig {
  enableSeccomp: boolean;
  allowNetwork: boolean;
  allowFilesystem: boolean;
  maxMemoryMb: number;
  socketPath?: string;
}

/**
 * Plugin manager class for easier plugin management
 */
export class NativePluginManager {
  private initialized = false;

  /**
   * Initialize the plugin manager
   */
  async init(): Promise<string> {
    if (this.initialized) {
      throw new Error('Plugin manager already initialized');
    }
    const pluginsDir = await initNativePluginManager();
    this.initialized = true;
    return pluginsDir;
  }

  /**
   * Load a plugin
   */
  async load(path: string): Promise<PluginMetadata> {
    if (!this.initialized) {
      await this.init();
    }
    return loadNativePlugin(path);
  }

  /**
   * Unload a plugin
   */
  async unload(id: string): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }
    return unloadNativePlugin(id);
  }

  /**
   * List all plugins
   */
  async list(): Promise<PluginMetadata[]> {
    if (!this.initialized) {
      await this.init();
    }
    return listNativePlugins();
  }

  /**
   * Get a specific plugin
   */
  async get(id: string): Promise<PluginMetadata | null> {
    if (!this.initialized) {
      await this.init();
    }
    return getNativePlugin(id);
  }

  /**
   * Execute a command on a plugin
   */
  async execute(id: string, command: string, params?: PluginCommandParams): Promise<unknown> {
    if (!this.initialized) {
      await this.init();
    }
    return executePluginCommand(id, command, params);
  }

  /**
   * Enable or disable a plugin
   */
  async setEnabled(id: string, enabled: boolean): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }
    return setNativePluginEnabled(id, enabled);
  }

  /**
   * Scan for plugins
   */
  async scan(): Promise<number> {
    if (!this.initialized) {
      await this.init();
    }
    return scanNativePlugins();
  }

  /**
   * Reload plugins that have been modified
   */
  async reloadChanged(): Promise<string[]> {
    if (!this.initialized) {
      await this.init();
    }
    return reloadChangedPlugins();
  }

  /**
   * Get resource statistics for a plugin
   */
  async getResourceStats(id: string): Promise<ResourceStats> {
    if (!this.initialized) {
      await this.init();
    }
    return getPluginResourceStats(id);
  }

  /**
   * Get audit log entries
   */
  async getAuditLog(pluginId?: string): Promise<AuditLogEntry[]> {
    if (!this.initialized) {
      await this.init();
    }
    return getAuditLog(pluginId);
  }

  /**
   * Clear old audit log entries
   */
  async clearOldAuditLogs(olderThanSeconds: number): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }
    return clearOldAuditLogs(olderThanSeconds);
  }

  /**
   * Enable sandbox isolation for plugins
   */
  async enableSandbox(
    enableSeccomp: boolean,
    allowNetwork: boolean,
    allowFilesystem: boolean,
    maxMemoryMb: number
  ): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }
    return enablePluginSandbox(enableSeccomp, allowNetwork, allowFilesystem, maxMemoryMb);
  }

  /**
   * Disable sandbox isolation
   */
  async disableSandbox(): Promise<void> {
    if (!this.initialized) {
      await this.init();
    }
    return disablePluginSandbox();
  }

  /**
   * Get sandbox status and configuration
   */
  async getSandboxStatus(): Promise<SandboxStatus> {
    if (!this.initialized) {
      await this.init();
    }
    return getSandboxStatus();
  }
}

// Export a singleton instance
export const nativePluginManager = new NativePluginManager();
