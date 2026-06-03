# Rust 应用测试案例

本目录包含 Exodus 浏览器 Rust 后端的测试案例和验证脚本。

## 测试列表

1. **推理引擎测试** - 验证 Allama 推理引擎功能
2. **Hermes 智能体测试** - 验证智能体任务管理
3. **Python 微服务测试** - 验证 Python 微服务集成
4. **微服务架构测试** - 验证整体微服务架构

## 运行测试

### 运行所有测试

```bash
cd src-tauri
cargo test
```

### 运行特定测试

```bash
# 推理引擎测试
cargo test inference_engine

# Hermes 智能体测试
cargo test hermes_agent

# Python 微服务测试
cargo test python_microservice
```

### 运行测试并显示输出

```bash
cargo test -- --nocapture
```

## 测试覆盖

- ✅ 单元测试
- ✅ 集成测试
- ✅ 错误处理测试
- ✅ 性能测试
