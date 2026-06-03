# Exodus Browser Native Plugin System

## Overview

The Exodus Browser Native Plugin System allows developers to create high-performance plugins using Rust that run natively within the browser. These plugins are loaded as dynamic libraries (`.dylib` on macOS, `.so` on Linux, `.dll` on Windows) and can interact with the browser through a type-safe API.

## Architecture

### Components

1. **Plugin SDK** (`exodus-plugin-sdk/`) - Shared crate providing types and macros for plugin development
2. **Plugin Manager** (`src-tauri/src/native_plugins/mod.rs`) - Backend plugin loading and management
3. **Frontend Bindings** (`src/lib/nativePlugins.ts`) - TypeScript API for frontend integration
4. **Example Plugin** (`examples/native-plugin/`) - Reference implementation

### Plugin Lifecycle

```
Load .dylib → Validate → Initialize → Execute Commands → Cleanup → Unload
```

## Quick Start

### 1. Create a New Plugin

```bash
mkdir my-plugin
cd my-plugin
cargo init --lib
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[workspace]
# Standalone plugin

[lib]
crate-type = ["cdylib"]

[dependencies]
exodus-plugin-sdk = { path = "../../exodus-plugin-sdk" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 3. Implement the Plugin

```rust
use exodus_plugin_sdk::{ExodusPlugin, PluginMetadata, PluginContext, export_plugin};
use serde_json::Value;

pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                id: "my-plugin".to_string(),
                name: "My Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "My awesome plugin".to_string(),
                author: "Your Name".to_string(),
                permissions: vec!["storage".to_string()],
                api_version: exodus_plugin_sdk::PLUGIN_API_VERSION.to_string(),
            },
        }
    }
}

impl ExodusPlugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn initialize(&mut self, context: PluginContext) -> Result<(), String> {
        println!("Initializing plugin: {:?}", context.plugin_id);
        Ok(())
    }
    
    fn handle_command(&self, command: &str, params: Value) -> Result<Value, String> {
        match command {
            "my_command" => {
                Ok(serde_json::json!({ "result": "success" }))
            }
            _ => Err(format!("Unknown command: {}", command))
        }
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        println!("Cleaning up plugin");
        Ok(())
    }
}

export_plugin!(MyPlugin);
```

### 4. Build the Plugin

```bash
cargo build --release
```

Output:
- macOS: `target/release/libmy_plugin.dylib`
- Linux: `target/release/libmy_plugin.so`
- Windows: `target/release/my_plugin.dll`

### 5. Install the Plugin

```bash
# macOS
cp target/release/libmy_plugin.dylib ~/Library/Application\ Support/com.exodus.browser/plugins/native/

# Linux
cp target/release/libmy_plugin.so ~/.local/share/com.exodus.browser/plugins/native/

# Windows
copy target\release\my_plugin.dll %APPDATA%\com.exodus.browser\plugins\native\
```

## Plugin API

### ExodusPlugin Trait

All plugins must implement the `ExodusPlugin` trait:

```rust
pub trait ExodusPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin with context
    fn initialize(&mut self, context: PluginContext) -> Result<(), String>;
    
    /// Handle a command from the browser
    fn handle_command(&self, command: &str, params: serde_json::Value) -> Result<serde_json::Value, String>;
    
    /// Cleanup before unloading
    fn cleanup(&mut self) -> Result<(), String>;
}
```

### PluginMetadata

```rust
pub struct PluginMetadata {
    pub id: String,           // Unique identifier (alphanumeric, hyphens, underscores)
    pub name: String,         // Display name (max 100 chars)
    pub version: String,      // Semantic version (max 50 chars)
    pub description: String,  // Description (max 500 chars)
    pub author: String,       // Author name
    pub permissions: Vec<String>,  // Required permissions
    pub api_version: String,  // Must match "1.0.0"
}
```

### PluginContext

```rust
pub struct PluginContext {
    pub plugin_id: String,              // Plugin ID
    pub data_dir: PathBuf,             // Plugin data directory
    pub config: HashMap<String, String>, // Configuration
}
```

## Permissions

### Available Permissions

- `storage` - Access to local storage
- `network` - Network access
- `tabs` - Access to browser tabs
- `bookmarks` - Access to bookmarks
- `history` - Access to browsing history (sensitive)
- `downloads` - Access to downloads
- `cookies` - Access to cookies (sensitive)
- `passwords` - Access to passwords (sensitive)

### Sensitive Permissions

Sensitive permissions (history, cookies, passwords) require explicit user approval and are subject to additional security checks.

## Frontend Integration

### Using the TypeScript API

```typescript
import { nativePluginManager } from '@/lib/nativePlugins';

// Initialize
await nativePluginManager.init();

// Load a plugin
const metadata = await nativePluginManager.load('/path/to/plugin.dylib');

// Execute a command
const result = await nativePluginManager.execute('plugin-id', 'command', { param: 'value' });

// List all plugins
const plugins = await nativePluginManager.list();

// Enable/disable a plugin
await nativePluginManager.setEnabled('plugin-id', false);

// Unload a plugin
await nativePluginManager.unload('plugin-id');
```

### Function API

```typescript
import {
  initNativePluginManager,
  loadNativePlugin,
  unloadNativePlugin,
  listNativePlugins,
  getNativePlugin,
  executePluginCommand,
  setNativePluginEnabled,
  scanNativePlugins
} from '@/lib/nativePlugins';
```

## Security

### Aerospace-Level Security

The plugin system implements aerospace-level security measures:

1. **Path Validation** - Plugins must be located in the designated plugins directory
2. **File Permissions** - Unix systems check for world-writable files
3. **Metadata Validation** - All metadata fields are validated for length and format
4. **API Versioning** - Plugins must match the current API version
5. **Permission System** - Plugins declare required permissions upfront
6. **Sensitive Permission Approval** - User approval required for sensitive operations

### Security Constants

```rust
const MAX_PLUGIN_NAME_LENGTH: usize = 100;
const MAX_PLUGIN_VERSION_LENGTH: usize = 50;
const MAX_PLUGIN_DESCRIPTION_LENGTH: usize = 500;
const MAX_PLUGIN_ID_LENGTH: usize = 100;
```

## Testing

### Unit Tests

Run unit tests for the plugin system:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib native_plugins::tests
```

### Integration Tests

Run integration tests (requires actual plugin file):

```bash
# First build the example plugin
cd examples/native-plugin
cargo build --release

# Copy to test location
mkdir -p /tmp/exodus_test_plugins/native
cp target/release/libexodus_example_plugin.dylib /tmp/exodus_test_plugins/native/

# Run integration test
cd ../..
cargo test --manifest-path src-tauri/Cargo.toml --lib native_plugins::integration_test::test_load_actual_plugin -- --ignored
```

### Verify Plugin Symbols

```bash
nm -g /path/to/plugin.dylib | grep exodus_plugin
```

Expected output:
```
0000000000002c64 T _exodus_plugin_deinit
0000000000002d40 T _exodus_plugin_init
0000000000003228 T _exodus_plugin_version
```

## Example Plugin

The example plugin (`examples/native-plugin/`) demonstrates:

- Plugin structure and implementation
- Command handling (ping, increment, get_counter, echo)
- State management with thread-safe counter
- Permission declaration
- Build configuration

### Building the Example

```bash
cd examples/native-plugin
cargo build --release
```

### Testing the Example

```bash
# Load the plugin
await nativePluginManager.load('/path/to/libexodus_example_plugin.dylib');

# Test ping command
const ping = await nativePluginManager.execute('example-plugin', 'ping');
// Returns: { status: "pong", message: "Example plugin is working!" }

# Test increment
await nativePluginManager.execute('example-plugin', 'increment');
const counter = await nativePluginManager.execute('example-plugin', 'get_counter');
// Returns: { counter: 1 }
```

## Troubleshooting

### Plugin Fails to Load

1. Check the plugin file exists in the correct directory
2. Verify the plugin symbols are present using `nm`
3. Check the API version matches "1.0.0"
4. Review the browser console for error messages

### Command Execution Fails

1. Verify the command name matches exactly
2. Check the parameters are valid JSON
3. Ensure the plugin is enabled
4. Review the plugin's command handling code

### Permission Errors

1. Verify the plugin declares all required permissions
2. Check if sensitive permissions need user approval
3. Review the permission system logs

## Advanced Topics

### Hot Reload

Hot reload support allows plugins to be reloaded without restarting the browser. This feature is planned for future releases.

### Plugin Signing

Plugin signing verification ensures plugin authenticity and integrity. This feature is planned for future releases.

### Custom Permissions

Plugins can declare custom permissions:

```rust
permissions: vec!["storage".to_string(), "custom:my_permission".to_string()]
```

### State Management

Plugins can maintain state using thread-safe data structures:

```rust
use std::sync::{Arc, Mutex};

pub struct MyPlugin {
    counter: Arc<Mutex<u32>>,
}
```

## API Reference

### Tauri Commands

The plugin system exposes the following Tauri commands:

- `init_native_plugin_manager` - Initialize the plugin manager
- `load_native_plugin` - Load a plugin from a file path
- `unload_native_plugin` - Unload a plugin
- `list_native_plugins` - List all loaded plugins
- `get_native_plugin` - Get metadata for a specific plugin
- `execute_plugin_command` - Execute a command on a plugin
- `set_native_plugin_enabled` - Enable or disable a plugin
- `scan_native_plugins` - Scan and load all plugins from the plugins directory

## Contributing

When contributing to the plugin system:

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Ensure aerospace-level security standards
5. Test on all supported platforms (macOS, Linux, Windows)

## License

The plugin system is part of the Exodus Browser project and follows the same license terms.

## Support

For issues, questions, or contributions, please refer to the main Exodus Browser repository.
