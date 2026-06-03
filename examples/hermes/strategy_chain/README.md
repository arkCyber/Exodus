# 策略链案例

使用 Hermes 智能体的策略链功能编排复杂的多步骤任务。

## 场景描述

将多个任务组合成策略链，实现复杂的自动化工作流，如完整的电商购物流程、数据采集和分析流程等。

## 基础策略链示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 创建简单的两步策略
const strategySteps = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'extract_title',
    parameters: {
      selector: 'h1'
    },
    condition: null
  }
];

// 创建策略
const strategyId = await invoke('hermes_create_strategy', {
  name: 'Simple Strategy',
  description: 'Navigate and extract title',
  steps: strategySteps
});

// 执行策略
const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});

console.log('Strategy results:', results);
```

## 电商购物流程

```typescript
const shoppingStrategy = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://shop.example.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'search_product',
    parameters: {
      search_selector: '#search-input',
      search_term: 'laptop',
      submit_selector: '#search-button'
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'Navigation',
    action: 'wait_for_selector',
    parameters: {
      selector: '.product-list'
    },
    condition: null
  },
  {
    id: 'step4',
    step_type: 'DataExtraction',
    action: 'extract_products',
    parameters: {
      selectors: {
        products: '.product-item',
        name: '.product-name',
        price: '.product-price',
        rating: '.product-rating'
      }
    },
    condition: null
  },
  {
    id: 'step5',
    step_type: 'Automation',
    action: 'select_product',
    parameters: {
      selector: '.product-item:first-child .product-link'
    },
    condition: 'products.length > 0'
  },
  {
    id: 'step6',
    step_type: 'Navigation',
    action: 'wait_for_url',
    parameters: {
      url_pattern: '/product/'
    },
    condition: null
  },
  {
    id: 'step7',
    step_type: 'FormFill',
    action: 'add_to_cart',
    parameters: {
      quantity_selector: '#quantity',
      quantity: 1,
      add_to_cart_selector: '#add-to-cart'
    },
    condition: null
  },
  {
    id: 'step8',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://shop.example.com/cart'
    },
    condition: null
  },
  {
    id: 'step9',
    step_type: 'FormFill',
    action: 'checkout',
    parameters: {
      form_selector: '#checkout-form',
      fields: [
        { selector: '#name', value: 'John Doe' },
        { selector: '#email', value: 'john@example.com' },
        { selector: '#address', value: '123 Main St' }
      ]
    },
    condition: null
  },
  {
    id: 'step10',
    step_type: 'Automation',
    action: 'submit_order',
    parameters: {
      selector: '#submit-order'
    },
    condition: null
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'E-commerce Shopping Strategy',
  description: 'Complete shopping workflow from search to checkout',
  steps: shoppingStrategy
});

const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});
```

## 数据采集和分析流程

```typescript
const dataPipelineStrategy = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://news.example.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'extract_articles',
    parameters: {
      selectors: {
        articles: '.article-card',
        title: '.article-title',
        url: '.article-link',
        date: '.article-date',
        category: '.article-category'
      },
      pagination: {
        next_selector: '.next-page',
        max_pages: 5
      }
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'Custom',
    action: 'process_data',
    parameters: {
      transform: {
        date: 'parse_date',
        category: 'normalize'
      },
      filter: {
        min_date: '2024-01-01',
        categories: ['technology', 'science']
      }
    },
    condition: 'articles.length > 0'
  },
  {
    id: 'step4',
    step_type: 'Custom',
    action: 'save_to_database',
    parameters: {
      collection: 'articles',
      data: '${previous_step_output}'
    },
    condition: null
  },
  {
    id: 'step5',
    step_type: 'Custom',
    action: 'generate_report',
    parameters: {
      format: 'json',
      output: '/reports/news_summary.json'
    },
    condition: null
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Data Pipeline Strategy',
  description: 'Extract, process, and save news articles',
  steps: dataPipelineStrategy
});

const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});
```

## 条件执行策略

```typescript
const conditionalStrategy = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'check_login_status',
    parameters: {
      selector: '.user-profile'
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'FormFill',
    action: 'login',
    parameters: {
      form_selector: '#login-form',
      fields: [
        { selector: '#username', value: 'user123' },
        { selector: '#password', value: 'password' }
      ]
    },
    condition: '!logged_in'
  },
  {
    id: 'step4',
    step_type: 'Navigation',
    action: 'navigate_to_dashboard',
    parameters: {
      url: 'https://example.com/dashboard'
    },
    condition: 'logged_in || step3_success'
  },
  {
    id: 'step5',
    step_type: 'DataExtraction',
    action: 'extract_dashboard_data',
    parameters: {
      selectors: {
        stats: '.stat-item',
        charts: '.chart-container'
      }
    },
    condition: null
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Conditional Strategy',
  description: 'Execute steps based on conditions',
  steps: conditionalStrategy
});
```

## 错误恢复策略

```typescript
const robustStrategy = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com',
      timeout: 10000
    },
    condition: null,
    on_failure: 'retry',
    max_retries: 3
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'extract_data',
    parameters: {
      selector: '.data-container'
    },
    condition: null,
    on_failure: 'skip',
    continue_on_failure: true
  },
  {
    id: 'step3',
    step_type: 'Custom',
    action: 'fallback_extraction',
    parameters: {
      selector: '.alternative-container'
    },
    condition: 'step2_failed',
    on_failure: 'abort'
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Robust Strategy',
  description: 'Strategy with error handling',
  steps: robustStrategy
});
```

## 并行执行策略

```typescript
const parallelStrategy = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'extract_section_a',
    parameters: {
      selector: '#section-a'
    },
    condition: null,
    parallel: true
  },
  {
    id: 'step3',
    step_type: 'DataExtraction',
    action: 'extract_section_b',
    parameters: {
      selector: '#section-b'
    },
    condition: null,
    parallel: true
  },
  {
    id: 'step4',
    step_type: 'DataExtraction',
    action: 'extract_section_c',
    parameters: {
      selector: '#section-c'
    },
    condition: null,
    parallel: true
  },
  {
    id: 'step5',
    step_type: 'Custom',
    action: 'merge_results',
    parameters: {
      sources: ['step2', 'step3', 'step4']
    },
    condition: 'step2_success && step3_success && step4_success'
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Parallel Strategy',
  description: 'Execute steps in parallel',
  steps: parallelStrategy
});
```

## 策略管理

### 查看所有策略

```typescript
// 获取 Hermes 统计信息
const stats = await invoke('hermes_get_stats');
console.log(`Total strategies: ${stats.total_strategies}`);
console.log(`Active strategies: ${stats.active_strategies}`);
```

### 启用/禁用策略

```typescript
// 获取策略配置
const config = await invoke('hermes_get_config');

// 更新配置
await invoke('hermes_update_config', {
  config: {
    ...config,
    enable_learning: true,
    max_concurrent_tasks: 10
  }
});
```

### 监控策略执行

```typescript
const strategyId = await invoke('hermes_create_strategy', {
  name: 'Monitored Strategy',
  description: 'Strategy with monitoring',
  steps: strategySteps
});

// 执行策略
const executionPromise = invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});

// 监控进度
const monitorInterval = setInterval(async () => {
  const stats = await invoke('hermes_get_stats');
  console.log(`Active tasks: ${stats.active_tasks}`);
  console.log(`Completed tasks: ${stats.completed_tasks}`);
}, 1000);

// 等待执行完成
const results = await executionPromise;
clearInterval(monitorInterval);
console.log('Strategy completed:', results);
```

## 策略模板

### 可重用模板

```typescript
// 创建导航模板
const navigationTemplate = (url: string, waitForSelector?: string) => ({
  id: 'navigate',
  step_type: 'Navigation',
  action: 'navigate',
  parameters: {
    url,
    wait_for_selector: waitForSelector
  },
  condition: null
});

// 创建数据提取模板
const extractionTemplate = (selectors: any) => ({
  id: 'extract',
  step_type: 'DataExtraction',
  action: 'extract',
  parameters: { selectors },
  condition: null
});

// 使用模板构建策略
const strategySteps = [
  navigationTemplate('https://example.com', '#main-content'),
  extractionTemplate({
    title: 'h1',
    content: '.content'
  })
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Template Strategy',
  description: 'Strategy built from templates',
  steps: strategySteps
});
```

## 最佳实践

1. **模块化设计** - 将复杂策略分解为可重用的子策略
2. **错误处理** - 为每个步骤定义失败处理逻辑
3. **条件执行** - 使用条件避免不必要的步骤
4. **并行执行** - 对独立步骤使用并行执行提高效率
5. **监控和日志** - 记录策略执行过程便于调试
6. **测试策略** - 在生产环境前充分测试策略

## 注意事项

- ⚠️ 策略步骤之间的依赖关系要清晰
- ⚠️ 设置合理的超时时间
- ⚠️ 处理网络异常和页面加载失败
- ⚠️ 避免策略过于复杂导致难以维护
- ⚠️ 定期审查和更新策略
- ⚠️ 保护敏感数据（密码、API密钥等）
