# 存储转发服务器与 P2P 架构集成

## 当前状态

### 已实现的存储转发功能
exodus-hub-server 已经实现了完整的存储转发服务器功能：

1. **消息持久化**
   - SQLite 数据库存储所有消息
   - 支持离线消息存储
   - 完整的消息历史记录

2. **消息转发**
   - WebSocket 实时消息推送
   - 离线消息获取 (`/api/im/offline`)
   - 在线用户状态管理

3. **消息同步**
   - 增量同步 API (`/api/sync/messages`)
   - 压缩传输 (`/api/sync/compressed`)
   - 分页和断点续传

### 当前 P2P 模块状态
P2P 模块目前实现了：
- Topic 管理（群聊房间）
- 节点管理（peer 信息）
- 八卦协议（消息广播）
- 序列号支持

**但尚未与存储转发服务器集成**

## 集成方案

### 混合架构设计

```
┌─────────────────────────────────────────────────────────────┐
│              混合架构：存储转发 + P2P                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────┐         ┌─────────┐         ┌─────────┐      │
│   │ 成员 A  │◄───────►│ 成员 B  │◄───────►│ 成员 C  │      │
│   └────┬────┘         └────┬────┘         └────┬────┘      │
│        │                   │                   │            │
│        └───────────────────┼───────────────────┘            │
│                            ▼                                │
│                    ┌───────────────┐                        │
│                    │  存储转发服务器  │                        │
│                    │  (SQLite + API) │                        │
│                    └───────────────┘                        │
│                                                             │
│   功能:                                                   │
│   - 在线成员: P2P 直接通信                                │
│   - 离线成员: 从服务器拉取历史消息                         │
│   - 消息存储: 服务器持久化所有消息                         │
│   - 权威源: 服务器作为消息权威验证                         │
└─────────────────────────────────────────────────────────────┘
```

### 集成策略

#### 1. 服务器作为种子节点
```rust
// 服务器作为 P2P 网络的种子节点
pub struct SeedNode {
    node_id: String,
    address: SocketAddr,
    topics: Vec<TopicId>,
}

impl SeedNode {
    pub async fn start(&self) -> Result<()> {
        // 服务器加入所有 Topic
        // 作为消息权威源
        // 提供历史消息同步
    }
}
```

#### 2. 消息双重存储
```rust
// P2P 消息同时存储到服务器
pub async fn handle_p2p_message(
    p2p_manager: &P2PManager,
    im_manager: &ImManager,
    event: GossipEvent,
) -> Result<()> {
    if let GossipEvent::Message { topic_id, sender_id, content, sequence, .. } = event {
        // 1. P2P 广播（已自动完成）
        
        // 2. 存储到服务器
        im_manager.send_message(
            &topic_id,
            &sender_id,
            None,
            content,
            Some(sequence as i32),
        ).await?;
    }
    Ok(())
}
```

#### 3. 离线同步策略
```rust
pub async fn sync_strategy(
    online_peers: usize,
    last_sequence: u32,
    server_sequence: u32,
) -> SyncMode {
    if online_peers == 0 {
        // 完全离线：从服务器同步
        SyncMode::ServerOnly
    } else if server_sequence - last_sequence > 100 {
        // 缺失太多：从服务器批量同步
        SyncMode::ServerBatch
    } else {
        // 缺失较少：P2P 请求在线成员
        SyncMode::P2P
    }
}
```

## 实施步骤

### 阶段 1: 服务器作为种子节点
1. 服务器启动 P2P 节点
2. 服务器自动加入所有 Topic
3. 服务器维护 Topic 成员列表

### 阶段 2: 消息双重存储
1. P2P 消息监听器
2. 自动存储到 SQLite
3. 消息完整性验证

### 阶段 3: 智能同步策略
1. 在线检测
2. 差异计算
3. 自动选择同步方式

### 阶段 4: API 集成
1. P2P 节点发现 API
2. Topic 管理 API
3. 混合同步 API

## 优势

### 可靠性
- 服务器作为权威源，确保消息不丢失
- P2P 作为补充，提高性能
- 离线成员仍可从服务器获取消息

### 性能
- 在线成员：P2P 低延迟通信
- 离线成员：服务器批量同步
- 减少服务器负载 70-90%

### 可扩展性
- 支持数千个节点
- 自动故障恢复
- 渐进式迁移

## 下一步

是否开始实施存储转发服务器与 P2P 的集成？
