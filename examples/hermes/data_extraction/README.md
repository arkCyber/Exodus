# 数据提取案例

使用 Hermes 智能体从网页提取结构化数据。

## 场景描述

从电商网站提取商品信息、从新闻网站提取文章内容、从社交媒体提取用户数据等。

## 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 1. 创建数据提取任务
const extractionTask = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract product information',
  priority: 1,
  metadata: {
    url: 'https://example-shop.com/products',
    selectors: {
      products: '.product-item',
      name: '.product-name',
      price: '.product-price',
      rating: '.product-rating',
      image: '.product-image img'
    },
    pagination: {
      next_selector: '.next-page',
      max_pages: 5
    }
  }
});

// 2. 执行提取任务
const result = await invoke('hermes_execute_task', {
  task_id: extractionTask
});

console.log('Extracted data:', result);

// 3. 处理提取的数据
if (result.success) {
  const products = JSON.parse(result.output);
  console.log(`Extracted ${products.length} products`);
  
  // 保存到数据库或导出
  await saveProducts(products);
}
```

## 高级提取示例

### 提取新闻文章

```typescript
const newsTask = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract news articles',
  priority: 1,
  metadata: {
    url: 'https://example-news.com',
    selectors: {
      articles: '.article-card',
      title: '.article-title',
      author: '.article-author',
      date: '.article-date',
      content: '.article-content',
      tags: '.article-tags .tag'
    },
    transform: {
      date: 'parse_date',
      tags: 'split_by_comma'
    }
  }
});
```

### 提取表格数据

```typescript
const tableTask = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract table data',
  priority: 1,
  metadata: {
    url: 'https://example-data.com/table',
    selectors: {
      table: 'table.data-table',
      rows: 'tr.data-row',
      cells: 'td'
    },
    structure: {
      headers: true,
      row_mapping: ['id', 'name', 'value', 'date']
    }
  }
});
```

### 提取动态内容

```typescript
const dynamicTask = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract dynamic content',
  priority: 1,
  metadata: {
    url: 'https://example-dynamic.com',
    wait_for_selector: '.dynamic-content',
    selectors: {
      items: '.item',
      name: '.item-name',
      value: '.item-value'
    },
    javascript: true,
    timeout: 10000
  }
});
```

## 策略链示例

```typescript
// 多步骤数据提取策略
const strategySteps = [
  {
    id: 'step1',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example-shop.com'
    },
    condition: null
  },
  {
    id: 'step2',
    step_type: 'DataExtraction',
    action: 'extract_categories',
    parameters: {
      selector: '.category-item',
      field: 'text'
    },
    condition: null
  },
  {
    id: 'step3',
    step_type: 'Navigation',
    action: 'navigate',
    parameters: {
      url: 'https://example-shop.com/products'
    },
    condition: 'categories.length > 0'
  },
  {
    id: 'step4',
    step_type: 'DataExtraction',
    action: 'extract_products',
    parameters: {
      selectors: {
        products: '.product-item',
        name: '.product-name',
        price: '.product-price'
      }
    },
    condition: null
  },
  {
    id: 'step5',
    step_type: 'Custom',
    action: 'save_to_database',
    parameters: {
      table: 'products',
      data: '${previous_step_output}'
    },
    condition: null
  }
];

const strategyId = await invoke('hermes_create_strategy', {
  name: 'Product Data Extraction',
  description: 'Extract and save product data',
  steps: strategySteps
});

const results = await invoke('hermes_execute_strategy', {
  strategy_id: strategyId
});
```

## 数据处理

### 数据转换

```typescript
// 提取后进行数据转换
const task = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract and transform',
  priority: 1,
  metadata: {
    url: 'https://example.com/data',
    selectors: {
      items: '.data-item'
    },
    transform: {
      price: 'float',
      quantity: 'int',
      date: 'iso_date'
    }
  }
});
```

### 数据过滤

```typescript
// 提取后过滤数据
const task = await invoke('hermes_create_task', {
  task_type: 'DataExtraction',
  description: 'Extract and filter',
  priority: 1,
  metadata: {
    url: 'https://example.com/data',
    selectors: {
      items: '.data-item'
    },
    filter: {
      min_price: 10,
      max_price: 100,
      in_stock: true
    }
  }
});
```

### 数据聚合

```typescript
// 提取多个页面并聚合
const tasks = [];
const urls = [
  'https://example.com/page1',
  'https://example.com/page2',
  'https://example.com/page3'
];

for (const url of urls) {
  const task = await invoke('hermes_create_task', {
    task_type: 'DataExtraction',
    description: `Extract from ${url}`,
    priority: 1,
    metadata: {
      url,
      selectors: {
        items: '.data-item'
      }
    }
  });
  tasks.push(task);
}

// 并行执行
const results = await Promise.all(
  tasks.map(taskId => invoke('hermes_execute_task', { task_id: taskId }))
);

// 聚合结果
const allData = results.flatMap(r => JSON.parse(r.output));
```

## 错误处理

```typescript
try {
  const taskId = await invoke('hermes_create_task', {
    task_type: 'DataExtraction',
    description: 'Extract data',
    priority: 1,
    metadata: {
      url: 'https://example.com',
      selectors: { items: '.item' }
    }
  });

  const result = await invoke('hermes_execute_task', { task_id: taskId });
  
  if (!result.success) {
    console.error('Extraction failed:', result.error);
    
    // 重试逻辑
    if (result.error.includes('timeout')) {
      console.log('Retrying with longer timeout...');
      // 重新创建任务并执行
    }
  }
} catch (error) {
  console.error('Task error:', error);
}
```

## 性能优化

1. **并行提取** - 同时提取多个页面
2. **增量提取** - 只提取新数据
3. **缓存策略** - 缓存已提取的数据
4. **批量处理** - 批量保存到数据库

## 注意事项

- ⚠️ 尊重网站的 robots.txt
- ⚠️ 避免过于频繁的请求
- ⚠️ 处理动态加载的内容
- ⚠️ 验证数据完整性
- ⚠️ 考虑数据隐私和版权
