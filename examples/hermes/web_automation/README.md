# 网页自动化案例

使用 Hermes 智能体自动化网页导航和交互任务。

## 场景描述

自动登录网站、导航到特定页面、点击按钮、等待页面加载等自动化操作。

## 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 1. 创建导航任务
const navigateTask = await invoke('hermes_create_task', {
  task_type: 'Navigation',
  description: 'Navigate to login page',
  priority: 1,
  metadata: {
    url: 'https://example.com/login',
    wait_for_selector: '#login-form'
  }
});

// 2. 执行导航任务
const navigateResult = await invoke('hermes_execute_task', {
  task_id: navigateTask
});

console.log('Navigation result:', navigateResult);

// 3. 创建表单填充任务
const formFillTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill login form',
  priority: 2,
  metadata: {
    form_selector: '#login-form',
    fields: [
      { selector: '#username', value: 'user@example.com' },
      { selector: '#password', value: 'secure_password' }
    ]
  }
});

// 4. 执行表单填充
const formFillResult = await invoke('hermes_execute_task', {
  task_id: formFillTask
});

console.log('Form fill result:', formFillResult);

// 5. 创建点击任务
const clickTask = await invoke('hermes_create_task', {
  task_type: 'Automation',
  description: 'Click submit button',
  priority: 3,
  metadata: {
    selector: '#submit-button',
    action: 'click'
  }
});

// 6. 执行点击
const clickResult = await invoke('hermes_execute_task', {
  task_id: clickTask
});

console.log('Click result:', clickResult);
```

## 策略链示例

```typescript
// 创建多步骤策略
const strategySteps = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com/login',
      wait_for_selector: '#login-form'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'FormFill',
    action: 'fill_form',
    parameters: {
      form_selector: '#login-form',
      fields: [
        { selector: '#username', value: 'user@example.com' },
        { selector: '#password', value: 'password123' }
      ]
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'Automation',
    action: 'click',
    parameters: {
      selector: '#submit-button'
    },
    condition: null
  },
  {
    id: 'step4',
    step_type: 'Navigation',
    action: 'wait_for_url',
    parameters: {
      url_pattern: '/dashboard'
    },
    condition: null
  }
];

// 创建策略
const strategyId = await invoke('hermes_create_strategy', {
  name: 'Login Strategy',
  description: 'Automated login workflow',
  steps: strategySteps
});

// 执行策略
const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});

console.log('Strategy execution results:', results);
```

## 后端配置

```json
{
  "enabled": true,
  "max_concurrent_tasks": 5,
  "task_timeout_secs": 30,
  "enable_learning": true,
  "enable_python_backend": false,
  "python_service_url": null
}
```

## 高级用法

### 条件执行

```typescript
const conditionalSteps = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: { url: 'https://example.com' },
    condition: 'url === "https://example.com"'
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'check_element',
    parameters: { selector: '#special-banner' },
    condition: 'element_exists'
  }
];
```

### 错误处理

```typescript
try {
  const taskId = await invoke('hermes_create_task', {
    task_type: 'Navigation',
    description: 'Navigate to page',
    priority: 1,
    metadata: { url: 'https://example.com' }
  });

  const result = await invoke('hermes_execute_task', { task_id: taskId });
  
  if (result.success) {
    console.log('Task completed successfully');
  } else {
    console.error('Task failed:', result.error);
    
    // 重试逻辑
    const retryTask = await invoke('hermes_create_task', { /* ... */ });
    await invoke('hermes_execute_task', { task_id: retryTask });
  }
} catch (error) {
  console.error('Task execution error:', error);
}
```

### 监控任务状态

```typescript
const taskId = await invoke('hermes_create_task', { /* ... */ });

// 轮询任务状态
const checkStatus = setInterval(async () => {
  const task = await invoke('hermes_get_task', { task_id: taskId });
  
  console.log('Task status:', task.status);
  
  if (task.status === 'Completed' || task.status === 'Failed' || task.status === 'Cancelled') {
    clearInterval(checkStatus);
    console.log('Task finished:', task);
  }
}, 1000);
```

## 性能优化

1. **批量操作** - 使用策略链而不是单个任务
2. **并行执行** - 设置合适的 `max_concurrent_tasks`
3. **超时设置** - 避免长时间等待
4. **缓存结果** - 存储常用页面的数据

## 注意事项

- ⚠️ 确保目标网站允许自动化访问
- ⚠️ 尊重网站的 robots.txt
- ⚠️ 避免过于频繁的请求
- ⚠️ 处理动态加载的内容
- ⚠️ 考虑网络延迟和超时
