# 容错架构文档

## 概述

本文档描述了 Exodus 浏览器系统的容错架构设计、实现和使用指南。容错架构旨在提高系统的稳定性、可用性和可靠性，通过多种模式确保服务在面临故障时能够优雅降级和自动恢复。

## 容错架构组件

### 1. 统一重试机制 (`resilience::retry_with_backoff`)

#### 功能描述
提供带指数退避和随机抖动的重试机制，用于处理临时性故障。

#### 重试策略

- **Transient**: 适用于短暂性故障
  - 最大重试次数: 3
  - 初始延迟: 50ms
  - 最大延迟: 1s
  - 退避乘数: 1.5
  - 抖动因子: 0.2

- **Aggressive**: 适用于网络问题
  - 最大重试次数: 5
  - 初始延迟: 100ms
  - 最大延迟: 10s
  - 退避乘数: 2.0
  - 抖动因子: 0.1

- **Conservative**: 适用于关键操作
  - 最大重试次数: 2
  - 初始延迟: 200ms
  - 最大延迟: 2s
  - 退避乘数: 1.2
  - 抖动因子: 0.3

#### 使用示例

```rust
use crate::microservice::resilience::{retry_with_backoff, RetryPolicy};

let result = retry_with_backoff(
    || async {
        // 执行可能失败的操作
        some_async_operation().await
    },
    RetryPolicy::Transient.to_config(),
).await?;
```

### 2. 熔断器 (`resilience::ResilientHttpClient`)

#### 功能描述
集成 Circuit Breaker 模式的 HTTP 客户端，防止级联故障。

#### 特性
- 自动熔断：当故障超过阈值时停止请求
- 半开状态：定期尝试恢复服务
- 统计监控：记录请求成功/失败次数
- 自动恢复：服务恢复后自动关闭熔断器

#### 配置选项

```rust
use crate::microservice::resilience::CircuitBreakerConfig;

let config = CircuitBreakerConfig {
    failure_threshold: 5,        // 失败阈值
    timeout: Duration::from_secs(60),  // 熔断超时
    success_threshold: 2,        // 成功阈值（用于恢复）
    window_duration: Duration::from_secs(30),  // 统计窗口
};
```

#### 使用示例

```rust
use crate::microservice::resilience::ResilientHttpClient;

let client = ResilientHttpClient::new();
let response = client.get("https://api.example.com/data").await?;
```

### 3. 健康检查监控 (`resilience::HealthMonitor`)

#### 功能描述
跟踪和监控服务健康状态，提供系统整体健康评估。

#### 健康状态

- **Healthy**: 服务正常运行
- **Degraded**: 服务性能下降但可用
- **Unhealthy**: 服务不可用

#### 使用示例

```rust
use crate::microservice::resilience::{HealthMonitor, HealthCheck, HealthStatus};

let monitor = HealthMonitor::new();

// 记录健康检查
monitor.record_check(HealthCheck {
    service_name: "api-service".to_string(),
    status: HealthStatus::Healthy,
    message: "All systems operational".to_string(),
    last_check: Instant::now(),
}).await;

// 获取系统整体健康状态
let system_health = monitor.get_system_health().await;
```

### 4. 故障转移机制 (`resilience::Fallback`)

#### 功能描述
提供主备切换机制，当主服务失败时自动切换到备用服务。

#### 特性
- 主备自动切换
- 故障状态跟踪
- 优雅降级
- 自动恢复

#### 使用示例

```rust
use crate::microservice::resilience::Fallback;

let fallback = Fallback::with_fallback(primary_service, backup_service);

let result = fallback.execute(|service| async move {
    service.process_request().await
}).await?;
```

### 5. 降级管理 (`resilience::DegradationManager`)

#### 功能描述
根据系统指标自动调整服务降级级别，防止系统过载。

#### 降级级别

- **Full**: 完全功能
- **Degraded**: 部分功能受限
- **Minimal**: 最小功能集
- **Offline**: 服务下线

#### 降级阈值

```rust
use crate::microservice::resilience::DegradationThresholds;

let thresholds = DegradationThresholds {
    error_rate_threshold: 0.5,      // 错误率阈值 50%
    latency_threshold_ms: 5000,     // 延迟阈值 5秒
    circuit_breaker_threshold: 5,   // 熔断器阈值
};
```

#### 使用示例

```rust
use crate::microservice::resilience::DegradationManager;

let manager = DegradationManager::new();

// 根据指标评估并调整降级级别
manager.evaluate_and_adjust(
    error_rate,
    latency_ms,
    circuit_failures,
).await;

let current_level = manager.get_current_level().await;
```

## 已集成的容错机制

### 1. Allama HTTP 客户端

所有 API 方法已集成重试机制：

- `list_model_names()`: Transient 重试策略
- `generate()`: Aggressive 重试策略
- `chat()`: Aggressive 重试策略
- `embed()`: Transient 重试策略
- `embed_openai()`: Transient 重试策略

### 2. Embeddings 服务

- `fetch_embedding()`: 使用 Transient 重试策略
- 超时配置: 30 秒
- 自动重试临时性故障

### 3. P2P CDN 模块

- `fetch_from_mesh_peers()`: 使用重试机制
- `fetch_from_ticket()`: Transient 重试策略
- `fetch_chunks_parallel()`: Transient 重试策略
- 超时配置: 3600 秒（1 小时）

## 容错架构测试

### 测试覆盖

```bash
# 运行容错架构测试
cargo test --lib microservice::resilience
```

### 测试用例

- `test_retry_with_backoff_success`: 测试重试成功场景
- `test_retry_with_backoff_exhausted`: 测试重试耗尽场景
- `test_health_monitor`: 测试健康监控
- `test_health_monitor_system_health`: 测试系统整体健康评估
- `test_fallback_triggers`: 测试故障转移触发
- `test_degradation_manager`: 测试降级管理器
- `test_degradation_manager_auto_adjust`: 测试自动降级调整
- `test_retry_policy_configs`: 测试重试策略配置

## 最佳实践

### 1. 选择合适的重试策略

- **Transient**: 适用于短暂性网络故障、临时服务不可用
- **Aggressive**: 适用于网络不稳定、需要尽力而为的场景
- **Conservative**: 适用于关键操作、不能容忍重复执行的场景

### 2. 配置合理的超时时间

- 根据操作复杂度设置超时
- 考虑网络延迟和服务响应时间
- 避免设置过短导致误报，或过长导致资源浪费

### 3. 监控和告警

- 定期检查健康状态
- 监控熔断器状态
- 设置降级告警阈值
- 记录故障事件

### 4. 优雅降级

- 为关键服务提供备用方案
- 实现功能降级而非完全失败
- 保持核心功能可用
- 提供用户友好的错误信息

## 容错架构改进路线图

### 已完成

- ✅ 统一重试机制实现
- ✅ 熔断器集成
- ✅ 健康检查监控
- ✅ 故障转移机制
- ✅ 降级管理
- ✅ Allama HTTP 客户端容错
- ✅ Embeddings 服务容错
- ✅ P2P CDN 模块容错
- ✅ 综合测试覆盖

### 计划中

- 🔄 为其他 HTTP 客户端添加容错机制
- 🔄 实现分布式追踪集成
- 🔄 添加更详细的监控指标
- 🔄 实现自动故障恢复策略
- 🔄 添加容错配置热更新

## 故障排查

### 常见问题

#### 1. 重试不生效

**检查项:**
- 确认使用了正确的重试策略
- 检查错误类型是否为可重试错误
- 验证重试配置参数

#### 2. 熔断器频繁触发

**检查项:**
- 检查服务健康状态
- 调整故障阈值
- 检查网络连接稳定性
- 增加超时时间

#### 3. 降级级别不正确

**检查项:**
- 验证阈值配置
- 检查指标计算逻辑
- 确认自动调整策略

## 性能影响

### 重试机制

- 增加延迟：重试会增加总响应时间
- 资源消耗：重试会增加网络和 CPU 使用
- 建议：合理设置重试次数和超时

### 熔断器

- 内存开销：维护状态需要少量内存
- CPU 开销：状态检查需要少量 CPU
- 建议：在高并发场景下监控性能指标

### 健康检查

- 网络开销：定期检查需要网络请求
- 建议调整检查频率以平衡开销和及时性

## 相关文档

- [Circuit Breaker 模式](./CIRCUIT_BREAKER.md)
- [微服务架构](./MICROSERVICES.md)
- [服务发现](./SERVICE_DISCOVERY.md)
- [分布式追踪](./DISTRIBUTED_TRACING.md)

## 贡献指南

如需改进容错架构，请遵循以下步骤：

1. 提出改进建议
2. 设计容错方案
3. 实现功能
4. 添加测试用例
5. 更新文档
6. 提交 Pull Request

## 联系方式

如有问题或建议，请联系开发团队或提交 Issue。

---

**最后更新**: 2026-05-21
**版本**: 1.0.0
