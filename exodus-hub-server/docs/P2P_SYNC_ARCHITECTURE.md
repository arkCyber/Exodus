# P2P 群聊同步架构设计

## 问题分析

### 当前架构的问题
- **服务器负载过高**: 所有成员向服务器请求历史消息
- **网络瓶颈**: 服务器带宽成为传输瓶颈
- **延迟问题**: 跨地域传输延迟高
- **成本问题**: 海外服务器流量费用高

### P2P 同步的优势
- **负载分散**: 成员之间直接传输，减轻服务器压力
- **降低延迟**: 成员之间地理位置可能更近
- **提高速度**: 多个成员并行传输
- **增强容错**: 即使服务器暂时不可用，成员之间也能同步

## 推荐方案：混合架构

### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      群聊网络拓扑                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────┐         ┌─────────┐         ┌─────────┐      │
│   │ 成员 A  │◄───────►│ 成员 B  │◄───────►│ 成员 C  │      │
│   │  (在线) │         │  (在线) │         │  (在线) │      │
│   └────┬────┘         └────┬────┘         └────┬────┘      │
│        │                   │                   │            │
│        └───────────────────┼───────────────────┘            │
│                            ▼                                │
│                    ┌───────────────┐                        │
│                    │  群管理员服务器  │                        │
│                    │  (权威源/协调)  │                        │
│                    └───────────────┘                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 核心组件

#### 1. 服务器角色（协调者）
- **元数据管理**: 维护群成员列表和在线状态
- **消息索引**: 维护消息的 Merkle Tree 根哈希
- **完整性验证**: 验证消息完整性
- **种子节点**: 为新加入成员提供初始连接

#### 2. 客户端角色（P2P 节点）
- **节点发现**: 通过 DHT 或服务器发现其他成员
- **差异检测**: 使用 Merkle Tree 检测缺失消息
- **数据传输**: P2P 传输消息数据
- **缓存管理**: 缓存已接收的消息供其他成员下载

## 技术实现

### 1. 节点发现

#### 方案 A: 服务器辅助发现（推荐）
```rust
// 客户端向服务器请求群成员列表
GET /api/groups/{group_id}/members
Response: {
  "members": [
    { "id": "user1", "ip": "192.168.1.100", "port": 9000, "last_seen": "..." },
    { "id": "user2", "ip": "192.168.1.101", "port": 9000, "last_seen": "..." }
  ]
}

// 客户端建立 P2P 连接
connect_to_peer(ip, port)
```

#### 方案 B: DHT 发现
```rust
// 使用 Kademlia DHT
use libp2p::{kad, identity};

let local_key = identity::Keypair::generate_ed25519();
let local_peer_id = PeerId::from(local_key.public());
```

### 2. 消息差异检测

#### Merkle Tree 方案
```rust
pub struct MessageMerkleTree {
    root: Hash,
    leaves: Vec<Hash>,
}

impl MessageMerkleTree {
    pub fn from_messages(messages: &[Message]) -> Self {
        // 为每条消息计算哈希
        let leaves: Vec<Hash> = messages.iter()
            .map(|m| hash_message(m))
            .collect();
        
        // 构建 Merkle Tree
        let root = compute_merkle_root(&leaves);
        
        Self { root, leaves }
    }
    
    pub fn detect_missing(&self, other_root: Hash) -> Vec<usize> {
        // 比较根哈希，返回缺失的消息索引
        if self.root == other_root {
            return vec![];
        }
        
        // 递归比较子树，找出差异
        self.compare_subtrees(&self.leaves, other_root)
    }
}
```

#### 同步流程
```
1. 成员 A 向成员 B 发送同步请求
   Request: { "conversation_id": "xxx", "merkle_root": "hash_a" }

2. 成员 B 比较本地 Merkle Tree
   Response: { "missing_indices": [10, 15, 20], "merkle_root": "hash_b" }

3. 成员 A 请求缺失的消息
   Request: { "indices": [10, 15, 20] }

4. 成员 B 返回消息数据
   Response: { "messages": [...] }
```

### 3. P2P 传输协议

#### 协议定义
```rust
#[derive(Debug, Serialize, Deserialize)]
enum P2PMessage {
    // 节点发现
    Discover { conversation_id: String },
    DiscoverResponse { peers: Vec<PeerInfo> },
    
    // 差异检测
    SyncRequest { conversation_id: String, merkle_root: Hash },
    SyncResponse { missing_indices: Vec<usize>, merkle_root: Hash },
    
    // 数据传输
    FetchMessages { indices: Vec<usize> },
    FetchResponse { messages: Vec<Message> },
    
    // 心跳
    Ping { timestamp: u64 },
    Pong { timestamp: u64 },
}
```

#### 传输优化
```rust
pub struct P2PTransport {
    compression: bool,
    chunk_size: usize,
}

impl P2PTransport {
    pub fn send_messages(&self, messages: &[Message]) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(messages)?;
        let compressed = if self.compression {
            zstd::encode_all(&serialized[..], 3)?
        } else {
            serialized
        };
        
        // 分块传输
        self.chunk_data(&compressed)
    }
}
```

### 4. 混合同步策略

#### 智能同步选择
```rust
pub enum SyncStrategy {
    ServerOnly,      // 仅从服务器同步
    P2POnly,         // 仅 P2P 同步
    Hybrid,          // 混合模式
}

pub fn select_sync_strategy(
    online_peers: usize,
    missing_messages: usize,
    server_latency: u64,
) -> SyncStrategy {
    if online_peers == 0 {
        SyncStrategy::ServerOnly
    } else if missing_messages < 100 {
        SyncStrategy::P2POnly
    } else if server_latency < 100 {
        SyncStrategy::Hybrid
    } else {
        SyncStrategy::P2POnly
    }
}
```

#### 混合同步流程
```
1. 客户端上线，向服务器请求成员列表
2. 客户端与多个在线成员建立 P2P 连接
3. 客户端计算本地 Merkle Tree
4. 客户端向服务器请求权威 Merkle Tree
5. 客户端与多个成员比较差异
6. 客户端从最近的成员下载缺失消息
7. 客户端验证消息完整性
8. 客户端向服务器报告同步完成
```

## 性能优化

### 1. 并行下载
```rust
pub async fn parallel_fetch(
    peers: Vec<Peer>,
    missing_indices: Vec<usize>,
) -> Result<Vec<Message>> {
    let chunks = split_indices(missing_indices, peers.len());
    
    let tasks: Vec<_> = chunks.into_iter()
        .zip(peers.into_iter())
        .map(|(indices, peer)| {
            tokio::spawn(async move {
                fetch_from_peer(peer, indices).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    // 合并结果
}
```

### 2. 智能缓存
```rust
pub struct MessageCache {
    lru: LruCache<MessageId, Message>,
    ttl: Duration,
}

impl MessageCache {
    pub fn get(&mut self, id: &MessageId) -> Option<Message> {
        self.lru.get(id).cloned()
    }
    
    pub fn insert(&mut self, message: Message) {
        self.lru.put(message.id.clone(), message);
    }
}
```

### 3. 带宽限制
```rust
pub struct BandwidthLimiter {
    max_bytes_per_sec: u64,
    current_bytes: u64,
}

impl BandwidthLimiter {
    pub async fn acquire(&mut self, bytes: u64) -> Result<()> {
        while self.current_bytes + bytes > self.max_bytes_per_sec {
            tokio::time::sleep(Duration::from_millis(100)).await;
            self.current_bytes = self.current_bytes.saturating_sub(self.max_bytes_per_sec / 10);
        }
        self.current_bytes += bytes;
        Ok(())
    }
}
```

## 安全考虑

### 1. 消息验证
```rust
pub fn verify_message(message: &Message, expected_hash: Hash) -> bool {
    let actual_hash = hash_message(message);
    actual_hash == expected_hash
}
```

### 2. 节点认证
```rust
pub struct NodeAuth {
    public_key: PublicKey,
    signature: Signature,
}

impl NodeAuth {
    pub fn verify(&self, data: &[u8]) -> bool {
        self.public_key.verify(data, &self.signature)
    }
}
```

### 3. 加密传输
```rust
pub struct EncryptedTransport {
    cipher: ChaCha20Poly1305,
}

impl EncryptedTransport {
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // 加密数据
    }
    
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>> {
        // 解密数据
    }
}
```

## 实施步骤

### 阶段 1: 基础 P2P 框架
1. 实现节点发现机制
2. 实现 P2P 连接管理
3. 实现基础消息传输

### 阶段 2: 差异检测
1. 实现 Merkle Tree
2. 实现差异检测算法
3. 实现智能同步策略

### 阶段 3: 性能优化
1. 实现并行下载
2. 实现缓存机制
3. 实现带宽限制

### 阶段 4: 安全增强
1. 实现消息验证
2. 实现节点认证
3. 实现加密传输

## 预期效果

### 性能提升
- **服务器负载**: 降低 70-90%
- **同步速度**: 提高 3-5 倍
- **网络延迟**: 降低 50-70%
- **带宽成本**: 降低 60-80%

### 可扩展性
- **支持更多成员**: 单群可支持 1000+ 成员
- **支持更大消息量**: 历史消息可达到 100 万条
- **跨地域优化**: 自动选择最近的节点

## 技术选型

### Rust 生态
- **libp2p**: P2P 网络框架
- **tokio**: 异步运行时
- **bincode**: 序列化
- **zstd**: 压缩
- **sha2**: 哈希计算

### 可选依赖
- **quinn**: QUIC 传输协议
- **yamux**: 多路复用
- **noise**: 加密协议
