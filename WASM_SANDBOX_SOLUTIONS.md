# Wasm沙箱解决方案

## 问题总结

经过OpenClaw项目审计，我们发现：
- OpenClaw是一个成熟的AI助手平台，但无法直接编译到Wasm
- 当前WasmEdge沙箱需要改进以支持真正的JavaScript执行
- 需要借鉴OpenClaw的安全设计理念

## 推荐解决方案

### 方案1: 改进当前沙箱（立即执行）

**目标**: 借鉴OpenClaw设计理念，快速改进当前沙箱

**实施步骤**:

1. **添加工具策略系统**
   - 实现allow/deny工具列表
   - 支持通配符匹配
   - 添加工具权限验证

2. **增强工作空间隔离**
   - 改进工作空间创建逻辑
   - 添加工作空间清理策略
   - 实现工作空间权限控制

3. **环境变量清理**
   - 实现环境变量过滤
   - 添加敏感变量检测
   - 支持自定义环境变量策略

4. **增强安全验证**
   - 添加路径遍历检测
   - 实现命令注入防护
   - 加强输入验证

**优势**:
- 无需外部依赖
- 快速实施（1-2天）
- 完全可控
- 立即见效

**代码实现位置**: `src-tauri/src/wasmedge_sandbox.rs`

### 方案2: WasmEdge QuickJS集成（近期执行）

**目标**: 实现真正的Wasm级别JavaScript执行

**实施步骤**:

1. **获取WasmEdge QuickJS运行时**
   ```bash
   # 手动下载从GitHub releases
   # https://github.com/WasmEdge/wasmedge-quickjs/releases
   wget -O src-tauri/resources/wasmedge_quickjs.wasm \
       https://github.com/WasmEdge/wasmedge-quickjs/releases/download/v0.4.0/wasmedge_quickjs.wasm
   ```

2. **添加Wasmtime依赖**
   ```toml
   # src-tauri/Cargo.toml
   wasmtime = "20"
   wasmtime-wasi = "20"
   ```

3. **实现Wasm执行引擎**
   - 使用Wasmtime加载WasmEdge QuickJS
   - 配置WASI进行文件系统隔离
   - 实现JavaScript代码注入和执行
   - 添加资源限制（燃料、内存）

4. **更新沙箱接口**
   - 替换模拟执行为真实Wasm执行
   - 保持现有的Tauri命令接口
   - 添加Wasm特定的错误处理

**优势**:
- 真正的Wasm级别隔离
- 支持动态JavaScript执行
- 性能良好
- 与当前架构兼容

**预计时间**: 1周

### 方案3: 原生Wasm智能体（长期规划）

**目标**: 用Rust编写轻量级AI智能体，编译到Wasm

**实施步骤**:

1. **设计智能体架构**
   - 定义智能体接口
   - 设计消息处理系统
   - 规划工具调用机制

2. **实现核心功能**
   - 基础的消息处理
   - 简单的工具系统
   - 状态管理

3. **编译到Wasm**
   - 使用wasm-pack
   - 配置WASI支持
   - 优化性能

4. **集成到沙箱**
   - 添加Wasm智能体加载
   - 实现通信接口
   - 添加监控和日志

**优势**:
- 原生Wasm性能
- 完全可控
- 类型安全
- 资源效率高

**预计时间**: 2-4周

## 具体实施计划

### 第一阶段：立即改进（方案1）

**时间**: 1-2天

**任务**:
1. 实现工具策略系统
2. 增强工作空间隔离
3. 添加环境变量清理
4. 改进安全验证

**验收标准**:
- 工具策略正确过滤不允许的操作
- 工作空间完全隔离
- 环境变量安全清理
- 安全测试通过

### 第二阶段：Wasm集成（方案2）

**时间**: 1周

**任务**:
1. 下载和配置WasmEdge QuickJS
2. 添加Wasmtime依赖
3. 实现Wasm执行引擎
4. 更新沙箱接口
5. 测试和调试

**验收标准**:
- 成功加载和执行WasmEdge QuickJS
- JavaScript代码正确执行
- 文件系统隔离有效
- 资源限制工作正常

### 第三阶段：长期规划（方案3）

**时间**: 2-4周（可选）

**任务**:
1. 设计智能体架构
2. 实现核心功能
3. 编译到Wasm
4. 集成和测试

**验收标准**:
- 智能体基本功能正常
- Wasm编译成功
- 集成测试通过
- 性能满足要求

## 立即行动建议

**现在开始**: 方案1（改进当前沙箱）

这是最简单且能立即见效的方案。我将：

1. 在 `wasmedge_sandbox.rs` 中添加工具策略系统
2. 改进工作空间隔离机制
3. 实现环境变量清理
4. 增强安全验证

**下一步**: 方案2（WasmEdge QuickJS集成）

在方案1完成后，开始实施真正的Wasm执行能力。

**可选**: 方案3（原生Wasm智能体）

根据需求和资源决定是否实施。

## 技术细节

### 工具策略系统设计

```rust
// 工具策略配置
pub struct ToolPolicy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

// 工具权限检查
pub fn is_tool_allowed(tool: &str, policy: &ToolPolicy) -> bool {
    // 检查deny列表
    for pattern in &policy.deny {
        if matches_pattern(tool, pattern) {
            return false;
        }
    }
    
    // 检查allow列表
    if policy.allow.is_empty() {
        return true; // 空allow列表表示允许所有
    }
    
    for pattern in &policy.allow {
        if matches_pattern(tool, pattern) {
            return true;
        }
    }
    
    false
}
```

### 环境变量清理

```rust
pub fn sanitize_env_vars(env: &std::collections::HashMap<String, String>) -> std::collections::HashMap<String, String> {
    let mut sanitized = std::collections::HashMap::new();
    
    let sensitive_keys = vec![
        "PASSWORD", "TOKEN", "SECRET", "KEY", "API_KEY",
        "PRIVATE", "CREDENTIAL", "AUTH"
    ];
    
    for (key, value) in env {
        let is_sensitive = sensitive_keys.iter()
            .any(|sensitive| key.to_uppercase().contains(sensitive));
        
        if is_sensitive {
            sanitized.insert(key.clone(), "***REDACTED***".to_string());
        } else {
            sanitized.insert(key.clone(), value.clone());
        }
    }
    
    sanitized
}
```

### Wasm执行引擎设计

```rust
use wasmtime::*;
use wasmtime_wasi::*;

pub struct WasmExecutor {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl WasmExecutor {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_simd(true);
        config.consume_fuel(true);
        let engine = Engine::new(&config)?;
        
        let mut linker = Linker::new(&engine);
        wasi::add_to_linker(&mut linker, |s| s)?;
        
        Ok(Self { engine, linker })
    }
    
    pub fn execute(&self, wasm_bytes: &[Vec<u8>], script: &str) -> Result<String> {
        // 加载Wasm模块
        let module = Module::new(&self.engine, &wasm_bytes)?;
        
        // 配置WASI
        let wasi = WasiCtxBuilder::new()
            .inherit_stdin()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        
        // 创建存储
        let mut store = Store::new(&self.engine, wasi);
        store.set_fuel(1_000_000_000)?;
        
        // 实例化并执行
        let instance = self.linker.instantiate(&mut store, &module)?;
        let run_func = instance.get_typed_func::<(), ()>(&mut store, "run")?;
        run_func.call(&mut store, ())?;
        
        Ok("Execution completed".to_string())
    }
}
```

## 总结

**推荐执行顺序**:
1. ✅ 立即执行方案1（改进当前沙箱）
2. 🔄 近期执行方案2（WasmEdge QuickJS集成）
3. 📋 长期规划方案3（原生Wasm智能体）

这种渐进式方法可以：
- 快速改善当前系统
- 逐步增强能力
- 控制风险和复杂度
- 保持灵活性

我将从方案1开始实施，您同意这个计划吗？
