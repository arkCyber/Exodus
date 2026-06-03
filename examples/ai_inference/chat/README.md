# 聊天对话案例

使用 Allama 推理引擎进行多轮对话、角色扮演、智能助手等任务。

## 场景描述

实现智能客服、虚拟助手、角色扮演机器人等对话系统。

## 前端调用示例

### 基础对话

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 单轮对话
const response = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: '你好，请介绍一下你自己' }
    ],
    max_tokens: 300,
    temperature: 0.7
  }
});

console.log('AI 回复:', response.text);
```

### 多轮对话

```typescript
// 对话历史
const conversationHistory = [
  { role: 'user', content: '我想学习 Python 编程' },
  { role: 'assistant', content: '很好的选择！Python 是一门功能强大的编程语言。你想从哪里开始学习？' },
  { role: 'user', content: '从基础语法开始' }
];

const response = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: conversationHistory,
    max_tokens: 500,
    temperature: 0.7
  }
});

// 添加到历史
conversationHistory.push({ role: 'assistant', content: response.text });
```

### 角色扮演

```typescript
// 技术顾问角色
const techAdvisorResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { 
        role: 'system', 
        content: '你是一位经验丰富的技术顾问，擅长架构设计和性能优化。请以专业、客观的方式回答问题。' 
      },
      { role: 'user', content: '如何优化数据库查询性能？' }
    ],
    max_tokens: 800,
    temperature: 0.5
  }
});

// 客服角色
const customerServiceResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { 
        role: 'system', 
        content: '你是一位友好的客服代表，耐心解答用户问题，提供帮助。' 
      },
      { role: 'user', content: '我的订单一直显示处理中，已经3天了' }
    ],
    max_tokens: 300,
    temperature: 0.6
  }
});
```

### 智能助手

```typescript
// 个人助理
const assistantResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: '帮我安排明天的日程' },
      { role: 'assistant', content: '好的，请告诉我明天有哪些事情需要安排？' },
      { role: 'user', content: '上午10点开会，下午2点面试候选人，晚上7点健身' }
    ],
    max_tokens: 400,
    temperature: 0.7
  }
});
```

## 高级用法

### 流式对话

```typescript
const response = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: '详细解释机器学习的工作原理' }
    ],
    max_tokens: 1000,
    stream: true
  }
});

// 实时显示生成内容
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

### 上下文管理

```typescript
class ChatSession {
  private history: Array<{ role: string; content: string }> = [];
  private maxHistory: number = 10;

  async sendMessage(userMessage: string): Promise<string> {
    // 添加用户消息
    this.history.push({ role: 'user', content: userMessage });

    // 限制历史长度
    if (this.history.length > this.maxHistory * 2) {
      this.history = this.history.slice(-this.maxHistory * 2);
    }

    // 调用推理引擎
    const response = await invoke('inference_chat', {
      request: {
        model: 'qwen3.6-35b-a3b',
        messages: this.history,
        max_tokens: 500,
        temperature: 0.7
      }
    });

    // 添加助手回复
    this.history.push({ role: 'assistant', content: response.text });
    
    return response.text;
  }

  getHistory() {
    return this.history;
  }

  clearHistory() {
    this.history = [];
  }
}

// 使用
const session = new ChatSession();
const reply1 = await session.sendMessage('你好');
const reply2 = await session.sendMessage('Python 的主要用途是什么？');
```

### 知识库对话

```typescript
// 结合知识库的对话
const knowledgeBaseResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { 
        role: 'system', 
        content: `你是公司的知识库助手。请基于以下知识库回答问题：
        
知识库：
- 公司成立于2020年
- 主要产品：AI推理引擎、微服务框架
- 员工数：200人
- 总部：北京
        `
      },
      { role: 'user', content: '公司什么时候成立的？' }
    ],
    max_tokens: 200,
    temperature: 0.3  // 较低温度以确保准确性
  }
});
```

### 多语言对话

```typescript
// 英文对话
const englishResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: 'Explain quantum computing in simple terms' }
    ],
    max_tokens: 500,
    temperature: 0.7
  }
});

// 中文对话
const chineseResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: '用简单的语言解释量子计算' }
    ],
    max_tokens: 500,
    temperature: 0.7
  }
});
```

### 对话模板

```typescript
// 产品咨询模板
const productConsultation = async (productName: string, question: string) => {
  const response = await invoke('inference_chat', {
    request: {
      model: 'qwen3.6-35b-a3b',
      messages: [
        { 
          role: 'system', 
          content: `你是产品咨询助手，专门解答关于 ${productName} 的问题。请提供准确、有用的信息。` 
        },
        { role: 'user', content: question }
      ],
      max_tokens: 400,
      temperature: 0.5
    }
  });
  
  return response.text;
};

// 使用
const answer = await productConsultation('智能手表', '这款手表的续航时间是多少？');
```

## 错误处理

```typescript
try {
  const response = await invoke('inference_chat', {
    request: {
      model: 'qwen3.6-35b-a3b',
      messages: [
        { role: 'user', content: '你好' }
      ],
      max_tokens: 500
    }
  });

  if (!response.success) {
    console.error('对话失败:', response.error);
    
    // 处理常见错误
    if (response.error.includes('not loaded')) {
      console.log('模型未加载');
      await invoke('inference_load_model', { model_name: 'qwen3.6-35b-a3b' });
    } else if (response.error.includes('context')) {
      console.log('上下文过长，正在压缩');
      // 减少历史消息数量
    }
  }
} catch (error) {
  console.error('请求错误:', error);
}
```

## 性能优化

### 使用上下文压缩

```typescript
// 启用压缩以支持更长对话
await invoke('inference_update_config', {
  config: {
    context_compression: {
      enabled: true,
      compression_method: 'StreamingLLM',
      sliding_window: true,
      sliding_window_size: 1024
    }
  }
});

// 长对话
const longConversation = [
  { role: 'user', content: '开始长对话...' },
  // ... 更多消息
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

### 批量对话

```typescript
// 并行处理多个对话
const conversations = [
  [{ role: 'user', content: '问题1' }],
  [{ role: 'user', content: '问题2' }],
  [{ role: 'user', content: '问题3' }]
];

const responses = await Promise.all(
  conversations.map(messages =>
    invoke('inference_chat', {
      request: {
        model: 'qwen3.6-35b-a3b',
        messages,
        max_tokens: 300
      }
    })
  )
);
```

## 与其他功能集成

### 与 Hermes 智能体集成

```typescript
// 创建对话任务
const taskId = await invoke('hermes_create_task', {
  task_type: 'Custom',
  description: 'AI 客服对话',
  priority: 1,
  metadata: {
    inference_engine: 'allama',
    model: 'qwen3.6-35b-a3b',
    role: 'customer_service',
    user_message: '我想退货'
  }
});

const result = await invoke('hermes_execute_task', { task_id: taskId });
```

### 与网页自动化集成

```typescript
// 结合网页自动化的对话
const pageContent = await invoke('hermes_execute_task', {
  task_id: someTaskId
});

const contextResponse = await invoke('inference_chat', {
  request: {
    model: 'qwen3.6-35b-a3b',
    messages: [
      { role: 'user', content: `基于以下网页内容回答问题：\n${pageContent}\n\n问题：这个网站的主要功能是什么？` }
    ],
    max_tokens: 400
  }
});
```

## 注意事项

- ⚠️ 对话历史需要合理管理，避免过长
- ⚠️ 不同角色设定会产生不同风格
- ⚠️ 敏感信息需要过滤
- ⚠️ 对话需要适当的错误处理
- ⚠️ 考虑使用上下文压缩支持长对话
