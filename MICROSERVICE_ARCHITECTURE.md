# Exodus 浏览器微服务架构设计文档

## 一、架构概述

### 1.1 设计目标

利用 Rust 的轻量化和极高的并发性能，在系统后台以低廉的内存开销（每个服务仅几 MB 到十几 MB）运行多个微服务，各模块之间通过本地高性能 IPC（Unix Domain Socket / 内存映射 IPC）进行通信。

### 1.2 架构图

```
┌──────────────────────────────┐
│  Exodus 前端 WebView UI 外壳 │
└──────────────┬───────────────┘
               │ (Tauri API Gateway / UDS / JSON-RPC)
               ▼
┌───────────────────────────────────────────────────────────────────┐
│               Exodus Rust 后端本地微服务总线 (IPC Bus)               │
└─┬───────────────┬───────────────┬───────────────┬───────────────┬─┘
  │               │               │               │               │
  ▼               ▼               ▼               ▼               ▼
┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐┌──────────────┐
│  AI 推理服务 ││  Hermes 智能 ││  本地向量库  ││  Web3 区块链 ││自动化动作执行│
│  (Micro-LLM) ││  (Agent Core)││  (Local RAG) ││  (Crypto RPC)││ (Browser-Act)│
│ 跑Gemma4/Qwen││ 负责策略规划 ││嵌入sled/qdrant││  钱包/合约审计││  DOM控制/填表 │
└──────────────┘└──────────────┘└──────────────┘└──────────────┘└──────────────┘
```

## 二、核心微服务模块

### 2.1 exodus-rag-service (独立 RAG 微服务)

**功能描述：**
- 独立常驻后台，使用纯 Rust 编写
- 结合 sled 键值库或 faiss-rs 进行向量存储
- 使用 notify 库实时监听本地文件系统（~/Documents/ 或 Obsidian 笔记目录）
- 文件变动时自动增量向量化

**创新点：**
- 不仅为浏览器服务，可作为全系统的本地知识库
- 浏览器 AI 可随时调用此微服务实现本地知识库问答
- 支持多种文档格式（Markdown, PDF, TXT, Obsidian vaults）

**技术栈：**
- 存储：sled (键值) + faiss-rs (向量索引)
- 文件监听：notify-rs
- 嵌入：candle-core / burn (本地推理)
- 通信：Unix Domain Socket + JSON-RPC 2.0

**API 接口示例：**
```json
{
  "jsonrpc": "2.0",
  "method": "rag.upsert",
  "params": {
    "documents": [
      {"id": "doc1", "content": "...", "metadata": {"source": "notes"}}
    ]
  },
  "id": 1
}

{
  "jsonrpc": "2.0",
  "method": "rag.search",
  "params": {
    "query": "如何配置 Rust 开发环境",
    "top_k": 5
  },
  "id": 2
}
```

### 2.2 exodus-crypto-service (Web3 区块链微服务)

**功能描述：**
- 集成 Rust Web3 生态库（ethers-rs 或 alloy）
- 作为轻量级本地 RPC 节点
- 后台高频模拟交易（EVM Simulation）
- AI 配合计算交易安全性（蜜罐检测、夹子攻击检测）

**创新点：**
- 浏览器打开 DApp 时提供钱包签名接口
- 交易前安全审计，防止资金损失
- 支持多链（EVM, Solana, Bitcoin）

**技术栈：**
- Web3：alloy-rs / ethers-rs
- EVM 模拟：revm (Rust EVM)
- 密码学：k256 / secp256k1
- 通信：Unix Domain Socket + JSON-RPC 2.0

**API 接口示例：**
```json
{
  "jsonrpc": "2.0",
  "method": "crypto.simulate_transaction",
  "params": {
    "to": "0x...",
    "value": "1000000000000000000",
    "data": "0x...",
    "chain_id": 1
  },
  "id": 1
}

{
  "jsonrpc": "2.0",
  "method": "crypto.audit_transaction",
  "params": {
    "tx": {...}
  },
  "id": 2
}
```

### 2.3 exodus-os-service (跨平台自动化微服务)

**功能描述：**
- 拥有操作系统最高权限的原生 Rust 模块
- 使用 enigo 库模拟全局键盘鼠标
- 调用系统 API 进行窗口管理

**创新点：**
- 智能体触手冲出浏览器
- 可控制窗口管理器、终端执行命令
- 自动打开系统应用

**技术栈：**
- 输入模拟：enigo / input-emulator
- 窗口管理：windows-rs (Windows), x11-dl (Linux), cocoa (macOS)
- 进程管理：sysinfo
- 通信：Unix Domain Socket + JSON-RPC 2.0

**API 接口示例：**
```json
{
  "jsonrpc": "2.0",
  "method": "os.simulate_keypress",
  "params": {
    "key": "Ctrl+C"
  },
  "id": 1
}

{
  "jsonrpc": "2.0",
  "method": "os.execute_command",
  "params": {
    "command": "cargo build",
    "working_dir": "/path/to/project"
  },
  "id": 2
}
```

### 2.4 exodus-depin-node (分布式 P2P 算力共享微服务)

**功能描述：**
- 基于 Rust 的 libp2p 网络库编写
- 接入全球 DePIN 网络接单
- 与浏览器运行完全解耦

**创新点：**
- 用户关闭浏览器后仍可在后台运行
- 不影响网页浏览体验
- 算力出租收益

**技术栈：**
- P2P 网络：libp2p-rs
- 共识：可选（根据 DePIN 协议）
- 资源监控：sysinfo
- 通信：Unix Domain Socket + JSON-RPC 2.0

**API 接口示例：**
```json
{
  "jsonrpc": "2.0",
  "method": "depin.start_mining",
  "params": {
    "gpu": true,
    "cpu_cores": 4
  },
  "id": 1
}

{
  "jsonrpc": "2.0",
  "method": "depin.get_status",
  "params": {},
  "id": 2
}
```

## 三、IPC 通信协议设计

### 3.1 通信方案

**主方案：Unix Domain Socket (UDS) + JSON-RPC 2.0**

**优势：**
- 低延迟（本地通信，无需网络栈）
- 高性能（避免序列化开销，可选 MessagePack）
- 安全（文件系统权限控制）
- 跨语言友好（JSON-RPC 标准协议）

**备选方案：**
- 共享内存 + 信号量（极高性能，复杂度高）
- gRPC over UDS（类型安全，但需要 protobuf）

### 3.2 消息格式

```rust
// src-tauri/src/microservice/protocol.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
```

### 3.3 服务注册与发现

```rust
// 服务注册表
pub struct ServiceRegistry {
    services: HashMap<String, ServiceInfo>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub socket_path: String,
    pub status: ServiceStatus,
    pub pid: u32,
}

pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}
```

## 四、微服务总线设计

### 4.1 总线架构

```rust
// src-tauri/src/microservice/bus.rs

pub struct MicroserviceBus {
    registry: Arc<Mutex<ServiceRegistry>>,
    clients: HashMap<String, UnixStream>,
}

impl MicroserviceBus {
    // 注册服务
    pub async fn register_service(&mut self, name: String, socket_path: String) -> Result<(), Error> {
        // ...
    }
    
    // 调用服务
    pub async fn call_service(&mut self, service: String, request: JsonRpcRequest) -> Result<JsonRpcResponse, Error> {
        // ...
    }
    
    // 广播消息
    pub async fn broadcast(&mut self, request: JsonRpcRequest) -> Vec<Result<JsonRpcResponse, Error>> {
        // ...
    }
    
    // 健康检查
    pub async fn health_check(&self, service: String) -> Result<ServiceStatus, Error> {
        // ...
    }
}
```

### 4.2 故障隔离与恢复

```rust
pub struct ServiceSupervisor {
    services: HashMap<String, ServiceHandle>,
    max_retries: u32,
    retry_delay: Duration,
}

impl ServiceSupervisor {
    // 监控服务状态
    pub async fn supervise(&mut self) {
        loop {
            for (name, handle) in &mut self.services {
                if let Some(status) = handle.check_status().await {
                    if status == ServiceStatus::Error(String::new()) {
                        self.restart_service(name).await;
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    
    // 重启服务
    async fn restart_service(&mut self, name: &str) {
        // ...
    }
}
```

## 五、实现计划

### Phase 1: 基础设施（优先级：高）
- [ ] 实现 IPC 通信协议（UDS + JSON-RPC）
- [ ] 实现微服务总线
- [ ] 实现服务注册与发现
- [ ] 实现服务监控与故障恢复

### Phase 2: RAG 微服务（优先级：高）
- [ ] 实现 exodus-rag-service
- [ ] 集成文件监听
- [ ] 实现增量向量化
- [ ] 提供搜索 API

### Phase 3: Crypto 微服务（优先级：中）
- [ ] 实现 exodus-crypto-service
- [ ] 集成 EVM 模拟
- [ ] 实现交易安全审计

### Phase 4: OS 自动化微服务（优先级：中）
- [ ] 实现 exodus-os-service
- [ ] 实现输入模拟
- [ ] 实现窗口管理

### Phase 5: DePIN 微服务（优先级：低）
- [ ] 实现 exodus-depin-node
- [ ] 集成 libp2p
- [ ] 实现算力管理

## 六、工程优势

### 6.1 故障隔离
- 每个微服务独立进程，一个崩溃不影响其他服务
- Tauri 总线自动感知并重启故障服务
- 用户浏览器窗口和标签页不受影响

### 6.2 多语言生态融合
- 核心总线、RAG、DOM 控制使用 Rust 追求性能
- Hermes 智能体可用 Python 作为微服务节点
- 通过标准协议（JSON-RPC/gRPC）实现语言无关

### 6.3 热插拔与扩展
- 社区极客可用 Rust/Go 编写微服务
- 符合通信协议规范即可挂载到总线
- 插件目录动态加载

## 七、性能指标

### 7.1 内存开销
- 每个微服务：5-20 MB
- 总线进程：2-5 MB
- 总计：< 100 MB（5个微服务）

### 7.2 延迟
- 本地 IPC 延迟：< 1ms
- JSON-RPC 调用：1-5ms
- 服务重启：< 500ms

### 7.3 吞吐量
- 单服务 QPS：> 1000
- 总线并发：> 10000

## 八、安全考虑

### 8.1 权限控制
- Unix Domain Socket 文件权限（0600）
- 服务间通信加密（可选 TLS）
- API 调用鉴权

### 8.2 资源限制
- 每个服务 CPU/内存限制（cgroups）
- 磁盘 I/O 限制
- 网络带宽限制

## 九、部署方案

### 9.1 开发环境
```bash
# 启动总线
cargo run --bin exodus-bus

# 启动各个微服务
cargo run --bin exodus-rag-service
cargo run --bin exodus-crypto-service
cargo run --bin exodus-os-service
cargo run --bin exodus-depin-node
```

### 9.2 生产环境
```bash
# 使用 systemd 管理
systemctl start exodus-bus
systemctl start exodus-rag
systemctl start exodus-crypto
systemctl start exodus-os
systemctl start exodus-depin
```

### 9.3 打包方案
- 每个微服务独立可执行文件
- 统一配置管理
- 日志集中收集
