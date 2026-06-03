# Exodus Browser Native Plugin Example

This is a sample native plugin for Exodus Browser written in Rust.

## Building the Plugin

```bash
cd examples/native-plugin
cargo build --release
```

On macOS, this will create a `.dylib` file in `target/release/libexodus_example_plugin.dylib`.
On Linux, it will create a `.so` file.
On Windows, it will create a `.dll` file.

## Installing the Plugin

Copy the compiled library to the Exodus Browser plugins directory:

```bash
# macOS
cp target/release/libexodus_example_plugin.dylib ~/Library/Application\ Support/com.exodus.browser/plugins/native/

# Linux
cp target/release/libexodus_example_plugin.so ~/.local/share/com.exodus.browser/plugins/native/

# Windows
copy target\release\exodus_example_plugin.dll %APPDATA%\com.exodus.browser\plugins\native\
```

## Plugin API

The plugin implements the following commands:

### `ping`
Returns a pong response to verify the plugin is working.

```json
{
  "command": "ping"
}
```

Response:
```json
{
  "status": "pong",
  "message": "Example plugin is working!"
}
```

### `increment`
Increments an internal counter.

```json
{
  "command": "increment"
}
```

Response:
```json
{
  "counter": 1
}
```

### `get_counter`
Gets the current counter value.

```json
{
  "command": "get_counter"
}
```

Response:
```json
{
  "counter": 5
}
```

### `echo`
Echoes back the provided parameters.

```json
{
  "command": "echo",
  "params": {
    "message": "Hello, World!"
  }
}
```

Response:
```json
{
  "message": "Hello, World!"
}
```

## Creating Your Own Plugin

1. Create a new Rust project with `cargo new --lib my-plugin`
2. Add the following to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

3. Implement the `ExodusPlugin` trait
4. Export the required symbols:
   - `exodus_plugin_version`
   - `exodus_plugin_init`
   - `exodus_plugin_deinit`

See `src/lib.rs` for a complete example.

## Plugin Metadata

Your plugin must provide metadata including:
- `id`: Unique identifier (alphanumeric, hyphens, underscores only)
- `name`: Human-readable name
- `version`: Semantic version
- `description`: Plugin description
- `author`: Plugin author
- `permissions`: List of required permissions
- `api_version`: Plugin API version (must match browser version)

## Security Considerations

- Plugins are loaded from a specific plugins directory only
- File permissions are validated on Unix systems
- Plugin metadata is validated before loading
- API version compatibility is checked
