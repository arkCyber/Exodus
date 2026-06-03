# 嵌入生成案例

使用 Allama 推理引擎生成文本嵌入，用于语义搜索、相似度计算、聚类等任务。

## 场景描述

将文本转换为向量表示，用于语义搜索、文档相似度、推荐系统等。

## 前端调用示例

### 基础嵌入生成

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// 生成单个文本的嵌入
const response = await invoke('inference_embed', {
  request: {
    model: 'qwen3.6-35b-a3b',
    text: '人工智能是计算机科学的一个分支'
  }
});

console.log('嵌入维度:', response.dimensions);
console.log('嵌入向量:', response.embedding);
```

### 批量嵌入生成

```typescript
const texts = [
  '人工智能是计算机科学的一个分支',
  '机器学习是人工智能的一个子领域',
  '深度学习是机器学习的一种方法'
];

const embeddings = await Promise.all(
  texts.map(text =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text
      }
    })
  )
);

embeddings.forEach((result, index) => {
  console.log(`文本 ${index + 1} 的嵌入:`, result.embedding);
});
```

## 高级用法

### 语义搜索

```typescript
// 生成查询和文档的嵌入
const queryEmbedding = await invoke('inference_embed', {
  request: {
    model: 'qwen3.6-35b-a3b',
    text: '如何学习机器学习'
  }
});

const documents = [
  '机器学习入门教程',
  'Python 编程基础',
  '深度学习实战',
  '数据科学导论'
];

const docEmbeddings = await Promise.all(
  documents.map(doc =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: doc
      }
    })
  )
);

// 计算余弦相似度
function cosineSimilarity(a: number[], b: number[]): number {
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  
  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }
  
  return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
}

// 计算相似度并排序
const similarities = docEmbeddings.map((doc, index) => ({
  document: documents[index],
  similarity: cosineSimilarity(
    queryEmbedding.embedding!,
    doc.embedding!
  )
}));

similarities.sort((a, b) => b.similarity - a.similarity);

console.log('最相关的文档:', similarities[0]);
```

### 文档聚类

```typescript
const documents = [
  '人工智能技术',
  '机器学习算法',
  '深度学习框架',
  '自然语言处理',
  '计算机视觉',
  'Web 开发',
  '数据库管理',
  '系统架构'
];

// 生成所有文档的嵌入
const embeddings = await Promise.all(
  documents.map(doc =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: doc
      }
    })
  )
);

// 简单的 K-means 聚类
function kmeans(embeddings: number[][], k: number): number[][] {
  // 简化的 K-means 实现
  const centroids = embeddings.slice(0, k);
  const assignments = embeddings.map(() => 0);
  
  for (let iter = 0; iter < 10; iter++) {
    // 分配到最近的中心
    for (let i = 0; i < embeddings.length; i++) {
      let minDist = Infinity;
      for (let j = 0; j < k; j++) {
        const dist = cosineSimilarity(embeddings[i], centroids[j]);
        if (dist < minDist) {
          minDist = dist;
          assignments[i] = j;
        }
      }
    }
    
    // 更新中心（简化）
    for (let j = 0; j < k; j++) {
      const clusterPoints = embeddings.filter((_, i) => assignments[i] === j);
      if (clusterPoints.length > 0) {
        centroids[j] = clusterPoints[0]; // 简化：使用第一个点
      }
    }
  }
  
  return assignments;
}

const clusters = kmeans(embeddings.map(e => e.embedding!), 3);

console.log('聚类结果:', clusters);
```

### 推荐系统

```typescript
// 用户历史
const userHistory = [
  'Python 编程教程',
  '机器学习入门',
  '数据可视化'
];

// 生成用户历史嵌入
const historyEmbeddings = await Promise.all(
  userHistory.map(item =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: item
      }
    })
  )
);

// 计算用户兴趣向量（平均）
const userVector = historyEmbeddings.reduce((acc, emb) => {
  const vec = emb.embedding!;
  return acc.map((v, i) => v + vec[i]);
}, new Array(768).fill(0)).map(v => v / historyEmbeddings.length);

// 候选推荐
const candidates = [
  '深度学习实战',
  'Web 开发指南',
  '数据库优化',
  '自然语言处理',
  '计算机视觉应用'
];

const candidateEmbeddings = await Promise.all(
  candidates.map(cand =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: cand
      }
    })
  )
);

// 计算推荐分数
const recommendations = candidateEmbeddings.map((cand, index) => ({
  candidate: candidates[index],
  score: cosineSimilarity(userVector, cand.embedding!)
}));

recommendations.sort((a, b) => b.score - a.score);

console.log('推荐结果:', recommendations);
```

### 文档去重

```typescript
const documents = [
  '人工智能是计算机科学的一个重要分支',
  'AI 是计算机科学的重要分支',
  '机器学习是人工智能的一个子领域',
  '深度学习是机器学习的一种方法',
  'Python 是一种编程语言'
];

// 生成嵌入
const embeddings = await Promise.all(
  documents.map(doc =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: doc
      }
    })
  )
);

// 计算相似度矩阵
const similarityMatrix: number[][] = [];
for (let i = 0; i < embeddings.length; i++) {
  similarityMatrix[i] = [];
  for (let j = 0; j < embeddings.length; j++) {
    similarityMatrix[i][j] = cosineSimilarity(
      embeddings[i].embedding!,
      embeddings[j].embedding!
    );
  }
}

// 去重（相似度 > 0.95 的视为重复）
const uniqueDocs: string[] = [];
const seenIndices = new Set<number>();

for (let i = 0; i < documents.length; i++) {
  if (seenIndices.has(i)) continue;
  
  uniqueDocs.push(documents[i]);
  seenIndices.add(i);
  
  // 标记相似文档
  for (let j = i + 1; j < documents.length; j++) {
    if (similarityMatrix[i][j] > 0.95) {
      seenIndices.add(j);
    }
  }
}

console.log('去重后的文档:', uniqueDocs);
```

## 性能优化

### 批量处理

```typescript
// 分批处理大量文档
async function batchEmbeddings(texts: string[], batchSize: number = 10) {
  const results = [];
  
  for (let i = 0; i < texts.length; i += batchSize) {
    const batch = texts.slice(i, i + batchSize);
    const batchResults = await Promise.all(
      batch.map(text =>
        invoke('inference_embed', {
          request: {
            model: 'qwen3.6-35b-a3b',
            text
          }
        })
      )
    );
    
    results.push(...batchResults);
    
    // 避免过载
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  
  return results;
}

const largeTexts = Array.from({ length: 100 }, (_, i) => `文档 ${i}`);
const embeddings = await batchEmbeddings(largeTexts);
```

### 缓存嵌入

```typescript
class EmbeddingCache {
  private cache: Map<string, number[]> = new Map();
  
  async get(text: string): Promise<number[]> {
    if (this.cache.has(text)) {
      return this.cache.get(text)!;
    }
    
    const response = await invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text
      }
    });
    
    this.cache.set(text, response.embedding!);
    return response.embedding!;
  }
  
  clear() {
    this.cache.clear();
  }
}

const cache = new EmbeddingCache();
const embedding1 = await cache.get('重复的文本');
const embedding2 = await cache.get('重复的文本'); // 从缓存获取
```

## 与其他功能集成

### 与数据提取集成

```typescript
// 提取网页内容并生成嵌入
const extractedData = await invoke('hermes_execute_task', {
  task_id: extractionTaskId
});

const embeddings = await Promise.all(
  extractedData.items.map((item: any) =>
    invoke('inference_embed', {
      request: {
        model: 'qwen3.6-35b-a3b',
        text: item.content
      }
    })
  )
);

// 基于嵌入进行分类或聚类
```

### 与搜索集成

```typescript
// 结合传统搜索和语义搜索
const traditionalResults = await invoke('search_documents', {
  query: '机器学习',
  method: 'keyword'
});

const semanticResults = await invoke('inference_embed', {
  request: {
    model: 'qwen3.6-35b-a3b',
    text: '机器学习'
  }
});

// 混合排序
```

## 注意事项

- ⚠️ 嵌入维度较大（768+），需要足够的内存
- ⚠️ 批量生成时注意控制并发
- ⚠️ 嵌入质量取决于模型
- ⚠️ 相似度计算需要选择合适的度量
- ⚠️ 考虑使用缓存减少重复计算
