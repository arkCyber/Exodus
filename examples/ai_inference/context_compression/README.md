# 上下文压缩案例

使用 Allama 推理引擎的上下文压缩（TQ）功能处理长文本，优化内存使用。

## 场景描述

处理长文档、长对话、大量历史记录时，使用上下文压缩技术节省内存并提高性能。

## 前端调用示例

### 启用 Token 量化

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 配置 Token 量化
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      quantization_bits: 4,  // 4-bit 量化
      compression_ratio: 0.5
    }
  }
});

// 处理长文本
const longText = '这是一个非常长的文本...（10000+ tokens）';
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: longText,
    max_tokens: 1000
  }
});

console.log('压缩统计:', await invoke('inference_get_stats'));
```

### 启用 KV Cache 压缩

```typescript
// 配置 KV Cache 压缩
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'KVCacheCompression',
      kv_cache_compression: true,
      compression_ratio: 0.6
    }
  }
});

// 长对话
const longConversation = [
  { role: 'user', content: '开始对话...' },
  // ... 50+ 轮对话
  { role: 'user', content: '这是第50条消息' }
];

const response = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: longConversation,
    max_tokens: 300
  }
});
```

### 启用滑动窗口

```typescript
// 配置滑动窗口
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'StreamingLLM',
      sliding_window: true,
      sliding_window_size: 512
    }
  }
});

// 流式处理长文档
const document = '长文档内容...';
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: document,
    max_tokens: 2000,
    stream: true
  }
});
```

## 高级用法

### 动态调整压缩策略

```typescript
class AdaptiveCompression {
  private currentMethod = 'TokenQuantization';
  
  async adjustBasedOnMemory(memoryUsage: number) {
    if (memoryUsage > 0.8) {
      // 高内存使用，启用强压缩
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: true,
            compression_method: 'TokenQuantization',
            quantization_bits: 2,
            compression_ratio: 0.3
          }
        }
      });
    } else if (memoryUsage > 0.5) {
      // 中等内存使用，启用中等压缩
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: true,
            compression_method: 'KVCacheCompression',
            compression_ratio: 0.5
          }
        }
      });
    } else {
      // 低内存使用，禁用压缩
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: false
          }
        }
      });
    }
  }
}

const adaptive = new AdaptiveCompression();
await adaptive.adjustBasedOnMemory(0.7);
```

### 混合压缩策略

```typescript
// 结合多种压缩方法
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      quantization_bits: 4,
      kv_cache_compression: true,
      sliding_window: true,
      sliding_window_size: 1024
    }
  }
});

// 处理超长文本
const ultraLongText = '超长文本...（50000+ tokens）';
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: ultraLongText,
    max_tokens: 3000
  }
});
```

### 监控压缩效果

```typescript
// 执行推理
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: longText,
    max_tokens: 1000
  }
});

// 获取压缩统计
const stats = await invoke('inference_get_stats');
const compressionStats = stats.compression_stats;

console.log('压缩统计:');
console.log('- 原始 tokens:', compressionStats.original_tokens);
console.log('- 压缩后 tokens:', compressionStats.compressed_tokens);
console.log('- 压缩比率:', compressionStats.compression_ratio);
console.log('- KV Cache 节省:', compressionStats.kv_cache_saved_mb, 'MB');
console.log('- 滑动窗口使用:', compressionStats.sliding_window_used);
```

### 批量处理优化

```typescript
async function processWithCompression(texts: string[]) {
  const results = [];
  
  for (const text of texts) {
    // 根据文本长度动态调整
    const tokenCount = text.split(/\s+/).length;
    
    if (tokenCount > 10000) {
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: true,
            compression_method: 'StreamingLLM',
            sliding_window_size: 512
          }
        }
      });
    } else if (tokenCount > 5000) {
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: true,
            compression_method: 'TokenQuantization',
            quantization_bits: 4
          }
        }
      });
    } else {
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: false
          }
        }
      });
    }
    
    const response = await invoke('inference_generate', {
      request: {
        model: 'qwen3.6-35b-a3b',
        prompt: text,
        max_tokens: 500
      }
    });
    
    results.push(response);
  }
  
  return results;
}
```

## 性能对比

### 无压缩 vs 有压缩

```typescript
// 无压缩
await invoke('inference_update_config', {
  config: {
    context_compression: { enabled: false }
  }
});

const start1 = Date.now();
const response1 = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: longText,
    max_tokens: 1000
  }
});
const time1 = Date.now() - start1;

// 有压缩
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      quantization_bits: 4
    }
  }
});

const start2 = Date.now();
const response2 = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: longText,
    max_tokens: 1000
  }
});
const time2 = Date.now() - start2;

console.log('无压缩时间:', time1, 'ms');
console.log('有压缩时间:', time2, 'ms');
console.log('速度提升:', ((time1 - time2) / time1 * 100).toFixed(2), '%');
```

### 不同压缩方法对比

```typescript
const methods = [
  'TokenQuantization',
  'KVCacheCompression',
  'AttentionSink',
  'StreamingLLM'
];

for (const method of methods) {
  await invoke('inference_update_config', {
    config: {
      context_compression: {
        enabled: true,
        compression_method: method
      }
    }
  });
  
  const start = Date.now();
  const response = await invoke('inference_generate', {
    request: {
      model: 'qwen3.6-35b-a3b',
      prompt: longText,
      max_tokens: 1000
    }
  });
  const time = Date.now() - start;
  
  const stats = await invoke('inference_get_stats');
  
  console.log(`${method}:`);
  console.log(`  时间: ${time}ms`);
  console.log(`  压缩比率: ${stats.compression_stats.compression_ratio}`);
}
```

## 错误处理

```typescript
try {
  const response = await invoke('inference_generate', {
    request: {
      model: 'qwen3.6-35b-a3b',
      prompt: longText,
      max_tokens: 1000
    }
  });

  if (!response.success) {
    if (response.error.includes('memory')) {
      console.log('内存不足，启用更强的压缩');
      await invoke('inference_update_config', {
        config: {
          context_compression: {
            enabled: true,
            compression_method: 'TokenQuantization',
            quantization_bits: 2,
            compression_ratio: 0.3
          }
        }
      });
      // 重试
    }
  }
} catch (error) {
  console.error('压缩错误:', error);
}
```

## 最佳实践

### 1. 根据文本长度选择压缩方法

```typescript
function selectCompressionMethod(tokenCount: number) {
  if (tokenCount > 20000) {
    return 'StreamingLLM';  // 超长文本
  } else if (tokenCount > 10000) {
    return 'TokenQuantization';  // 长文本
  } else if (tokenCount > 5000) {
    return 'KVCacheCompression';  // 中等长度
  } else {
    return 'Custom';  // 短文本无需压缩
  }
}
```

### 2. 监控内存使用

```typescript
setInterval(async () => {
  const stats = await invoke('inference_get_stats');
  const memoryUsage = process.memoryUsage().heapUsed / process.memoryUsage().heapTotal;
  
  if (memoryUsage > 0.8) {
    console.warn('内存使用过高，启用压缩');
    await invoke('inference_update_config', {
      config: {
        context_compression: {
          enabled: true,
          compression_method: 'TokenQuantization',
          quantization_bits: 2
        }
      }
    });
  }
}, 5000);
```

### 3. 批量处理时的压缩策略

```typescript
async function batchProcessWithCompression(items: any[]) {
  const batchSize = 5;
  
  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize);
    const totalTokens = batch.reduce((sum, item) => 
      sum + item.text.split(/\s+/).length, 0
    );
    
    // 根据批量大小调整压缩
    const compressionRatio = Math.max(0.3, 1 - totalTokens / 50000);
    
    await invoke('inference_update_config', {
      config: {
        context_compression: {
          enabled: true,
          compression_ratio
        }
      }
    });
    
    // 处理批次
    await Promise.all(
      batch.map(item =>
        invoke('inference_generate', {
          request: {
            model: 'qwen3.6-35b-a3b',
            prompt: item.text,
            max_tokens: 500
          }
        })
      )
    );
  }
}
```

## 注意事项

- ⚠️ 压缩可能会影响生成质量
- ⚠️ 不同压缩方法适用于不同场景
- ⚠️ 需要在速度和质量之间权衡
- ⚠️ 定期监控压缩效果
- ⚠️ 根据硬件配置调整策略
