# Exodus Browser - 快速入门指南

## 🚀 快速开始

### 前置要求

1. **Rust** (最新稳定版)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (v18+)
   ```bash
   # macOS
   brew install node
   
   # 或使用 nvm
   nvm install 18
   ```

3. **Ollama** (可选 - 用于 AI 功能)
   ```bash
   # macOS
   brew install ollama
   
   # 启动 Ollama 服务
   ollama serve
   
   # 下载模型
   ollama pull llama2
   ollama pull nomic-embed-text
   ```

### 安装依赖

```bash
# 克隆仓库
git clone https://github.com/yourusername/Exodus.git
cd Exodus

# 安装前端依赖
npm install

# 构建前端
npm run build

# 运行开发模式
npm run tauri dev
```

### 构建生产版本

```bash
# 构建应用
npm run tauri build

# 构建产物位于:
# macOS: src-tauri/target/release/bundle/dmg/
# Linux: src-tauri/target/release/bundle/appimage/
# Windows: src-tauri/target/release/bundle/msi/
```

---

## 🎯 核心功能使用

### 1. 标签页管理

#### 创建标签页组
1. 右键点击标签页
2. 选择 "新建标签页组"
3. 自定义组名和颜色

#### 固定标签页
- 右键点击标签页 → "固定标签页"
- 固定的标签页不会被意外关闭

#### 标签页休眠 (内存优化)
```typescript
// 前端调用示例
import { invoke } from '@tauri-apps/api/core';

// 注册标签页
await invoke('tab_sleep_register', {
  tabId: 'tab-123',
  label: 'webview-label',
  url: 'https://example.com',
  title: 'Example Page',
  isPinned: false
});

// 获取休眠统计
const stats = await invoke('tab_sleep_get_stats');
console.log(`节省内存: ${stats.saved_memory_mb} MB`);

// 更新配置
await invoke('tab_sleep_update_config', {
  config: {
    enabled: true,
    inactive_threshold_secs: 300,  // 5 分钟
    max_active_tabs: 10,
    exclude_media: true,
    exclude_pinned: true
  }
});
```

### 2. AI 功能

#### 语义搜索
```typescript
// 在地址栏输入
/ask 如何使用 Rust 编写异步代码

// 或通过 API
const results = await invoke('semantic_search', {
  query: '如何使用 Rust 编写异步代码'
});
```

#### AI 聊天（推荐：Allama HTTP 11435）
```typescript
import { streamAllamaChatCompletions } from '$lib/allamaClient';

await streamAllamaChatCompletions(
  [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Explain microservices briefly.' },
  ],
  { model: 'exodus-default', port: 11435 },
  {
    onChunk: (c) => console.log(c),
    onDone: () => console.log('done'),
    onError: (e) => console.error(e),
  },
);
```

可选：Tauri 事件流（扩展或脚本）
```typescript
await invoke('ai_chat_stream', {
  prompt: '解释一下什么是微服务架构',
  // 或多轮：messages: [{ role: 'user', content: '...' }],
});
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('exodus-ai-chunk', (event) => {
  console.log('AI:', event.payload.content);
});
```

#### 页面索引 (RAG)
```typescript
// 索引当前页面
await invoke('capture_page', {
  url: currentUrl,
  title: pageTitle,
  content: pageContent
});

// 搜索已索引页面
const pages = await invoke('get_history');
```

### 3. 隐私与安全

#### 启用 HTTPS-Only 模式
```typescript
await invoke('set_privacy_settings', {
  httpsOnly: true,
  privateMode: false,
  blockPopups: true,
  sessionRestore: true
});
```

#### 查看隐私统计
```typescript
const stats = await invoke('get_privacy_stats');
console.log(`拦截追踪器: ${stats.trackers_blocked}`);
console.log(`拦截 Cookie: ${stats.cookies_blocked}`);
```

#### 安全浏览检查
```typescript
const result = await invoke('check_url_safe', {
  url: 'https://suspicious-site.com'
});

if (!result.safe) {
  console.log('警告:', result.reason);
}
```

### 4. 密码管理

#### 保存密码
```typescript
await invoke('save_password', {
  url: 'https://example.com',
  username: 'user@example.com',
  password: 'secure-password'
});
```

#### 自动填充
```typescript
const entry = await invoke('get_password_for_page', {
  url: currentUrl
});

if (entry) {
  // 应用自动填充
  await invoke('password_build_fill_script', {
    label: webviewLabel,
    entry: entry
  });
}
```

#### 检查密码强度
```typescript
const strength = await invoke('check_password_strength', {
  password: 'MyP@ssw0rd123'
});

console.log(`强度: ${strength.score}/4`);
console.log(`建议: ${strength.feedback}`);
```

### 5. 微服务架构

#### 服务发现
```typescript
// 注册服务 (后端)
use crate::microservice::ServiceDiscovery;

let discovery = ServiceDiscovery::new();
let endpoint = ServiceEndpoint::new("api-service", "localhost", 8080)
    .with_metadata("version", "1.0.0")
    .with_health_check("/health");

discovery.register(endpoint).await?;

// 发现服务
let endpoints = discovery.discover("api-service").await?;
for endpoint in endpoints {
    println!("服务地址: {}", endpoint.get_url());
}
```

#### 熔断器
```rust
use crate::microservice::CircuitBreaker;

let breaker = CircuitBreaker::new();

if breaker.allow_request().await {
    match call_external_service().await {
        Ok(response) => {
            breaker.record_success().await;
            Ok(response)
        }
        Err(e) => {
            breaker.record_failure().await;
            Err(e)
        }
    }
} else {
    Err("服务暂时不可用".into())
}
```

#### 分布式追踪
```rust
use crate::microservice::{TraceContext, Span, SpanStatus};

let context = TraceContext::new();
let mut span = Span::new(&context, "user-service", "create_user");

span.add_tag("user_id", "123");
span.add_log("开始创建用户", LogLevel::Info);

// 执行操作...

span.finish(SpanStatus::Ok);
collector.record_span(span).await;
```

#### 配置管理
```rust
use crate::microservice::{ConfigManager, ConfigEntry};

let manager = ConfigManager::new();

// 设置配置
let entry = ConfigEntry::new("database.url", "postgres://localhost")
    .for_service("api")
    .with_environment("production");

manager.set(entry).await?;

// 订阅配置变更
let mut rx = manager.subscribe().await;
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        println!("配置变更: {} -> {}", event.key, event.new_value);
        // 重新加载配置...
    }
});
```

#### 指标收集
```rust
use crate::microservice::MetricsCollector;

let collector = MetricsCollector::new();

// 计数器
let mut labels = HashMap::new();
labels.insert("method".to_string(), "GET".to_string());
collector.counter("http_requests_total", 1.0, labels).await;

// 直方图 (延迟)
collector.histogram("http_request_duration", 0.123).await;

// 导出 Prometheus 格式
let metrics = collector.export_prometheus().await;
```

---

## 🔧 配置

### 应用配置文件

配置文件位于:
- **macOS**: `~/Library/Application Support/com.exodus.browser/config.json`
- **Linux**: `~/.config/exodus-browser/config.json`
- **Windows**: `%APPDATA%\exodus-browser\config.json`

```json
{
  "ai_port": 11434,
  "ai_model": "llama2",
  "embedding_model": "nomic-embed-text",
  "homepage_url": "https://duckduckgo.com",
  "search_engine_url": "https://duckduckgo.com/?q={query}",
  "spawn_sidecar": true,
  "status_clear_ms": 3000
}
```

### 隐私设置

```json
{
  "https_only": true,
  "private_mode": false,
  "block_popups": true,
  "session_restore": true
}
```

### 标签页休眠配置

```json
{
  "tab_sleep": {
    "enabled": true,
    "inactive_threshold_secs": 300,
    "max_active_tabs": 10,
    "exclude_media": true,
    "exclude_pinned": true
  }
}
```

---

## 🐛 调试

### 启用开发者工具

```bash
# 开发模式自动启用
npm run tauri dev

# 在应用中按 Cmd+Option+I (macOS) 或 Ctrl+Shift+I (Windows/Linux)
```

### 查看日志

```bash
# Rust 后端日志
RUST_LOG=debug npm run tauri dev

# 前端控制台
# 在浏览器 DevTools 中查看
```

### 常见问题

#### 1. Ollama 连接失败
```bash
# 检查 Ollama 是否运行
curl http://localhost:11434/api/tags

# 重启 Ollama
ollama serve
```

#### 2. 标签页无法加载
```bash
# 清除缓存
rm -rf ~/Library/Application\ Support/com.exodus.browser/cache

# 重启应用
```

#### 3. 编译错误
```bash
# 清理构建
cargo clean
npm run clean

# 重新安装依赖
npm install
cargo build
```

---

## 📚 进阶主题

### 自定义扩展开发

参见 `docs/EXTENSION_DEVELOPMENT.md`

### 微服务集成

参见 `docs/MICROSERVICE_INTEGRATION.md`

### 性能优化

参见 `docs/PERFORMANCE_TUNING.md`

---

## 🤝 贡献

欢迎贡献！请查看 `CONTRIBUTING.md` 了解详情。

### 开发流程

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

---

## 📄 许可证

MIT License - 详见 `LICENSE` 文件

---

## 🆘 获取帮助

- **文档**: https://docs.exodus-browser.com
- **问题追踪**: https://github.com/yourusername/Exodus/issues
- **讨论**: https://github.com/yourusername/Exodus/discussions
- **Discord**: https://discord.gg/exodus-browser

---

**祝您使用愉快！** 🎉
