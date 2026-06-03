# ⛵ Exodus Browser

**Languages:** [English](README.md) · [简体中文](README.zh-CN.md)

> **"Go out from their servers, retake your technological sovereignty."**

[![CI](https://github.com/arksong/Exodus/actions/workflows/ci.yml/badge.svg)](https://github.com/arksong/Exodus/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Exodus is a next-generation, privacy-absolute AI browser designed for geeks, hackers, and open-source purists. **Pure Rust architecture with local AI inference**, Exodus brings complete technological sovereignty—your data never leaves your machine.

**First-time publish?** See [docs/GITHUB_PUBLISH.md](docs/GITHUB_PUBLISH.md). **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md).

## 🏗️ Architecture

Exodus has **two implementations**:

### 1. Pure Rust + wry WebView (Research Platform) �
- **Language**: Pure Rust (no JavaScript)
- **Rendering**: wry WebView (system WebView components)
- **Windowing**: winit
- **AI Engine**: `exodus-core` as native Sidecar process
- **Location**: `servo-browser/` directory

**Advantages**:
- Pure Rust architecture (no JavaScript frontend)
- Full control over windowing and event loop
- 100% web compatibility via system WebView
- Foundation for custom rendering experiments

**Status**: ✅ Working (builds and runs successfully)
- RAG system integrated (modules ready)
- Web Agent system integrated (modules ready)
- Sidecar management implemented
- **Note**: Originally planned Servo embedding was replaced with wry due to complex dependency issues

### 2. Tauri 2.0 + System WebView (Production-Ready) 📦
- **Frontend**: Svelte + TypeScript
- **Backend**: Rust via Tauri
- **Rendering**: System WebView (WebKitGTK/WebView2/WebKit)
- **AI Engine**: `exodus-core` as native Sidecar process
- **Location**: `src-tauri/` directory

**Advantages**:
- 100% web compatibility
- Mature ecosystem
- Complete UI framework
- Production-ready

**Status**: ✅ Fully implemented (MVP complete)
- Microservice bridge implemented (JSON-RPC 2.0)
- RAG service integration complete
- Firefox-style sidebar UI redesign
- All tests passing (8/8 Rust integration tests)
- Native throttle implementation (no external dependencies)

## 🛡️ Why Exodus?

Big Tech turned browsers into telemetry spyware and local AI into a subscription prison. Windows Copilot watches your screen; Chrome sends your inputs to cloud advertisers.

**Exodus is your escape route.** It packages a lightning-fast local LLM runner (`exodus-core`) as a native background sidecar. Your data never leaves your machine. Your hardware obeys nobody but you.

## ✨ Mind-Blowing Features

### 🔒 Air-Gapped Privacy Airbag
Absolute data isolation. Run full-context webpage summarization, code auditing, and translation 100% offline. No telemetry, no phoning home, no cloud dependencies.

### 🔍 Offline Omnibox (RAG)
A true memory palace. Exodus continuously vectors your local browsing history and bookmarks into an embedded vector database via native IPC. Search your history using pure natural language (e.g., `/ask that async rust lifetime compilation error table I saw last Friday`).

### 🤖 Local Web Agent (Embodied Autopilot)
Let local models control the web. Exodus parses DOM trees into lightweight tokens, feeds them directly into your local GPU (via CUDA/ROCm optimized rust engine), and executes multi-step web automations locally.

## 📁 Project Structure

```
exodus/
├── servo-browser/          # Pure Rust browser (Research Platform)
│   ├── src/
│   │   ├── main.rs       # Entry point with winit + wry
│   │   ├── rag.rs        # RAG system
│   │   ├── agent.rs      # Web Agent system
│   │   └── sidecar.rs    # Sidecar process management
│   ├── Cargo.toml
│   └── README.md         # Servo-specific documentation
├── src-tauri/             # Tauri implementation (Production-Ready)
│   ├── src/
│   │   ├── rag.rs
│   │   ├── agent.rs
│   │   └── lib.rs
│   ├── binaries/         # exodus-core binary placement
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                   # Tauri frontend (Vue 3 + TypeScript)
│   ├── views/
│   │   └── BrowserPage.vue  # Main browser shell
│   ├── routes/            # SvelteKit routes retired (see routes/RETIRED.md)
│   └── main.ts            # Vite entry (index.html)
└── Cargo.toml             # Workspace configuration
```

## 🚀 Quick Start

### Option 1: Pure Rust Browser (Research Platform)

**Prerequisites**:
- Rust (stable)
- OpenGL drivers (for wry WebView)

**Setup**:
```bash
cd servo-browser
cargo build
cargo run
```

**Note**: This implementation uses wry WebView for rendering. For details, see `SERVO_IMPLEMENTATION_REPORT.md`.

### Option 2: Tauri-Based Browser (Production-Ready)

**Prerequisites**:
- Rust (stable)
- Node.js 18+
- pnpm

**Setup**:
```bash
# Install dependencies
pnpm install

# Place exodus-core binary in src-tauri/binaries/
# See src-tauri/binaries/README.md

# Run development server
pnpm tauri dev
```

## ⚙️ Setting Up the Sidecar (exodus-core)

The Sidecar is your local LLM inference engine. Compile and place the binary:

### 1. Compile exodus-core
```bash
# For macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# For macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# For Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# For Linux (ARM64)
cargo build --release --target aarch64-unknown-linux-gnu

# For Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. Place the Binary

**For Tauri implementation**: Copy to `src-tauri/binaries/` with platform suffix
**For Servo implementation**: Copy to `servo-browser/binaries/` with platform suffix

Platform suffixes:
- **macOS (ARM64)**: `exodus-core-aarch64-apple-darwin`
- **macOS (Intel)**: `exodus-core-x86_64-apple-darwin`
- **Linux (x86_64)**: `exodus-core-x86_64-unknown-linux-gnu`
- **Linux (ARM64)**: `exodus-core-aarch64-unknown-linux-gnu`
- **Windows**: `exodus-core-x86_64-pc-windows-msvc.exe`

## 🎯 Implementation Status

### Tauri Implementation ✅ (Production-Ready)
- **Phase 1**: Text Selection AI Summary ✅
- **Phase 2**: Offline Omnibox (RAG) ✅
- **Phase 3**: Web Agent (DOM compression, action space) ✅
- **Recent Updates**: Navigation controls, frame binding, improved AI response formatting, status bar enhancements, icon brightness optimization

### Pure Rust Implementation ✅ (Working - Research Platform)
- **Phase 1**: Basic browser (winit + wry) ✅
- **Phase 2**: RAG system integration ✅ (modules ready)
- **Phase 3**: Web Agent integration ✅ (modules ready)
- **Phase 4**: Sidecar management ✅
- **Status**: Builds and runs successfully, ready for feature integration

## 🔧 Configuration

### Sidecar Configuration

Both implementations use the same Sidecar pattern:
- Binary placed in `binaries/` directory
- Spawned on application startup
- Monitored via stdout/stderr
- Automatically terminated on exit

### RAG Database

Uses sled embedded database (`exodus_rag_db`):
- Stores webpage content with embeddings
- Provides semantic search via cosine similarity
- Runs entirely offline

## 🗺️ Roadmap

### Pure Rust Implementation (servo-browser/)
- [x] Basic browser with winit + wry
- [x] RAG system integration (modules ready)
- [x] Web Agent system integration (modules ready)
- [x] Sidecar process management
- [ ] Navigation event handlers for RAG capture
- [ ] Basic UI controls (address bar, navigation buttons)
- [ ] DOM access integration for Web Agent
- [ ] Multi-view support (tabs)
- [ ] Performance optimization

### Tauri Implementation (src-tauri/)
- [x] Text Selection AI Summary
- [x] Offline Omnibox (RAG)
- [x] Web Agent (DOM compression, action space)
- [x] Navigation controls (back, forward, reload)
- [x] Frame binding for better DOM access
- [x] Improved AI response formatting
- [x] Status bar with agent status, AI model, online/offline display
- [x] Icon brightness optimization (Chrome-aligned)
- [x] Wallpaper system with automatic configuration
- [x] DMG installer generation for macOS
- [x] Window management improvements (logical pixel sizing, Retina display support)
- [x] Asset protocol support for local resource access
- [x] Enhanced window initialization flow
- [ ] Enhanced error handling
- [ ] Performance testing

## 🤝 Contributing

We welcome contributions from fellow geeks and privacy advocates. Focus on the Servo-based implementation for the pure Rust vision.

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- Servo team for the pure Rust rendering engine
- Tauri team for the excellent framework
- The open-source AI community
- All privacy advocates fighting for digital sovereignty

---

**Built with rage against the machine. Pure Rust, no compromises.**

⛵ *Go out from their servers.*

## 🚀 Quick Start

### Prerequisites

- **Rust**: latest stable (1.70+)
- **Node.js**: 18+
- **pnpm**: 8+ (recommended) or npm/yarn
- **GPU**: CUDA/ROCm support (for local LLM inference)
- **macOS**: 10.13+ (for macOS builds)
- **Linux**: x86_64 or ARM64 with OpenGL support
- **Windows**: 10+ with WebView2

### Development

```bash
# Install dependencies
pnpm install

# Run development server
pnpm tauri dev

# Typecheck, frontend tests, Rust tests, invoke sync
pnpm verify

# Quick verification (typecheck + frontend tests only)
pnpm verify:quick
```

### AI, inference, and Hermes agent

**Integration guide:** [docs/AI_HERMES_INTEGRATION.md](docs/AI_HERMES_INTEGRATION.md) · [中文版](docs/AI_HERMES_INTEGRATION.zh-CN.md)

- Local LLM: **Allama** on `ai_port` (default 11435) + **InferenceEngine**
- **Hermes** agent: Analysis tasks → Allama HTTP; agent panel uses `hermes_analyze_page` with sidebar stream fallback
- Tests: `pnpm test:ai-hermes` · Integration: `pnpm test:integration`

### Web Extensions (Tauri)

**Developer guide:** [docs/EXTENSIONS_DEV.md](docs/EXTENSIONS_DEV.md) (English) · [docs/EXTENSIONS_DEV.zh-CN.md](docs/EXTENSIONS_DEV.zh-CN.md) (中文) — Manifest V3, Chrome APIs, Allama bridge, testing, samples.

- Dev extensions live under [`extensions/`](extensions/) and are scanned automatically in development builds:
  - **`sample-hello`** (v1.1) — reference MV3 extension: storage, tabs, runtime messaging, alarms, notifications, action badge, CSS + content script markers, popup dashboard.
  - **`sample-all-frames`** — `all_frames` + CSS injection markers for top vs iframe.
  - **`sample-net-rules`** — `declarativeNetRequest` + `webRequest` block rules for fake host `exodus-blocked.test` only (safe hand-test).
  - **`test-host-perms-a` / `test-host-perms-b`** — install-time host permission prompt hand-tests.

**Automated tests** (manifest, JS syntax, load/inject, quality bar):

```bash
pnpm test:extensions
```
- **Hand-test install prompts**: enable “Ask before granting extension site access on install”, then install `test-host-perms-a` and `test-host-perms-b` (folder or Rescan) — two dialogs should appear in order with extension **names**, not ids.
- **Hand-test revoke**: Settings → Web Extensions → **Site access** → **Revoke all**, then navigate with that extension — host access should be blocked.
- Settings → **Web Extensions**: install, enable, per-extension **Site access** (revoke `host_permissions`), install-time host confirmation.
- Settings → **Privacy & memory** → **Site permissions**: camera / microphone / geolocation decisions per origin.

**CRX3 signature check (optional integration test)** — point at a real Chrome Web Store `.crx` export:

```bash
EXODUS_TEST_CRX_PATH=/path/to/extension.crx \
  cargo test -p exodus-tauri verify_real_webstore_crx_when_env_set -- --ignored
```

### Building

```bash
# Build for production (current platform)
pnpm tauri build

# Build for specific platform
pnpm tauri build --target aarch64-apple-darwin  # macOS ARM64
pnpm tauri build --target x86_64-apple-darwin     # macOS Intel
pnpm tauri build --target x86_64-unknown-linux-gnu  # Linux x86_64
pnpm tauri build --target x86_64-pc-windows-msvc    # Windows
```

**Build Outputs**:
- macOS: `target/aarch64-apple-darwin/release/bundle/dmg/Exodus_0.1.0_aarch64.dmg`
- Linux: `target/x86_64-unknown-linux-gnu/release/bundle/deb/exodus_0.1.0_amd64.deb`
- Windows: `target/x86_64-pc-windows-msvc/release/bundle/msi/Exodus_0.1.0_x64_en-US.msi`

## ⚙️ Setting Up the Sidecar (exodus-core)

The Sidecar is your local LLM inference engine. Compile and place the binary:

### 1. Compile exodus-core

```bash
# For macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# For macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# For Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# For Linux (ARM64 - domestic innovation/Raspberry Pi)
cargo build --release --target aarch64-unknown-linux-gnu

# For Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. Place the Binary

Copy the compiled binary to `src-tauri/binaries/` with the appropriate platform suffix:

- **macOS (ARM64)**: `exodus-core-aarch64-apple-darwin`
- **macOS (Intel)**: `exodus-core-x86_64-apple-darwin`
- **Linux (x86_64)**: `exodus-core-x86_64-unknown-linux-gnu`
- **Linux (ARM64)**: `exodus-core-aarch64-unknown-linux-gnu`
- **Windows**: `exodus-core-x86_64-pc-windows-msvc.exe`

See `src-tauri/binaries/README.md` for detailed instructions.

## 🎯 MVP Features

### Phase 1: Text Selection AI Summary ✅
- **WebView Rendering**: Loads websites in main browser area
- **Text Selection**: Detects when users select text on webpages
- **AI Popup**: Shows "✨ AI Summary" button near selected text
- **Local Inference**: Calls Sidecar API at `http://localhost:11434/v1/chat/completions`
- **Streaming Response**: Real-time streaming in right sidebar

### Phase 2: Offline Omnibox (RAG) ✅
- **Data Capture**: Async extraction of page title and body text
- **Vector Database**: Embedded Rust vector store (sled + cosine similarity)
- **Semantic Search**: Natural language queries with `/ask` prefix
- **Visual Results**: Context-aware result cards with match scores

### Phase 3: Web Agent ✅
- **DOM Compression**: Semantic pruning of DOM trees
- **Action Space**: Strongly typed Rust enum for agent actions
- **Local Inference**: Tool-calling optimized local models
- **Automation Pipeline**: Multi-step web automation without cloud

### Phase 4: UI Enhancements ✅
- **Status Bar**: Displays agent status, AI model, online/offline status
- **Icon Brightness**: Chrome-aligned icon brightness for address bar and bookmarks
- **Wallpaper System**: Brave-style new tab wallpapers with automatic configuration

## 📁 Project Structure

```
exodus/
├── src/                      # Frontend (Vue 3 + TypeScript)
│   ├── views/
│   │   └── BrowserPage.vue   # Main browser shell
│   ├── routes/               # Archived Svelte (see routes/RETIRED.md)
│   └── main.ts               # Vite entry (index.html)
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs          # Sidecar process management
│   ├── binaries/           # Platform-specific binaries
│   ├── Cargo.toml
│   └── tauri.conf.json     # Tauri configuration
└── package.json
```

## 🔧 Configuration

### Sidecar Configuration

The Sidecar is configured in `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "externalBin": [
      "binaries/exodus-core"
    ]
  },
  "plugins": {
    "shell": {
      "scope": [
        {
          "allow": [
            {
              "name": "sidecar/exodus-core",
              "cmd": "exodus-core",
              "args": true
            }
          ]
        }
      ]
    }
  }
}
```

### Sidecar Process Management

The Rust backend in `src-tauri/src/lib.rs`:
- Spawns Sidecar process on startup
- Monitors stdout/stderr for logging
- Automatically terminates on app exit
- Passes CLI arguments (e.g., `--port 11434`)

## 🔌 API Endpoints

The Sidecar should expose an OpenAI-compatible API:

```
POST http://localhost:11434/v1/chat/completions
```

Example request:

```json
{
  "model": "llama2",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful AI assistant."
    },
    {
      "role": "user",
      "content": "Summarize this text: ..."
    }
  ],
  "stream": true
}
```

## 🗺️ Roadmap

### Phase 2: Enhanced Features
- [ ] Tab management (add, close, switch)
- [ ] History navigation (back/forward)
- [ ] Bookmarks system
- [ ] Settings panel (model selection, port config)
- [ ] Multiple AI models support
- [ ] RAG integration with vector database
- [ ] Offline Omnibox with `/ask` prefix

### Phase 3: Web Agent
- [ ] DOM compression script injection
- [ ] Rust action space enum definitions
- [ ] Local inference loop for tool-calling
- [ ] WebView execution bridge
- [ ] Multi-step automation pipelines

### Phase 4: Advanced Features
- [ ] Extensions system
- [ ] Custom themes
- [ ] Keyboard shortcuts
- [ ] Privacy mode
- [ ] Sync across devices (optional, encrypted)

## 🐛 Troubleshooting

### Sidecar Not Found

If you see "Failed to find sidecar" error:
1. Ensure binary is placed in `src-tauri/binaries/`
2. Check binary name matches your platform
3. Verify binary has execute permissions

### AI API Connection Failed

If AI summary fails:
1. Ensure Sidecar is running (check logs)
2. Verify Sidecar is listening on port 11434
3. Check API endpoint matches Sidecar configuration

## 💻 Development

### Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## 🤝 Contributing

We welcome contributions from fellow geeks and privacy advocates. Please read our contributing guidelines before submitting PRs.

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- Tauri team for the excellent framework
- The open-source AI community
- All privacy advocates fighting for digital sovereignty

---

**Built with rage against the machine. For the geeks, by the geeks.**

⛵ *Go out from their servers.*
