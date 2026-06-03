#!/bin/bash
# OpenClaw Wasm 构建脚本

set -e

echo "🚀 开始构建 OpenClaw Wasm..."

# 检查源文件
if [ ! -f "examples/ai_inference/chat/openclaw_agent.js" ]; then
    echo "❌ 错误: OpenClaw 源文件不存在"
    exit 1
fi

# 创建资源目录
mkdir -p src-tauri/resources

# 方案1: 使用QuickJS编译JavaScript
if command -v quickjs &> /dev/null; then
    echo "📦 使用QuickJS编译JavaScript到Wasm..."
    quickjs -c examples/ai_inference/chat/openclaw_agent.js -o src-tauri/resources/openclaw_agent.wasm
    echo "✅ JavaScript编译完成"
else
    echo "⚠️  QuickJS未安装，跳过JavaScript编译"
    echo "   安装方法: git clone https://github.com/quickjs-ng/quickjs.git && cd quickjs && make && make wasm"
fi

# 方案2: 如果有Rust实现，使用wasm-pack
if [ -d "openclaw-wasm" ]; then
    echo "📦 使用wasm-pack编译Rust到Wasm..."
    cd openclaw-wasm
    wasm-pack build --target web
    cp pkg/openclaw_wasm_bg.wasm ../src-tauri/resources/
    cd ..
    echo "✅ Rust编译完成"
else
    echo "ℹ️  未找到Rust实现，跳过Rust编译"
fi

# 下载WasmEdge QuickJS运行时
if [ ! -f "src-tauri/resources/wasmedge_quickjs.wasm" ]; then
    echo "📥 下载WasmEdge QuickJS运行时..."
    wget -O src-tauri/resources/wasmedge_quickjs.wasm \
        https://github.com/WasmEdge/wasmedge-quickjs/releases/download/v0.5.0/wasmedge_quickjs.wasm
    echo "✅ WasmEdge QuickJS下载完成"
else
    echo "ℹ️  WasmEdge QuickJS已存在"
fi

echo "🎉 OpenClaw Wasm构建完成!"
echo "📁 Wasm文件位置: src-tauri/resources/"
