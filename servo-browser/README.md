# Exodus Browser - Pure Rust with Servo Embedding

> **"Go out from their servers, retake your technological sovereignty."**

This is the **pure Rust implementation** of Exodus Browser, embedding Servo directly as the rendering engine instead of using system WebView via Tauri.

## Architecture Change

We've pivoted from **Tauri 2.0 + System WebView** to **Pure Rust + Servo Embedding**:

### Previous Architecture (Tauri 2.0)
- **Frontend**: Svelte + TypeScript
- **Backend**: Rust via Tauri
- **Rendering**: System WebView (WebKitGTK/WebView2/WebKit)
- **Pros**: 100% web compatibility, mature ecosystem
- **Cons**: Depends on system components, less control

### New Architecture (Servo Embedding)
- **Language**: Pure Rust (no JavaScript frontend)
- **Rendering**: Servo (Rust web rendering engine)
- **Windowing**: winit + glutin (OpenGL)
- **Pros**: Complete control, pure Rust stack, no system dependencies
- **Cons**: Servo is experimental, web compatibility may be lower

## Project Structure

```
exodus/
├── servo-browser/          # Pure Rust Servo-based browser
│   ├── src/
│   │   ├── main.rs       # Entry point with winit/glutin
│   │   ├── rag.rs        # RAG system (copied from src-tauri)
│   │   ├── agent.rs      # Web Agent system (copied from src-tauri)
│   │   └── sidecar.rs    # Sidecar process management
│   └── Cargo.toml
├── src-tauri/             # Original Tauri implementation (kept for reference)
│   ├── src/
│   │   ├── rag.rs
│   │   ├── agent.rs
│   │   └── lib.rs
│   └── Cargo.toml
└── Cargo.toml             # Workspace configuration
```

## Current Status

### ✅ Completed
- Pure Rust project structure with Servo dependencies
- Window setup with winit + glutin
- Sidecar process management (exodus-core)
- RAG system module (sled embedded database)
- Web Agent action space definitions

### 🚧 In Progress
- Servo embedding integration (libservo)
- OpenGL context management for Servo rendering
- WindowMethods trait implementation for Servo callbacks

### ⏳ Pending
- Full Servo embedding with WindowMethods
- RAG system integration with Servo DOM access
- Web Agent integration with Servo control
- Browser UI implementation (address bar, tabs, etc.)

## Servo Embedding Requirements

### Prerequisites
1. **Rust Nightly**: Servo currently requires Rust Nightly
   ```bash
   rustup install nightly
   rustup default nightly
   ```

2. **Servo Dependencies**: Install Servo build dependencies
   ```bash
   # Follow Servo's build instructions
   # https://github.com/servo/servo/blob/main/PROCESS.md
   ```

3. **OpenGL**: Ensure OpenGL drivers are installed

### Setting Up Servo Embedding

1. **Clone Servo Repository** (for resources and toolchain)
   ```bash
   cd exodus
   git clone https://github.com/servo/servo.git
   ```

2. **Copy Servo Files**
   ```bash
   cp servo/rust-toolchain .
   cp servo/Cargo.lock servo-browser/
   cp -r servo/resources servo-browser/
   ```

3. **Enable Servo Feature**
   ```bash
   cd servo-browser
   cargo build --features servo
   ```

### Servo Embedding API

The Servo embedding requires implementing the `WindowMethods` trait:

```rust
impl WindowMethods for ExodusWindow {
    fn get_coordinates(&self) -> (f32, f32) {
        // Return OpenGL buffer coordinates
    }
    
    fn wake_up(&self) {
        // Wake up the event loop
    }
    
    fn prepare_for_composite(&self, width: usize, height: usize) -> (f32, f32) {
        // Prepare for rendering
    }
    
    fn on_load_started(&self) {
        // Navigation started callback
    }
    
    fn on_load_finished(&self) {
        // Navigation finished callback
    }
}
```

## Running the Browser

### Without Servo (Placeholder)
```bash
cd servo-browser
cargo run
```
This will run the window setup without Servo rendering (shows black screen).

### With Servo (Requires Setup)
```bash
cd servo-browser
cargo run --features servo
```
This will attempt to embed Servo (requires proper Servo setup).

## Advantages of Servo Embedding

1. **Pure Rust Stack**: No JavaScript, no system WebView dependencies
2. **Direct DOM Access**: Full control over DOM for RAG and Agent
3. **No Iframe Limitations**: Web Agent can directly control the page
4. **Custom Rendering**: Can implement custom rendering optimizations
5. **Technological Sovereignty**: Complete control over the rendering pipeline

## Challenges

1. **Servo Maturity**: Servo is experimental, not production-ready
2. **Web Compatibility**: Lower than system WebView
3. **Build Complexity**: Requires specific toolchain and dependencies
4. **Limited Documentation**: Embedding API is work-in-progress
5. **Performance**: May not match system WebView performance

## Next Steps

### Immediate
1. Complete Servo embedding setup (follow servo-embedding-example)
2. Implement WindowMethods trait
3. Get basic web page rendering working

### Short-term
1. Integrate RAG system with Servo DOM access
2. Integrate Web Agent with Servo control
3. Build basic browser UI (address bar, navigation)

### Long-term
1. Implement multi-view support (tabs)
2. Implement multi-window support
3. Optimize performance and web compatibility

## Alternative: Hybrid Approach

If Servo embedding proves too challenging, consider:
- Keep Tauri 2.0 for production (system WebView for compatibility)
- Use Servo embedding for experimental features
- Gradually migrate to Servo as it matures

## Resources

- [Servo Embedding Example](https://github.com/paulrouget/servo-embedding-example)
- [Servo Embedding API](https://github.com/paulrouget/servo-embedding-api)
- [Servo Documentation](https://servo.org/)
- [Building a Browser with Servo](https://servo.org/blog/2024/09/11/building-browser/)

## License

MIT License - see LICENSE file for details.

---

**Built with rage against the machine. Pure Rust, no compromises.**

⛵ *Go out from their servers.*
