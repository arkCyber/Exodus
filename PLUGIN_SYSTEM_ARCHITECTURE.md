# Exodus Browser Plugin System Architecture

**Date**: 2025-05-18
**Objective**: Design a hybrid plugin system compatible with Chrome Extensions and native Rust plugins

---

## Overview

Exodus Browser will support two types of plugins:
1. **Web Extensions**: Chrome Extension Manifest V3 compatible (JavaScript/TypeScript)
2. **Native Plugins**: Rust-based plugins with full system access

## Requirements

### Web Extensions Compatibility
- Support Chrome Extension Manifest V3 format
- Content scripts injection
- Background service workers
- Popup windows
- Permission system (storage, tabs, etc.)
- Web Extension API subset

### Native Rust Plugins
- Full system access via Tauri
- Rust API for browser control
- Performance-critical operations
- System-level integrations

## Architecture

### Plugin Manager

```rust
// src-tauri/src/plugin_manager.rs
pub struct PluginManager {
    web_extensions: HashMap<String, WebExtension>,
    native_plugins: HashMap<String, NativePlugin>,
    permissions: PermissionManager,
}

impl PluginManager {
    pub async fn load_extension(&mut self, manifest: ExtensionManifest) -> Result<(), PluginError>;
    pub async fn load_native_plugin(&mut self, path: PathBuf) -> Result<(), PluginError>;
    pub fn execute_extension_api(&self, extension_id: &str, api_call: ApiCall) -> Result<Value, PluginError>;
}
```

### Web Extension Bridge

```typescript
// Extension API Bridge (Frontend)
class ExtensionBridge {
  // Chrome Extension API subset
  chrome: {
    tabs: ChromeTabsAPI;
    storage: ChromeStorageAPI;
    runtime: ChromeRuntimeAPI;
  };
  
  // Communicate with Rust backend
  private async invokeRust(extensionId: string, method: string, params: any) {
    return invoke('extension_api', { extensionId, method, params });
  }
}
```

### Native Plugin Interface

```rust
// Native Plugin Trait
pub trait NativePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, context: PluginContext) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;
    fn handle_message(&mut self, message: Value) -> Result<Value, PluginError>;
}

// Plugin Context
pub struct PluginContext {
    pub browser_api: BrowserApi,
    pub storage: PluginStorage,
    pub permissions: PermissionSet,
}
```

## Manifest Format

### Web Extension Manifest (V3)
```json
{
  "manifest_version": 3,
  "name": "My Extension",
  "version": "1.0.0",
  "permissions": ["storage", "tabs"],
  "content_scripts": [{
    "matches": ["<all_urls>"],
    "js": ["content.js"]
  }],
  "background": {
    "service_worker": "background.js"
  }
}
```

### Native Plugin Manifest
```toml
[plugin]
name = "my_native_plugin"
version = "1.0.0"
type = "native"

[permissions]
system = true
network = true

[dependencies]
exodus-api = "0.1.0"
```

## Security Model

### Web Extensions
- Sandboxed execution
- Limited API access via permissions
- Content script isolation
- No direct system access

### Native Plugins
- Full system access (requires user approval)
- Code signing required
- Sandboxed where possible
- Permission prompts

## API Design

### Chrome Extension API Subset

```typescript
// Supported APIs
interface ChromeAPI {
  tabs: {
    query: (query: object) => Promise<Tab[]>;
    sendMessage: (tabId: number, message: any) => Promise<any>;
    create: (createProperties: object) => Promise<Tab>;
  };
  storage: {
    local: {
      get: (keys?: string | string[] | object) => Promise<object>;
      set: (items: object) => Promise<void>;
    };
  };
  runtime: {
    sendMessage: (message: any) => Promise<any>;
    onMessage: chrome.runtime.MessageSender;
  };
}
```

### Native Rust API

```rust
pub trait BrowserApi {
    fn create_tab(&self, url: &str) -> Result<String, PluginError>;
    fn navigate(&self, tab_id: &str, url: &str) -> Result<(), PluginError>;
    fn execute_script(&self, tab_id: &str, script: &str) -> Result<String, PluginError>;
    fn get_tabs(&self) -> Result<Vec<TabInfo>, PluginError>;
}
```

## Implementation Phases

### Phase 1: Foundation (4 weeks)
- Plugin manager structure
- Manifest parsing (Web + Native)
- Plugin loading/unloading
- Basic permission system

### Phase 2: Web Extension Bridge (6 weeks)
- Chrome API subset implementation
- Content script injection
- Background service worker support
- Storage API bridge

### Phase 3: Native Plugin System (COMPLETED ✅)
- Native plugin trait definition ✅
- Plugin loading from dynamic libraries ✅
- Rust API exposure ✅
- Permission prompts ✅
- Tauri commands for plugin management ✅
- Plugin lifecycle management ✅

**Implementation Details:**
- Created `NativePlugin` trait with lifecycle methods (on_load, on_unload, handle_message)
- Implemented `NativePluginManager` for loading/unloading plugins from .so/.dll/.dylib
- Added Tauri commands for plugin list, load, unload, enable, disable, send_message
- Implemented `PluginContext`, `BrowserApi`, `PluginStorage` structures
- Added permission system with `PermissionSet` for system, network, file, and browser access
- Created `PluginWrapper` for safe raw pointer handling from dynamic libraries

### Phase 4: Integration & Testing (IN PROGRESS)
- Plugin store UI ✅ (ExtensionsSettings.svelte)
- Installation/management flow ✅ (commands.rs + api.ts)
- Security testing ✅ (security_tests.rs)
- Performance testing ✅ (performance_tests.rs)

**Implementation Details:**
- Created ExtensionsSettings.svelte with full extension management UI
- Added Tauri commands: extension_install_crx, extension_store_list, extension_popup_url
- Implemented security tests: path traversal protection, manifest validation, permission boundaries, package security, storage isolation
- Implemented performance benchmarks: extension load, storage operations, content script injection, manifest parsing, extension list retrieval

## Plugin Directory Structure

```
~/.exodus/plugins/
├── web-extensions/
│   ├── adblocker/
│   │   ├── manifest.json
│   │   ├── content.js
│   │   └── background.js
│   └── password-manager/
├── native-plugins/
│   ├── system-monitor/
│   │   ├── plugin.toml
│   │   └── libplugin.so (or plugin.dll)
│   └── performance-optimizer/
└── disabled/
```

## Testing Strategy

### Web Extensions
- Test against Chrome Extension Test Suite
- Compatibility testing with popular extensions
- Security sandbox testing

### Native Plugins
- Memory safety testing
- Permission boundary testing
- Crash isolation testing

## Performance Considerations

### Web Extensions
- Lazy loading of extensions
- Background worker limits
- Memory quotas per extension

### Native Plugins
- Plugin isolation (separate processes if possible)
- Resource limits
- Graceful degradation on plugin failure

## Documentation

### For Extension Developers
- API reference (Chrome subset)
- Best practices
- Example extensions

### For Native Plugin Developers - COMPLETED
- Rust API reference
- Plugin development guide
- Security guidelines

## Comparison with Chrome

| Feature | Chrome | Exodus |
|---------|--------|--------|
| Web Extensions | ✅ Full | ✅ Subset (Phase 2) |
| Native Plugins | ❌ | ✅ Rust (Phase 3) |
| Rust Performance | ❌ | ✅ Full access |
| Security Model | Sandboxed | Hybrid (Sandbox + Native) |
| Plugin Store | Chrome Web Store | Local + Future Store |

## Risks and Mitigations

### Security Risks
- **Malicious extensions**: Permission system + code signing
- **Native plugin vulnerabilities**: Sandboxing + review process
- **Privilege escalation**: Strict permission boundaries

### Compatibility Risks
- **Extension incompatibility**: Test against popular extensions
- **API changes**: Versioned API contracts
- **Performance impact**: Resource limits and monitoring

## Success Metrics

- Support top 100 most popular Chrome extensions
- Native plugin performance > 10x faster than JS equivalents
- Plugin load time < 100ms
- Zero security vulnerabilities in plugin system
