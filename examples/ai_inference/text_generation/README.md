# 文本生成案例

使用 Allama 推理引擎进行文本生成、内容创作、代码生成等任务。

## 场景描述

自动生成文章、代码、邮件、报告等内容，提高创作效率。

## 前端调用示例

### 基础文本生成

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 生成文章
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '写一篇关于人工智能未来发展的文章，包括技术趋势、应用场景和挑战',
    max_tokens: 1000,
    temperature: 0.7,
    top_p: 0.9
  }
});

console.log('生成的文章:', response.text);
```

### 代码生成

```typescript
const codeResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '用 Python 实现一个快速排序算法，包含注释和测试用例',
    max_tokens: 800,
    temperature: 0.3,  // 较低温度以获得更确定性的代码
    stop: ['```', '"""']
  }
});

console.log('生成的代码:', codeResponse.text);
```

### 邮件生成

```typescript
const emailResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: `
主题：项目进度汇报
收件人：项目组
内容：本周完成了前端开发工作，下周计划开始后端开发，需要协调资源。
请生成一封专业的项目进度汇报邮件。
    `,
    max_tokens: 500,
    temperature: 0.5
  }
});

console.log('生成的邮件:', emailResponse.text);
```

### 创意写作

```typescript
const creativeResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '写一个关于时间旅行的短篇科幻故事，约500字',
    max_tokens: 800,
    temperature: 0.9,  // 较高温度以增加创意
    top_p: 0.95
  }
});

console.log('生成的故事:', creativeResponse.text);
```

## 高级用法

### 流式生成

```typescript
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '解释量子计算的基本原理',
    max_tokens: 1000,
    stream: true  // 启用流式生成
  }
});

// 处理流式输出
if (response.stream) {
  const reader = response.text.getReader();
  const decoder = new TextDecoder();
  
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    console.log('生成中:', chunk);
  }
}
```

### 条件生成

```typescript
const conditionalResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '根据以下条件生成产品描述：\n产品：智能手表\n特点：健康监测、消息通知、防水\n目标用户：运动爱好者',
    max_tokens: 400,
    temperature: 0.6
  }
});
```

### 批量生成

```typescript
const prompts = [
  '写一篇关于环保的文章',
  '写一首关于春天的诗',
  '写一个技术博客的开头'
];

const results = await Promise.all(
  prompts.map(prompt => 
    invoke('inference_generate', {
      request: {
        model: 'qwen3.6-35b-a3b',
        prompt,
        max_tokens: 500,
        temperature: 0.7
      }
    })
  )
);

results.forEach((result, index) => {
  console.log(`结果 ${index + 1}:`, result.text);
});
```

### 模板填充

```typescript
const templateResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: `
根据以下模板生成内容：

职位名称：[前端开发工程师]
公司名称：[科技初创公司]
工作地点：[北京]
薪资范围：[20K-35K]

请生成一份完整的职位描述。
    `,
    max_tokens: 800,
    temperature: 0.5
  }
});
```

## 参数调优

### Temperature（温度）

- **0.1-0.3**: 确定性输出，适合代码生成、技术文档
- **0.4-0.7**: 平衡输出，适合一般文本生成
- **0.8-1.0**: 创意输出，适合故事、创意写作

```typescript
// 确定性输出（代码生成）
const codeResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '实现二叉树遍历算法',
    temperature: 0.2
  }
});

// 创意输出（故事创作）
const storyResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '写一个奇幻故事',
    temperature: 0.9
  }
});
```

### Top P（Nucleus Sampling）

```typescript
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '解释机器学习',
    top_p: 0.9  // 只考虑累积概率前 90% 的词
  }
});
```

### Top K

```typescript
const response = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '生成技术文档',
    top_k: 40  // 只考虑概率最高的 40 个词
  }
});
```

## 错误处理

```typescript
try {
  const response = await invoke('inference_generate', {
    request: {
      model: 'qwen3.6-35b-a3b',
      prompt: '生成内容',
      max_tokens: 1000
    }
  });

  if (!response.success) {
    console.error('生成失败:', response.error);
    
    // 根据错误类型处理
    if (response.error.includes('not loaded')) {
      console.log('模型未加载，正在加载...');
      await invoke('inference_load_model', { model_name: 'qwen3.6-35b-a3b' });
      // 重试
    } else if (response.error.includes('timeout')) {
      console.log('超时，减少 token 数量');
      // 使用较小的 max_tokens 重试
    }
  }
} catch (error) {
  console.error('请求错误:', error);
}
```

## 性能优化

### 使用上下文压缩

```typescript
// 启用压缩以处理长文本
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'TokenQuantization',
      quantization_bits: 4,
      sliding_window: true,
      sliding_window_size: 512
    }
  }
});

// 生成长文档
const longDocResponse = await invoke('inference_generate', {
  request: {
    model: 'qwen3.6-35b-a3b',
    prompt: '写一份完整的技术白皮书，包括摘要、引言、技术架构、实现细节、结论',
    max_tokens: 2000
  }
});
```

### GPU 加速

```typescript
await invoke('inference_update_config', {
  config: {
    n_gpu_layers: 35,  // 将所有层加载到 GPU
    n_threads: 8      // 使用更多 CPU 线程
  }
});
```

## 与其他功能集成

### 与 Hermes 智能体集成

```typescript
// 创建内容生成任务
const taskId = await invoke('hermes_create_task', {
  task_type: 'Custom',
  description: '使用 AI 生成产品描述',
  priority: 1,
  metadata: {
    inference_engine: 'allama',
    model: 'qwen3.6-35b-a3b',
    prompt: '生成智能手表的产品描述'
  }
});

// 执行任务
const result = await invoke('hermes_execute_task', { task_id: taskId });
```

### 与 Python 微服务集成

```typescript
// Python 服务处理复杂的文本生成
const pythonResponse = await invoke('python_microservice_execute', {
  request: {
    code: `
import allama_client
client = allama_client.AllamaClient()

# 批量生成多个版本
prompts = ['版本1', '版本2', '版本3']
results = [client.generate(f'生成产品描述：{p}') for p in prompts]

# 选择最佳版本
best_result = max(results, key=lambda x: len(x))
print(best_result)
    `
  }
});
```

## 注意事项

- ⚠️ 生成的文本需要人工审核
- ⚠️ 长文本生成需要更多时间和资源
- ⚠️ 不同 temperature 会产生不同风格的结果
- ⚠️ 合理设置 max_tokens 以避免超时
- ⚠️ 使用 stop tokens 控制生成结束
