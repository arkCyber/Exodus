# AI 推理应用案例

本目录包含使用 Allama LLM 推理引擎的实际应用案例，展示如何在 Exodus 浏览器中使用 AI 推理能力。

## 案例列表

1. **文本生成** (`text_generation/`) - 文本生成、内容创作、代码生成
2. **聊天对话** (`chat/`) - 多轮对话、角色扮演、智能助手
3. **嵌入生成** (`embeddings/`) - 文本嵌入、语义搜索、相似度计算
4. **上下文压缩** (`context_compression/`) - 长文本处理、内存优化

## 快速开始

每个案例目录包含：
- `frontend.ts` - 前端 TypeScript 调用示例
- `config.json` - 后端配置示例
- `README.md` - 详细使用说明

## 推理引擎配置

```typescript
// 配置推理引擎
await invoke('inference_update_config', {
  config: {
    enabled: true,
    model_path: './allama/models',
    backend_type: 'Allama',
    max_context_length: 2048,
    max_tokens: 512,
    temperature: 0.7,
    top_p: 0.9,
    top_k: 40,
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      compression_ratio: 0.5,
      quantization_bits: 4
    }
  }
});
```

## 模型管理

```typescript
// 添加模型
await invoke('inference_add_model', {
  model_info: {
    name: 'qwen3.6-35b-a3b',
    path: 'unsloth_Qwen3.6-35B-A3B-GGUF/Qwen3.6-35B-A3B-UD-Q4_K_M.gguf',
    size_bytes: 20000000000,
    quantization: 'Q4_K_M',
    parameters: '35B',
    context_length: 128000,
    loaded: false,
    backend: 'Allama'
  }
});

// 加载模型
await invoke('inference_load_model', { model_name: 'qwen3.6-35b-a3b' });
```

## 性能优化

### 上下文压缩

启用上下文压缩可以节省 50-70% 的内存使用：

```typescript
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      quantization_bits: 4
    }
  }
});
```

### GPU 加速

```typescript
await invoke('inference_update_config', {
  config: {
    n_gpu_layers: 35  // 将所有层加载到 GPU
  }
});
```

## 注意事项

- ⚠️ 模型文件较大（20GB+），需要足够的磁盘空间
- ⚠️ 推理需要大量内存（建议 32GB+）
- ⚠️ GPU 加速需要兼容的硬件
- ⚠️ 长上下文推理需要更多资源
- ⚠️ 首次加载模型需要时间

## 贡献

欢迎提交新的 AI 推理应用案例！
