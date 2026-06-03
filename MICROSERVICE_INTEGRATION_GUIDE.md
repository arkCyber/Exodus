# 微服务前后端集成指南

## 架构概览

### 前端 (TypeScript/Vue)
- **位置**: `src/composables/useMicroservice.ts`
- **功能**: JSON-RPC 2.0 客户端，带重试逻辑
- **调用方式**: 通过 Tauri `invoke` API 调用后端命令

### 后端 (Rust/Tauri)
- **位置**: `src-tauri/src/microservice_bridge.rs`
- **功能**: 微服务桥接器，转发 JSON-RPC 请求到 Unix Domain Socket
- **注册表**: `src-tauri/src/microservice/registry.rs` - 服务注册与健康检查

### 微服务
- **RAG Service**: `src-tauri/src/microservice/rag_service.rs`
- **其他 30+ 微服务**: AI 模型、文件传输、群组聊天等

## 数据流

```
前端 → useMicroservice.callMethod()
    ↓
Tauri invoke('invoke_microservice')
    ↓
MicroserviceBridge.invoke()
    ↓
ServiceRegistry.get(service_name)
    ↓
UnixStream.connect(socket_path)
    ↓
JSON-RPC 2.0 Request → Microservice
    ↓
JSON-RPC 2.0 Response → Frontend
```

## 使用示例

### 前端调用 RAG 服务

```typescript
import { useRagService } from '@/composables/useMicroservice';

const ragService = useRagService();

// 存储页面
await ragService.storePage(url, title, content);

// 搜索页面
const results = await ragService.searchPages(query);

// 添加书签
await ragService.addBookmark(url, title, folder);

// 记录访问
await ragService.recordVisit(url, title);
```

### 前端调用任意微服务

```typescript
import { useMicroservice } from '@/composables/useMicroservice';

const service = useMicroservice({
  name: 'crypto-service',
  retries: 3,
});

// 调用方法
const result = await service.callMethod('hash', { data: 'test' });
```

### 后端注册新微服务

```rust
// 在 setup() 中注册服务
let service_info = ServiceInfo::new(
    "my-service",
    socket_path.to_string_lossy().to_string(),
    std::process::id(),
);

registry.register(service_info)?;
```

## 已实现的 Tauri 命令

### 微服务桥接器
- `invoke_microservice(service_name, request)` - 通用微服务调用
- `is_microservice_available(service_name)` - 检查服务可用性
- `list_microservices()` - 列出所有注册的服务

### RAG 服务专用
- `rag_service_start(data_dir)` - 启动 RAG 服务
- `rag_service_stop()` - 停止 RAG 服务
- `rag_store_page(url, title, content)` - 存储页面
- `rag_search_pages(query)` - 搜索页面
- `rag_add_bookmark(url, title, folder)` - 添加书签
- `rag_list_bookmarks()` - 列出书签
- `rag_record_visit(url, title)` - 记录访问
- `rag_search_visits(query)` - 搜索访问历史

## 错误处理

### 前端重试逻辑
- 默认重试 3 次
- 指数退避：100ms, 200ms, 400ms
- 自动设置 loading 和 error 状态

### 后端错误处理
- 服务健康检查（30 秒超时）
- JSON-RPC 错误解析
- 连接错误处理
- 详细日志记录

## 测试

### 前端集成测试
- `src/composables/useMicroservice.integration.test.ts`
- 测试 JSON-RPC 协议
- 测试重试逻辑
- 测试状态管理
- 测试 RAG 服务方法

### 后端单元测试
- `src-tauri/src/microservice_bridge.rs` 中的测试
- 测试 JSON-RPC 序列化/反序列化
- 测试错误处理

## 部署检查清单

- [x] 前端 JSON-RPC 客户端实现
- [x] 后端微服务桥接器实现
- [x] Tauri 命令注册
- [x] RAG 服务集成
- [x] 错误处理和日志
- [x] 集成测试
- [x] 编译测试 (cargo build)
- [x] 运行时测试
- [x] 修复依赖问题 (lodash-es 替换为原生实现)
- [x] 侧边栏 UI 重新设计 (Firefox 风格)

## 下一步

1. **编译测试**: 运行 `cargo build -p exodus-tauri` 确保编译通过
2. **运行时测试**: 启动应用，测试 RAG 服务调用
3. **添加更多服务**: 为其他微服务创建专用 composable
4. **性能优化**: 添加连接池和批处理
5. **监控**: 添加服务调用指标收集
