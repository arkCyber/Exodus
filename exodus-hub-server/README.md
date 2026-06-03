# Exodus Hub Server

Exodus核心服务 - 管理聊天、群组、微信公众号、视频会议和P2P协调的中心枢纽服务器。

## 功能特性

- **SQLite持久化存储**：所有消息、用户、会话信息存储在SQLite数据库中
- **循环序列号**：群聊消息使用1-9999的循环序列号，帮助断线重连用户检测缺失消息
- **离线消息队列**：离线用户的消息存储在待处理队列中，上线时自动推送
- **在线状态跟踪**：WebSocket连接维护用户在线状态
- **缺失消息检测**：自动计算用户缺失的消息序列号范围
- **消息同步**：支持按序列号范围获取缺失消息
- **群助手功能**：每个群配备云端群助手，支持自动回复和消息存储
- **P2P协调**：协调P2P网络与云端群助手的消息检索优先级
- **消息重发**：自动处理缺失消息的重发请求

## 数据存储结构

### 数据目录
- `im_data/` - 数据存储根目录
- `im_data/im_server.db` - SQLite数据库文件

### 数据库表结构

#### users (用户表)
- `id` - 12位数字用户ID
- `username` - 用户名（唯一）
- `display_name` - 显示名称
- `created_at` - 创建时间
- `last_seen` - 最后在线时间

#### conversations (会话表)
- `id` - 会话ID（UUID）
- `chat_type` - 聊天类型（one_on_one/group）
- `title` - 会话标题
- `created_at` - 创建时间
- `updated_at` - 更新时间
- `message_count` - 消息总数
- `last_sequence` - 最后序列号

#### messages (消息表)
- `id` - 消息ID（UUID）
- `conversation_id` - 会话ID
- `sender_id` - 发送者ID
- `receiver_id` - 接收者ID（可选，群聊为NULL）
- `content` - 消息内容
- `timestamp` - 时间戳
- `sequence` - 序列号（群聊为1-9999循环，私聊为NULL）
- `reply_to` - 回复消息ID（可选）

#### group_members (群组成员表)
- `id` - 成员记录ID
- `conversation_id` - 会话ID
- `user_id` - 用户ID
- `joined_at` - 加入时间
- `role` - 角色（member/admin）

#### pending_messages (待处理消息表)
- `id` - 记录ID
- `message_id` - 消息ID
- `receiver_id` - 接收者ID
- `conversation_id` - 会话ID
- `created_at` - 创建时间

#### user_sequences (用户序列号跟踪表)
- `id` - 记录ID
- `user_id` - 用户ID
- `conversation_id` - 会话ID
- `last_sequence` - 用户最后看到的序列号
- `updated_at` - 更新时间

## 循环序列号机制

### 序列号规则
- 群聊消息序列号范围：1-9999
- 达到9999后回归到1
- 私聊消息不使用序列号（sequence为NULL）

### 缺失消息检测
用户断线重连后，通过比较用户的`last_sequence`和会话的`last_sequence`来检测缺失消息：

```
如果 current_seq > user_seq:
    缺失范围: (user_seq + 1) 到 current_seq
否则（序列号循环）:
    缺失范围: (user_seq + 1) 到 9999
```

### 消息同步流程
1. 客户端调用 `GET /api/im/missing/detect` 检测缺失消息
2. 如果有缺失，调用 `GET /api/im/missing/fetch` 获取缺失的消息
3. 客户端处理完消息后，调用 `POST /api/im/sequence/update` 更新自己的序列号

## HTTP API

### 消息发送
```http
POST /api/im/send
Content-Type: application/json

{
  "conversation_id": "uuid",
  "sender_id": "123456789012",
  "receiver_id": "123456789013",  // 可选，群聊为null
  "content": "Hello",
  "reply_to": "message_uuid"  // 可选
}

响应:
{
  "message_id": "uuid",
  "sequence": 123,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### 获取消息
```http
GET /api/im/messages?conversation_id=uuid&limit=100

响应: [Message对象数组]
```

### 用户注册
```http
POST /api/im/register
Content-Type: application/json

{
  "username": "alice",
  "display_name": "Alice"
}

响应:
{
  "user_id": "123456789012"
}
```

### 获取离线消息
```http
GET /api/im/offline?user_id=123456789012&conversation_id=uuid&limit=100

响应: [Message对象数组]
```

### 获取在线用户
```http
GET /api/im/online

响应: ["123456789012", "123456789013"]
```

### 获取待处理消息
```http
GET /api/im/pending?user_id=123456789012

响应: [PendingMessage对象数组]
```

### 清除待处理消息
```http
POST /api/im/pending/clear?user_id=123456789012

响应: {"cleared": 5}
```

### 更新用户序列号
```http
POST /api/im/sequence/update
Content-Type: application/json

{
  "user_id": "123456789012",
  "conversation_id": "uuid",
  "last_sequence": 9999
}

响应: {"success": true}
```

### 获取用户序列号
```http
GET /api/im/sequence/get?user_id=123456789012&conversation_id=uuid

响应: UserSequence对象或null
```

### 检测缺失消息
```http
GET /api/im/missing/detect?user_id=123456789012&conversation_id=uuid

响应:
{
  "missing": true,
  "start_sequence": 100,
  "end_sequence": 150,
  "missing_count": 51
}
```

### 获取缺失消息
```http
GET /api/im/missing/fetch?conversation_id=uuid&start_sequence=100&end_sequence=150

响应: [Message对象数组]
```

## WebSocket API

### 连接
```
ws://localhost:8080/ws/im/{user_id}
```

### 客户端消息
```json
{
  "type": "heartbeat"
}
```
```json
{
  "type": "typing",
  "conversation_id": "uuid"
}
```
```json
{
  "type": "subscribe",
  "conversation_id": "uuid"
}
```
```json
{
  "type": "unsubscribe",
  "conversation_id": "uuid"
}
```

### 服务器消息
```json
{
  "type": "new_message",
  "conversation_id": "uuid",
  "sender_id": "123456789012",
  "content": "Hello",
  "timestamp": "2024-01-01T00:00:00Z",
  "sequence": 123
}
```
```json
{
  "type": "user_online",
  "user_id": "123456789012",
  "username": "alice"
}
```
```json
{
  "type": "user_offline",
  "user_id": "123456789012"
}
```
```json
{
  "type": "typing",
  "conversation_id": "uuid",
  "user_id": "123456789012"
}
```

## 运行服务器

### 编译
```bash
cd storage-forward-server
cargo build --release
```

### 运行
```bash
cargo run --release
```

服务器将在 `http://0.0.0.0:8080` 启动。

### 开发模式
```bash
cargo run
```

## 使用示例

### 1. 注册用户
```bash
curl -X POST http://localhost:8080/api/im/register \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "display_name": "Alice"}'
```

### 2. 创建群聊会话
```rust
// 需要通过数据库直接创建或扩展API
```

### 3. 发送消息
```bash
curl -X POST http://localhost:8080/api/im/send \
  -H "Content-Type: application/json" \
  -d '{
    "conversation_id": "conv-uuid",
    "sender_id": "123456789012",
    "content": "Hello everyone!"
  }'
```

### 4. 检测缺失消息
```bash
curl "http://localhost:8080/api/im/missing/detect?user_id=123456789012&conversation_id=conv-uuid"
```

### 5. 获取缺失消息
```bash
curl "http://localhost:8080/api/im/missing/fetch?conversation_id=conv-uuid&start_sequence=100&end_sequence=150"
```

### 6. 更新序列号
```bash
curl -X POST http://localhost:8080/api/im/sequence/update \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "123456789012",
    "conversation_id": "conv-uuid",
    "last_sequence": 150
  }'
```

## Rust客户端集成

本项目是Rust项目，客户端应使用Rust实现。以下是在Tauri应用中集成IM客户端的示例：

### 添加依赖
在 `src-tauri/Cargo.toml` 中添加：
```toml
[dependencies]
exodus-hub-server = { path = "../exodus-hub-server" }
reqwest = { version = "0.11", features = ["json"] }
tokio-tungstenite = "0.20"
serde_json = "1.0"
```

### Rust客户端实现
```rust
use reqwest::Client;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;
use std::collections::HashMap;

pub struct IMClient {
    user_id: String,
    http_client: Client,
    sender_sequences: HashMap<String, u32>,
}

impl IMClient {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            http_client: Client::new(),
            sender_sequences: HashMap::new(),
        }
    }
    
    pub async fn register(&self, username: &str, display_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.http_client
            .post("http://localhost:8080/api/im/register")
            .json(&json!({
                "username": username,
                "display_name": display_name
            }))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        Ok(data["user_id"].as_str().unwrap().to_string())
    }
    
    pub async fn send_message(&self, conversation_id: &str, receiver_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.http_client
            .post("http://localhost:8080/api/im/send")
            .json(&json!({
                "conversation_id": conversation_id,
                "sender_id": self.user_id,
                "receiver_id": receiver_id,
                "content": content
            }))
            .send()
            .await?;
        Ok(())
    }
    
    pub async fn check_user_online(&self, user_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let response = self.http_client
            .get(&format!("http://localhost:8080/api/im/check-online?user_id={}", user_id))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        Ok(data["online"].as_bool().unwrap_or(false))
    }
    
    pub async fn detect_missing_messages(&self, sender_id: &str) -> Result<Option<(u32, u32)>, Box<dyn std::error::Error>> {
        let response = self.http_client
            .get(&format!("http://localhost:8080/api/im/missing/detect?user_id={}&sender_id={}", self.user_id, sender_id))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        if data["missing"].as_bool().unwrap_or(false) {
            Ok(Some((
                data["start_sequence"].as_u64().unwrap() as u32,
                data["end_sequence"].as_u64().unwrap() as u32
            )))
        } else {
            Ok(None)
        }
    }
    
    pub async fn request_resend(&self, sender_id: &str, start_seq: u32, end_seq: u32) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let response = self.http_client
            .post("http://localhost:8080/api/im/resend/request")
            .json(&json!({
                "sender_id": sender_id,
                "start_sequence": start_seq,
                "end_sequence": end_seq
            }))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        Ok(data["messages"].as_array().unwrap().clone())
    }
    
    pub async fn connect_websocket(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("ws://localhost:8080/ws/im/{}", self.user_id);
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        // 发送心跳
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                write.send(Message::Text(json!({"type": "heartbeat"}).to_string())).await.ok();
            }
        });
        
        // 接收消息
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    let data: serde_json::Value = serde_json::from_str(&text)?;
                    match data["type"].as_str() {
                        Some("new_message") => {
                            // 处理新消息
                            let sender_id = data["sender_id"].as_str().unwrap();
                            let sequence = data["sequence"].as_u64().unwrap() as u32;
                            self.sender_sequences.insert(sender_id.to_string(), sequence);
                            
                            // 自动发送收据
                            let receipt = json!({
                                "type": "message_receipt",
                                "message_id": data["id"],
                                "sequence": sequence
                            });
                            write.send(Message::Text(receipt.to_string())).await.ok();
                        }
                        Some("message_receipt") => {
                            // 处理收据
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}
```

### 定时检查实现
```rust
impl IMClient {
    pub async fn start_periodic_check(&self) {
        let client = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30 * 60)).await; // 30分钟
                
                // 检查待处理消息
                let pending = client.get_pending_messages().await.ok();
                
                // 检查每个发送者的缺失消息
                for (sender_id, _) in &client.sender_sequences {
                    if let Ok(Some((start, end))) = client.detect_missing_messages(sender_id).await {
                        if let Ok(messages) = client.request_resend(sender_id, start, end).await {
                            // 处理重发的消息
                        }
                    }
                }
            }
        });
    }
}
```

## 架构说明

### 断线重连消息恢复流程

1. **用户离线**：消息存入`pending_messages`表
2. **用户上线**：
   - WebSocket连接建立
   - 服务器自动推送`pending_messages`中的消息
   - 清空`pending_messages`队列
3. **检测缺失**：
   - 客户端调用`/api/im/missing/detect`
   - 比较用户的`last_sequence`和会话的`last_sequence`
   - 计算缺失的序列号范围
4. **同步消息**：
   - 客户端调用`/api/im/missing/fetch`获取缺失消息
   - 从其他用户或服务器数据库获取
   - 处理完成后更新`last_sequence`

### 循环序列号处理

当序列号从9999回归到1时，客户端需要：
1. 检测到`current_seq < user_seq`表示序列号已循环
2. 获取从`user_seq + 1`到9999的消息
3. 获取从1到`current_seq`的消息
4. 合并两个范围的消息

## 技术栈

- Rust
- SQLite (rusqlite)
- Axum (Web框架)
- Tokio (异步运行时)
- WebSocket (实时通信)
- Chrono (时间处理)
- Serde (序列化)

## 依赖

```toml
anyhow = "1.0"
axum = { version = "0.7", features = ["ws"] }
chrono = { version = "0.4", features = ["serde"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["v4", "serde"] }
rand = "0.8"
futures-util = "0.3"
sha2 = "0.10"
```
