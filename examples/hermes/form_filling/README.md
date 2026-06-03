# 表单填充案例

使用 Hermes 智能体自动填充和提交网页表单。

## 场景描述

自动填写注册表单、登录表单、调查问卷、订单表单等，提高工作效率。

## 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 1. 创建表单填充任务
const formFillTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill registration form',
  priority: 1,
  metadata: {
    url: 'https://example.com/register',
    form_selector: '#registration-form',
    fields: [
      { selector: '#username', value: 'user123' },
      { selector: '#email', value: 'user@example.com' },
      { selector: '#password', value: 'secure_password' },
      { selector: '#confirm-password', value: 'secure_password' },
      { selector: '#full-name', value: 'John Doe' },
      { selector: '#phone', value: '+1234567890' },
      { selector: '#address', value: '123 Main St, City' }
    ],
    checkboxes: [
      { selector: '#terms-checkbox', checked: true },
      { selector: '#newsletter-checkbox', checked: false }
    ],
    dropdowns: [
      { selector: '#country', value: 'US' },
      { selector: '#language', value: 'en' }
    ]
  }
});

// 2. 执行表单填充
const result = await invoke('hermes_execute_task', {
  task_id: formFillTask
});

console.log('Form fill result:', result);
```

## 高级表单填充

### 动态表单

```typescript
const dynamicFormTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill dynamic form',
  priority: 1,
  metadata: {
    url: 'https://example.com/dynamic-form',
    wait_for_selector: '#dynamic-form',
    form_selector: '#dynamic-form',
    fields: [
      { selector: 'input[name="username"]', value: 'user123' },
      { selector: 'input[name="email"]', value: 'user@example.com' }
    ],
    dynamic_fields: {
      pattern: 'input[data-dynamic="true"]',
      source: 'api_endpoint'
    }
  }
});
```

### 多步骤表单

```typescript
const multiStepTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill multi-step form',
  priority: 1,
  metadata: {
    url: 'https://example.com/multi-step-form',
    steps: [
      {
        step: 1,
        form_selector: '#step1-form',
        fields: [
          { selector: '#first-name', value: 'John' },
          { selector: '#last-name', value: 'Doe' }
        ],
        next_button: '#next-step1'
      },
      {
        step: 2,
        form_selector: '#step2-form',
        fields: [
          { selector: '#email', value: 'john@example.com' },
          { selector: '#phone', value: '+1234567890' }
        ],
        next_button: '#next-step2'
      },
      {
        step: 3,
        form_selector: '#step3-form',
        fields: [
          { selector: '#address', value: '123 Main St' },
          { selector: '#city', value: 'New York' }
        ],
        submit_button: '#submit-form'
      }
    ]
  }
});
```

### 文件上传表单

```typescript
const uploadTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill file upload form',
  priority: 1,
  metadata: {
    url: 'https://example.com/upload',
    form_selector: '#upload-form',
    fields: [
      { selector: '#title', value: 'Document Title' },
      { selector: '#description', value: 'Document description' }
    ],
    file_inputs: [
      { 
        selector: '#file-upload',
        file_path: '/path/to/document.pdf',
        file_name: 'document.pdf'
      }
    ],
    submit_button: '#upload-button'
  }
});
```

## 策略链示例

```typescript
// 完整的注册流程策略
const registrationSteps = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com/register'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'FormFill',
    action: 'fill_personal_info',
    parameters: {
      form_selector: '#personal-info-form',
      fields: [
        { selector: '#first-name', value: 'John' },
        { selector: '#last-name', value: 'Doe' },
        { selector: '#email', value: 'john@example.com' }
      ]
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'Automation',
    action: 'click',
    parameters: {
      selector: '#next-button'
    },
    condition: null
  },
  {
    id: 'step4',
    step_type: 'FormFill',
    action: 'fill_account_info',
    parameters: {
      form_selector: '#account-info-form',
      fields: [
        { selector: '#username', value: 'johndoe123' },
        { selector: '#password', value: 'secure_password' },
        { selector: '#confirm-password', value: 'secure_password' }
      ]
    },
    condition: null
  },
  {
    id: 'step5',
    step_type: 'Automation',
    action: 'click',
    parameters: {
      selector: '#submit-button'
    },
    condition: null
  },
  {
    id: 'step6',
    step_type: 'Navigation',
    action: 'wait_for_url',
    parameters: {
      url_pattern: '/success'
    },
    condition: null
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Registration Strategy',
  description: 'Automated registration workflow',
  steps: registrationSteps
});

const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});
```

## 表单验证

### 验证字段

```typescript
const validationTask = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill with validation',
  priority: 1,
  metadata: {
    url: 'https://example.com/form',
    form_selector: '#my-form',
    fields: [
      { 
        selector: '#email',
        value: 'user@example.com',
        validation: {
          pattern: '^[^@]+@[^@]+\.[^@]+$',
          error_message: 'Invalid email format'
        }
      },
      { 
        selector: '#phone',
        value: '+1234567890',
        validation: {
          pattern: '^\+?[0-9]{10,15}$',
          error_message: 'Invalid phone number'
        }
      }
    ],
    validate_before_submit: true
  }
});
```

### 错误处理

```typescript
try {
  const taskId = await invoke('hermes_create_task', {
    task_type: 'FormFill',
    description: 'Fill form',
    priority: 1,
    metadata: {
      url: 'https://example.com/form',
      form_selector: '#my-form',
      fields: [
        { selector: '#username', value: 'user123' },
        { selector: '#email', value: 'user@example.com' }
      ]
    }
  });

  const result = await invoke('hermes_execute_task', { task_id: taskId });
  
  if (!result.success) {
    console.error('Form fill failed:', result.error);
    
    // 检查是否是验证错误
    if (result.error.includes('validation')) {
      console.log('Validation error, please check field values');
    } else if (result.error.includes('timeout')) {
      console.log('Form load timeout, retrying...');
      // 重试逻辑
    }
  }
} catch (error) {
  console.error('Task error:', error);
}
```

## 数据源集成

### 从 CSV 填充

```typescript
import Papa from 'papaparse';

// 读取 CSV 文件
const csvData = await fetch('/data/users.csv').then(r => r.text());
const parsed = Papa.parse(csvData, { header: true });

// 批量填充表单
for (const row of parsed.data) {
  const task = await invoke('hermes_create_task', {
    task_type: 'FormFill',
    description: `Fill form for ${row.username}`,
    priority: 1,
    metadata: {
      url: 'https://example.com/form',
      form_selector: '#my-form',
      fields: [
        { selector: '#username', value: row.username },
        { selector: '#email', value: row.email },
        { selector: '#phone', value: row.phone }
      ]
    }
  });

  await invoke('hermes_execute_task', { task_id: task });
  
  // 避免过于频繁的请求
  await new Promise(resolve => setTimeout(resolve, 1000));
}
```

### 从数据库填充

```typescript
// 从数据库获取数据
const users = await db.collection('users').find({}).toArray();

// 批量填充
for (const user of users) {
  const task = await invoke('hermes_create_task', {
    task_type: 'FormFill',
    description: `Fill form for ${user.name}`,
    priority: 1,
    metadata: {
      url: 'https://example.com/form',
      form_selector: '#my-form',
      fields: [
        { selector: '#name', value: user.name },
        { selector: '#email', value: user.email },
        { selector: '#address', value: user.address }
      ]
    }
  });

  await invoke('hermes_execute_task', { task_id: task });
}
```

## 安全考虑

### 密码处理

```typescript
// 使用加密存储密码
const encryptedPassword = encryptPassword('secure_password');

const task = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill login form',
  priority: 1,
  metadata: {
    url: 'https://example.com/login',
    form_selector: '#login-form',
    fields: [
      { selector: '#username', value: 'user123' },
      { selector: '#password', value: encryptedPassword, encrypted: true }
    ]
  }
});
```

### 敏感数据

```typescript
// 避免在日志中记录敏感数据
const task = await invoke('hermes_create_task', {
  task_type: 'FormFill',
  description: 'Fill form',
  priority: 1,
  metadata: {
    url: 'https://example.com/form',
    form_selector: '#my-form',
    fields: [
      { selector: '#username', value: 'user123', sensitive: false },
      { selector: '#password', value: '***', sensitive: true },
      { selector: '#ssn', value: '***', sensitive: true }
    ],
    log_sensitive: false
  }
});
```

## 性能优化

1. **批量处理** - 使用策略链处理多个表单
2. **并行执行** - 同时填充多个独立表单
3. **缓存表单结构** - 避免重复解析
4. **智能等待** - 只在必要时等待元素加载

## 注意事项

- ⚠️ 确保表单字段选择器准确
- ⚠️ 处理动态加载的表单字段
- ⚠️ 验证表单提交成功
- ⚠️ 保护敏感数据（密码、信用卡号等）
- ⚠️ 遵守网站的使用条款
- ⚠️ 避免过于频繁的表单提交
