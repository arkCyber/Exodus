# ⛵ Exodus 浏览器

**语言：** [English](README.md) · [简体中文](README.zh-CN.md)

> **「走出他们的服务器，夺回你的技术主权。」**

[![CI](https://github.com/YOUR_USER/Exodus/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USER/Exodus/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Exodus 是一款面向极客、黑客与开源原教旨主义者的**下一代隐私优先 AI 浏览器**。采用 **纯 Rust 架构 + 本地 AI 推理**，实现完整的技术主权——**你的数据永不离开本机**。

**首次发布 GitHub？** 见 [docs/GITHUB_PUBLISH.md](docs/GITHUB_PUBLISH.md)。**参与贡献：** [CONTRIBUTING.md](CONTRIBUTING.md)（英文）。

## 🏗️ 架构

Exodus 提供 **两套实现**：

### 1. 纯 Rust + wry WebView（研究平台）

- **语言**：纯 Rust（无 JavaScript 前端）
- **渲染**：wry WebView（系统 WebView 组件）
- **窗口**：winit
- **AI 引擎**：`exodus-core` 作为原生 Sidecar 进程
- **目录**：`servo-browser/`

**优势**：完全掌控窗口与事件循环；通过系统 WebView 获得完整 Web 兼容性；便于自定义渲染实验。

**状态**：✅ 可编译运行；RAG / Web Agent 模块已集成；Sidecar 管理已实现。  
**说明**：原计划嵌入 Servo，因依赖复杂已改用 wry，详见 `SERVO_IMPLEMENTATION_REPORT.md`。

### 2. Tauri 2.0 + 系统 WebView（生产就绪）

- **前端**：Svelte + TypeScript
- **后端**：Rust（Tauri）
- **渲染**：系统 WebView（WebKitGTK / WebView2 / WebKit）
- **AI 引擎**：`exodus-core` Sidecar
- **目录**：`src-tauri/`

**优势**：Web 生态成熟、UI 完整、适合日常开发与发布。

**状态**：✅ MVP 已完成

## 🛡️ 为什么选择 Exodus？

科技巨头把浏览器做成遥测间谍软件，把本地 AI 变成订阅牢笼。Windows Copilot 监视你的屏幕，Chrome 把输入送给云端广告商。

**Exodus 是出路。** 它将高速本地 LLM 运行时（`exodus-core`）打包为原生后台 Sidecar。**数据不出本机，算力只听你的。**

## ✨ 核心能力

### 🔒 气隙级隐私

网页摘要、代码审计、翻译等可在 **100% 离线** 下完成，无遥测、无回传、无云依赖。

### 🔍 离线全能地址栏（RAG）

将本地浏览历史与书签向量化存入嵌入式数据库，通过自然语言检索（例如：`/ask 上周五看到的 async rust 生命周期错误表`）。

### 🤖 本地 Web Agent

解析 DOM 为轻量 token，在本地 GPU（CUDA/ROCm 优化的 Rust 引擎）上推理，多步自动化 **无需云端**。

### 📡 应用生命周期（Rust）

`app_lifecycle` 模块监控启动→运行→关闭，结构化日志（`lifecycle.log`），预设自动修复（窗口恢复、前端重载、服务重启等）。详见 `src-tauri/src/app_lifecycle.rs`。

## 📁 项目结构

```
exodus/
├── servo-browser/       # 纯 Rust 浏览器（研究）
├── src-tauri/           # Tauri 后端（生产）
│   ├── src/
│   │   ├── app_lifecycle.rs   # 生命周期与自动修复
│   │   ├── lifecycle_log.rs   # 结构化日志
│   │   ├── rag.rs / agent.rs
│   │   └── lib.rs
│   └── binaries/        # exodus-core 二进制
├── src/                 # Svelte 前端
├── extensions/          # 开发用 Web 扩展样例
├── scripts/             # 构建、测试、验证脚本
└── docs/                # 文档（含 GitHub 发布指南）
```

## 🚀 快速开始

### 环境要求

- Rust（stable）
- Node.js 18+、pnpm 9+
- （可选）支持 CUDA/ROCm 的 GPU，用于本地 LLM

### Tauri 版（推荐）

```bash
pnpm install

# 将 exodus-core 放到 src-tauri/binaries/（见该目录 README）
pnpm tauri dev
```

### 纯 Rust 研究版

```bash
cd servo-browser
cargo build
cargo run
```

### 验证与测试

```bash
pnpm verify              # 完整检查（类型、前端、Rust、命令同步等）
pnpm test:quick          # 快速测试
pnpm test:rust           # Rust 测试（默认跳过 IM，见下）
```

**IM / 即时通讯测试**：网络未配置前默认跳过，启用：

```bash
./scripts/cargo-test-im.sh
# 或
pnpm test:rust:im
```

## ⚙️ 配置 Sidecar（exodus-core）

Sidecar 是本地 LLM 推理引擎。

### 1. 编译

```bash
# macOS (ARM64)
cargo build --release --target aarch64-apple-darwin

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Linux (ARM64)
cargo build --release --target aarch64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### 2. 放置二进制

复制到 `src-tauri/binaries/`，文件名带平台后缀，例如：

- `exodus-core-aarch64-apple-darwin`
- `exodus-core-x86_64-unknown-linux-gnu`
- `exodus-core-x86_64-pc-windows-msvc.exe`

详见 `src-tauri/binaries/README.md`。

### 3. API

Sidecar 提供 OpenAI 兼容接口：

```
POST http://localhost:11434/v1/chat/completions
```

## 🤖 AI 与 Hermes 智能体

**集成说明：** [docs/AI_HERMES_INTEGRATION.zh-CN.md](docs/AI_HERMES_INTEGRATION.zh-CN.md)

- 本地推理：**Allama**（默认端口 11435）+ **InferenceEngine**
- 智能体：**Hermes**（Analysis 任务 → Allama HTTP）
- 智能体面板 `ask:` / Ask AI：优先 `hermes_analyze_page`，失败回退侧边栏 HTTP 流式
- 测试：`pnpm test:ai-hermes` · 集成：`pnpm test:integration`（扩展 + AI/Hermes + invoke 同步）

## 🧩 Web 扩展（开发）

**开发者文档：** [docs/EXTENSIONS_DEV.zh-CN.md](docs/EXTENSIONS_DEV.zh-CN.md)（中文）· [docs/EXTENSIONS_DEV.md](docs/EXTENSIONS_DEV.md)（English）— MV3 清单、Chrome API 子集、`window.exodus.allama`、自动化测试与样例说明。

- 样例位于 `extensions/`（`sample-hello`、`sample-all-frames`、`sample-net-rules`、主机权限测试扩展等），开发构建会自动扫描。
- **设置 → Web 扩展**：安装、启用、站点权限、安装时主机权限确认。
- **自动测试**：`pnpm test:extensions`（manifest / JS 语法 / 加载注入 / 源码质量门禁）；已接入 `pnpm verify`。
- **网络规则手测**：启用 `sample-net-rules` 后访问 `https://exodus-blocked.test/`（假域名，不影响真实站点）。
- **设置 → 隐私与内存 → 站点权限**：摄像头 / 麦克风 / 地理位置。

可选 CRX 签名校验测试：

```bash
EXODUS_TEST_CRX_PATH=/path/to/extension.crx \
  cargo test -p exodus-tauri verify_real_webstore_crx_when_env_set -- --ignored
```

## 🎯 实现状态（摘要）

| 模块 | Tauri 生产版 | 纯 Rust 研究版 |
|------|-------------|----------------|
| 基础浏览 / WebView | ✅ | ✅ |
| 划词 AI 摘要 | ✅ | 模块就绪 |
| 离线 RAG / Omnibox | ✅ | 模块就绪 |
| Web Agent | ✅ | 模块就绪 |
| 导航 / 多标签等 | 部分 | 规划中 |

## 🔧 配置要点

- **Sidecar**：启动时拉起，退出时终止；端口等见 `tauri.conf.json`。
- **RAG**：嵌入式 sled 库 `exodus_rag_db`，余弦相似度语义检索。
- **环境变量**：复制 `.env.example` 为 `.env`（勿提交 `.env`）。
- **模型权重**：`allama/models/` 已在 `.gitignore` 中，需本地下载，勿提交 GitHub。

## 🗺️ 路线图（节选）

- [ ] 与 `exodus-core` 二进制深度集成与错误处理增强
- [ ] 标签页 / 书签 / 设置面板完善
- [ ] Web Agent 多步流水线与 DOM 注入
- [ ] IM 网络就绪后启用完整 `im-tests` 测试套件

## 🐛 常见问题

**找不到 Sidecar**

1. 确认二进制在 `src-tauri/binaries/` 且名称与平台匹配  
2. 确认有执行权限  

**AI 连接失败**

1. 查看 Sidecar 是否在 11434 端口监听  
2. 查看应用日志：`~/Library/Application Support/com.exodus.browser/logs/startup.log`（macOS）

**生命周期 / 自动修复（开发）**

```bash
RUST_LOG=info,exodus_startup=info pnpm tauri dev
# 控制台或 Tauri 内：
# await invoke('lifecycle_get_status')
# await invoke('lifecycle_list_presets')
```

## 💻 推荐 IDE

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 🤝 贡献

欢迎提交 PR。请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，提交前运行 `pnpm verify` 或 `pnpm test:quick`。

## 📄 许可证

MIT License — 见 [LICENSE](LICENSE)。

## 🙏 致谢

- Servo / wry / Tauri 社区  
- 开源 AI 生态  
- 所有为数字主权而努力的人  

---

**为极客而生，用 Rust 夺回主权。**

⛵ *走出他们的服务器。*
