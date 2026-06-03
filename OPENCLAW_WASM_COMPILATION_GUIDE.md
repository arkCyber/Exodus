# OpenClaw Wasm 编译指南

## 概述

本指南介绍如何将OpenClaw智能体代码编译成WebAssembly (Wasm) 格式，以便在WasmEdge沙箱中安全执行。

## 当前状态

经过调查，OpenClaw在当前项目中是一个**概念性的AI智能体名称**，用于描述安全的JavaScript智能体执行环境。当前实现使用JavaScript脚本来模拟OpenClaw智能体的行为。

## 编译方案

### 方案1: JavaScript → Wasm (使用QuickJS)

适用于将JavaScript代码编译成Wasm。

#### 前置要求

```bash
# 安装QuickJS
git clone https://github.com/quickjs-ng/quickjs.git
cd quickjs
make
make wasm
sudo make install
```

#### 编译步骤

1. **准备JavaScript源代码**

我已经创建了一个示例文件：`examples/ai_inference/chat/openclaw_agent.js`

2. **编译到Wasm**

```bash
# 使用QuickJS编译器
quickjs -c examples/ai_inference/chat/openclaw_agent.js -o src-tauri/resources/openclaw_agent.wasm
```

3. **验证Wasm文件**

```bash
# 检查Wasm文件是否有效
wasm-validate src-tauri/resources/openclaw_agent.wasm
```

### 方案2: Rust → Wasm (使用wasm-pack)

适用于将Rust代码编译成Wasm，提供更好的性能和类型安全。

#### 前置要求

```bash
# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装wasm-pack
cargo install wasm-pack
```

#### 创建Rust项目

```bash
# 创建新的Rust项目
mkdir openclaw-wasm
cd openclaw-wasm
cargo init --lib
```

#### 配置Cargo.toml

```toml
[package]
name = "openclaw-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
```

#### 实现OpenClaw智能体 (Rust)

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OpenClawAgent {
    name: String,
    version: String,
}

#[wasm_bindgen]
impl OpenClawAgent {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            name: "OpenClaw".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn execute_task(&self, task: &str) -> String {
        format!("Task '{}' completed successfully", task)
    }

    #[wasm_bindgen]
    pub fn security_check(&self, operation: &str) -> bool {
        let safe_operations = vec!["read", "write", "process"];
        safe_operations.contains(&operation)
    }
}
```

#### 编译到Wasm

```bash
# 编译为Web模块
wasm-pack build --target web

# 复制Wasm文件到资源目录
cp pkg/openclaw_wasm_bg.wasm ../src-tauri/resources/
```

### 方案3: 使用WasmEdge QuickJS运行时

这是最简单的方案，直接使用预编译的QuickJS运行时来执行JavaScript。

#### 下载WasmEdge QuickJS

```bash
# 下载预编译的WasmEdge QuickJS运行时
wget -O src-tauri/resources/wasmedge_quickjs.wasm \
    https://github.com/WasmEdge/wasmedge-quickjs/releases/download/v0.5.0/wasmedge_quickjs.wasm
```

#### 使用方法

JavaScript代码不需要预先编译，而是直接在运行时由QuickJS解释器执行：

```javascript
// 在沙箱中执行的JavaScript脚本
const agent = {
    name: "OpenClaw",
    executeTask: function(task) {
        console.log(`Executing: ${task}`);
        return { status: "completed", task: task };
    }
};

// 使用智能体
const result = agent.executeTask("Analyze data");
console.log(result);
```

## 集成到沙箱

### 更新沙箱代码以支持Wasm执行

在 `src-tauri/src/wasmedge_sandbox.rs` 中添加Wasm执行支持：

```rust
use wasmtime::*;
use wasmtime_wasi::{WasiCtxBuilder, preview1};

fn execute_wasm_in_sandbox(
    app_handle: &AppHandle,
    wasm_path: &Path,
    script_content: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. 加载Wasm模块
    let resource_path = app_handle.path().resource_dir()?;
    let wasm_bytes = fs::read(wasm_path)?;
    
    // 2. 配置Wasmtime引擎
    let mut config = Config::new();
    config.wasm_simd(true);
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    
    // 3. 配置WASI
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder.inherit_stdin();
    wasi_builder.inherit_stdout();
    wasi_builder.inherit_stderr();
    let wasi = wasi_builder.build();
    
    // 4. 创建链接器
    let mut linker = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |s| s)?;
    
    // 5. 编译模块
    let module = Module::new(&engine, &wasm_bytes)?;
    
    // 6. 创建存储并设置燃料限制
    let mut store = Store::new(&engine, wasi);
    store.set_fuel(1_000_000_000)?;
    
    // 7. 实例化模块
    let instance = linker.instantiate(&mut store, &module)?;
    
    // 8. 执行Wasm函数
    let run_func = instance.get_typed_func::<(), ()>(&mut store, "run")?;
    run_func.call(&mut store, ())?;
    
    Ok("Wasm execution completed".to_string())
}
```

## 推荐方案

### 对于当前项目

**推荐使用方案3（WasmEdge QuickJS运行时）**，原因：

1. **简单易用**: 不需要预先编译JavaScript代码
2. **灵活性**: 可以动态执行不同的JavaScript脚本
3. **兼容性**: 与当前的JavaScript测试脚本完全兼容
4. **安全性**: 仍然提供Wasm级别的隔离保护

### 实施步骤

1. **手动下载WasmEdge QuickJS运行时**
   
   由于GitHub releases可能不稳定，建议手动下载：
   
   ```bash
   # 访问官方发布页面下载
   # https://github.com/WasmEdge/wasmedge-quickjs/releases
   
   # 或者使用以下命令（如果可用）
   wget -O src-tauri/resources/wasmedge_quickjs.wasm \
       https://github.com/WasmEdge/wasmedge-quickjs/releases/download/v0.4.0/wasmedge_quickjs.wasm
   ```

   **注意**: 如果下载失败，请从GitHub releases页面手动下载并放置到 `src-tauri/resources/` 目录。

2. **更新Cargo.toml添加Wasmtime依赖**
   ```toml
   wasmtime = "20"
   wasmtime-wasi = "20"
   ```

3. **更新沙箱代码以使用真正的Wasm执行**
   - 替换当前的模拟执行
   - 使用Wasmtime加载和执行WasmEdge QuickJS
   - 配置WASI进行文件系统隔离

4. **测试Wasm执行**
   - 运行现有的安全测试
   - 验证隔离效果
   - 检查性能表现

## 性能对比

| 方案 | 编译时间 | 执行性能 | 灵活性 | 安全性 |
|------|---------|---------|--------|--------|
| QuickJS编译 | 中 | 高 | 低 | 高 |
| Rust+wasm-pack | 长 | 最高 | 低 | 最高 |
| WasmEdge QuickJS | 无 | 中 | 高 | 高 |

## 下一步

1. **选择方案**: 根据项目需求选择合适的编译方案
2. **实施集成**: 将选定的方案集成到沙箱中
3. **测试验证**: 全面测试Wasm执行的安全性和性能
4. **文档更新**: 更新相关文档以反映新的实现

## 参考资料

- [WasmEdge QuickJS](https://github.com/WasmEdge/wasmedge-quickjs)
- [Wasmtime](https://wasmtime.dev/)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [QuickJS](https://bellard.org/quickjs/)
- [WebAssembly](https://webassembly.org/)
