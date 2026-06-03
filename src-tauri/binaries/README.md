# Exodus Core Binaries

This directory should contain the compiled `exodus-core` binary for your platform.

## Platform-specific binary names

Place your compiled binary here with the appropriate name:

- **macOS (x86_64)**: `exodus-core-x86_64-apple-darwin`
- **macOS (ARM64)**: `exodus-core-aarch64-apple-darwin`
- **Linux (x86_64)**: `exodus-core-x86_64-unknown-linux-gnu`
- **Windows (x86_64)**: `exodus-core-x86_64-pc-windows-msvc.exe`

Tauri will automatically select the correct binary based on the target platform during build.

Build the bundled dev sidecar (OpenAI-compatible API on your AI port):

```bash
pnpm build:sidecar
```

This compiles `exodus-core/` from the workspace and installs `exodus-core-<target-triple>` here. A shell **stub** may remain until you run that command; replace with a full LLM runtime when you have one.

`tauri.conf.json` already lists:


```json
"externalBin": ["binaries/exodus-core"]
```

Shell capability for the sidecar is in `capabilities/default.json` (`shell:allow-execute` with `"sidecar": true`).

## Building exodus-core

To build your CLI inference engine, run:

```bash
# For macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# For macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# For Linux
cargo build --release --target x86_64-unknown-linux-gnu

# For Windows
cargo build --release --target x86_64-pc-windows-msvc
```

Then copy the binary from `target/<target>/release/exodus-core` to this directory with the appropriate platform suffix.
