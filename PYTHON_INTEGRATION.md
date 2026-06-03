# Exodus Browser - Python 微服务集成文档

## 概述

Exodus 浏览器采用**微服务架构**实现 Python 集成，而不是使用 PyO3 内嵌方式。这符合微服务的设计原则，确保故障隔离、可扩展性和可维护性。

## 架构对比

### PyO3 (内嵌式) vs 微服务 (独立进程)

| 特性 | PyO3 (内嵌式) | 微服务 (独立进程) |
|------|--------------|------------------|
| **进程** | Python 嵌入 Rust 进程 | Python 独立进程 |
| **通信** | 直接函数调用 | IPC/JSON-RPC |
| **性能** | ⭐⭐⭐⭐⭐ (最快) | ⭐⭐⭐⭐ (稍慢) |
| **故障隔离** | ❌ Python 崩溃影响 Rust | ✅ 完全隔离 |
| **可扩展性** | ❌ 受限于 Rust 进程 | ✅ 独立扩展 |
| **重启** | ❌ 需重启整个进程 | ✅ 独立重启 |
| **监控** | ❌ 难以独立监控 | ✅ 完全独立监控 |
| **符合微服务** | ❌ 不符合 | ✅ 完全符合 |

**Exodus 选择微服务架构**，因为：
- ✅ 完全的故障隔离
- ✅ 独立的进程管理和监控
- ✅ 符合微服务设计原则
- ✅ 可以独立扩展和重启
- ✅ 性能损失可接受（IPC 延迟 < 1ms）

## 微服务架构

### 组件

```
┌─────────────────────────────────────────┐
│        Exodus Browser (Rust)            │
│  ┌───────────────────────────────────┐  │
│  │   Python Microservice Manager    │  │
│  │   - 进程管理                      │  │
│  │   - IPC 通信                      │  │
│  │   - 故障恢复                      │  │
│  └──────────────┬────────────────────┘  │
│                 │ Unix Domain Socket    │
│                 ▼                        │
│  ┌───────────────────────────────────┐  │
│  │   Python Service (独立进程)       │  │
│  │   - NumPy/Pandas/PyTorch 支持     │  │
│  │   - 代码执行                      │  │
│  │   - 变量管理                      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### 核心模块

#### 1. PythonMicroservice (Rust)

**文件**: `src-tauri/src/python_microservice.rs`

**功能**:
- Python 进程管理（启动、停止、重启）
- IPC 通信（Unix Domain Socket）
- 故障检测和自动恢复
- 配置管理

**主要 API**:
```rust
// 启动服务
service.start().await

// 停止服务
service.stop().await

// 执行代码
service.execute(request).await

// 获取状态
service.get_info().await
```

#### 2. Python Service (Python)

**文件**: `python_service.py`

**功能**:
- Unix Domain Socket 服务器
- Python 代码执行
- 变量管理
- 健康检查
- 可选库支持（NumPy、Pandas、PyTorch）

**主要 API**:
```python
# 执行代码
{"type": "execute", "code": "print('Hello')", "variables": {}}

# 设置变量
{"type": "set_variable", "name": "x", "value": 42}

# 获取变量
{"type": "get_variable", "name": "x"}

# 健康检查
{"type": "health"}
```

## 使用指南

### 1. 启动 Python 微服务

```typescript
// 前端调用
await invoke('python_microservice_start');
```

### 2. 执行 Python 代码

```typescript
const response = await invoke('python_microservice_execute', {
  request: {
    code: "x = 42\nprint(x)",
    variables: {},
    timeout_secs: 10
  }
});

console.log(response.output); // "42"
```

### 3. 获取服务状态

```typescript
const info = await invoke('python_microservice_get_info');
console.log(info.status); // "Running"
console.log(info.pid); // 进程 ID
```

### 4. 配置 Python 微服务

```typescript
await invoke('python_microservice_update_config', {
  config: {
    enabled: true,
    python_path: "/usr/bin/python3",
    script_path: "./python_service.py",
    socket_path: "/tmp/exodus-python.sock",
    enable_numpy: true,
    enable_pandas: false,
    enable_torch: false,
    max_memory_mb: 512,
    restart_on_failure: true,
    max_restarts: 3
  }
});
```

## Python 脚本部署

### 1. 准备 Python 环境

```bash
# 安装 Python 3.8+
python3 --version

# 安装可选库（根据需要）
pip install numpy
pip install pandas
pip install torch
```

### 2. 部署服务脚本

```bash
# 复制脚本到项目目录
cp python_service.py /path/to/exodus/

# 设置执行权限
chmod +x python_service.py
```

### 3. 手动测试服务

```bash
# 启动服务（手动）
python3 python_service.py \
  --socket-path /tmp/exodus-python.sock \
  --service-name python-service \
  --enable-numpy
```

## 高级功能

### 1. NumPy 集成

```typescript
const response = await invoke('python_microservice_execute', {
  request: {
    code: `
import numpy as np
arr = np.array([1, 2, 3, 4, 5])
print(arr.mean())
    `,
    variables: {}
  }
});
```

### 2. Pandas 集成

```typescript
const response = await invoke('python_microservice_execute', {
  request: {
    code: `
import pandas as pd
data = {'col1': [1, 2, 3], 'col2': [4, 5, 6]}
df = pd.DataFrame(data)
print(df.describe())
    `,
    variables: {}
  }
});
```

### 3. PyTorch 集成

```typescript
const response = await invoke('python_microservice_execute', {
  request: {
    code: `
import torch
x = torch.randn(2, 3)
print(x)
    `,
    variables: {}
  }
});
```

## 故障处理

### 自动重启

Python 微服务支持自动重启功能：

```typescript
const config = {
  restart_on_failure: true,
  max_restarts: 3
};
```

当服务崩溃时，会自动重启最多 3 次。

### 健康检查

定期检查服务状态：

```typescript
const info = await invoke('python_microservice_get_info');
if (info.status !== 'Running') {
  await invoke('python_microservice_restart');
}
```

## 性能优化

### 1. 减少通信开销

- 批量执行多个操作
- 使用变量缓存中间结果
- 避免频繁的小请求

### 2. 内存管理

```typescript
const config = {
  max_memory_mb: 512  // 限制 Python 进程内存
};
```

### 3. 超时设置

```typescript
const response = await invoke('python_microservice_execute', {
  request: {
    code: "...",
    timeout_secs: 30  // 30 秒超时
  }
});
```

## 安全考虑

### 1. Socket 权限

Unix Domain Socket 使用文件权限控制访问：

```typescript
// Socket 文件权限 0600（仅所有者可读写）
// 确保只有 Exodus 进程可以访问
```

### 2. 代码沙箱

在生产环境中，考虑：
- 限制可用的 Python 模块
- 使用 RestrictedPython 进行代码沙箱
- 设置执行超时
- 限制内存使用

### 3. 资源限制

```typescript
const config = {
  max_memory_mb: 512,
  max_restarts: 3
};
```

## 监控和日志

### 1. 服务状态监控

```typescript
setInterval(async () => {
  const info = await invoke('python_microservice_get_info');
  console.log('Python service:', info.status, info.restart_count);
}, 5000);
```

### 2. 日志输出

Python 服务输出到 stderr：

```bash
# 查看日志
python3 python_service.py --socket-path /tmp/exodus-python.sock 2>&1 | tee python.log
```

## 与其他微服务集成

Python 微服务可以与其他微服务配合使用：

### 1. 与 Hermes 智能体集成

```typescript
// 创建任务让 Python 执行
const taskId = await invoke('hermes_create_task', {
  task_type: 'Custom',
  description: 'Execute Python analysis',
  priority: 1,
  metadata: {}
});

// 执行 Python 代码
const result = await invoke('python_microservice_execute', {
  request: { code: "..." }
});
```

### 2. 与指标系统集成

```typescript
// 记录 Python 执行指标
await invoke('metrics_histogram', {
  name: 'python_execution_duration_ms',
  value: response.execution_time_ms
});
```

## 常见问题

### Q: 为什么不使用 PyO3？

A: PyO3 将 Python 嵌入到 Rust 进程中，违反了微服务的故障隔离原则。如果 Python 崩溃，整个 Rust 进程也会崩溃。微服务架构确保 Python 作为独立进程运行，完全隔离。

### Q: 性能如何？

A: IPC 通信延迟 < 1ms，对于大多数应用场景完全足够。虽然比 PyO3 慢，但换来了更好的隔离性和可维护性。

### Q: 可以编译成 WASM 吗？

A: Python 编译成 WASM（如 Pyodide）适用于前端浏览器环境，但不适合后端微服务架构。对于 Exodus 的后端微服务，独立进程是最佳选择。

### Q: 如何调试？

A: 查看 stderr 输出，使用 Python 调试器，或在代码中添加 print 语句。

### Q: 支持哪些 Python 版本？

A: Python 3.8 及以上版本。

## 下一步

1. **Pyodide 前端支持** - 在前端使用 Pyodide 运行 Python（可选）
2. **更复杂的 IPC 协议** - 支持流式传输、二进制数据
3. **性能优化** - 使用共享内存减少拷贝
4. **安全增强** - 代码沙箱、权限控制

## 总结

Exodus 的 Python 微服务架构提供了：

- ✅ 完全的故障隔离
- ✅ 独立的进程管理
- ✅ 灵活的配置选项
- ✅ 自动故障恢复
- ✅ 与其他微服务无缝集成

这是符合微服务设计原则的最佳实践方案。

---

**文档版本**: 1.0  
**最后更新**: 2026-05-19  
**状态**: ✅ 生产就绪
