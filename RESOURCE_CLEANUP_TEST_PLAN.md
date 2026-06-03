# 资源清理测试计划 (Resource Cleanup Test Plan)

## 测试目标
验证所有资源在组件卸载时被正确清理，防止内存泄漏。

---

## 测试环境准备

### 1. 开启 Chrome DevTools
```bash
# 在 Tauri 应用中打开开发者工具
# macOS: Cmd + Option + I
# Windows/Linux: Ctrl + Shift + I
```

### 2. 启用性能监控
1. 打开 DevTools → Performance Monitor
2. 打开 DevTools → Memory → Heap Snapshot

---

## 测试用例

### Test 1: 事件监听器清理测试
**目标**: 验证所有 Tauri 事件监听器被正确移除

**步骤**:
1. 启动应用
2. 打开 DevTools Console
3. 创建多个标签页（点击 + 按钮 5次）
4. 关闭所有标签页
5. 观察控制台输出

**预期结果**:
```
[BrowserPage] Cleaning up 30+ event listeners
[BrowserPage] Cleanup complete
```

**验证**:
- 控制台没有错误
- 事件监听器数量回到初始值

---

### Test 2: Webview 资源释放测试
**目标**: 验证 webview 实例被正确关闭

**步骤**:
1. 启动应用
2. 创建 5 个标签页
3. 在每个标签页中导航到不同网站
4. 关闭应用

**预期结果**:
```
[BrowserPage] Closing 5 webviews
[BrowserPage] Cleanup complete
```

**验证**:
- 所有 webview 进程被终止
- 系统资源监视器显示内存释放

---

### Test 3: 定时器清理测试
**目标**: 验证所有 setTimeout/setInterval 被清除

**步骤**:
1. 启动应用
2. 打开 AI 侧边栏
3. 执行 Agent 命令（触发 setTimeout）
4. 立即关闭侧边栏
5. 等待 5 秒

**预期结果**:
```
[useBrowserSidebar] Cleaning up resources
[useBrowserSidebar] Cleanup complete
```

**验证**:
- 没有延迟执行的回调
- 控制台没有"组件已卸载"错误

---

### Test 4: AbortController 清理测试
**目标**: 验证异步请求被正确取消

**步骤**:
1. 启动应用
2. 打开 AI 聊天
3. 发送一个问题（触发流式响应）
4. 在响应完成前关闭侧边栏

**预期结果**:
```
[useBrowserSidebar] Cleaning up resources
[useBrowserSidebar] Cleanup complete
```

**验证**:
- 流式响应立即停止
- 没有网络请求继续进行

---

### Test 5: 内存泄漏压力测试
**目标**: 长时间运行验证无内存累积

**步骤**:
1. 启动应用
2. 打开 DevTools → Memory → Take Heap Snapshot (快照1)
3. 执行以下操作 20 次：
   - 创建 5 个标签页
   - 在每个标签页中导航到不同网站
   - 关闭所有标签页
4. 强制垃圾回收（DevTools → Memory → Collect garbage）
5. Take Heap Snapshot (快照2)
6. 对比两个快照

**预期结果**:
- 快照2 的内存占用 ≤ 快照1 + 10%
- 没有持续增长的对象
- Detached DOM nodes < 100

**验证指标**:
```
初始内存: ~50MB
20次循环后: ~55MB (允许 ±10%)
```

---

### Test 6: 组件卸载完整性测试
**目标**: 验证所有资源类型都被清理

**步骤**:
1. 启动应用
2. 打开所有功能：
   - 创建多个标签页
   - 打开侧边栏（AI、书签、历史）
   - 打开设置页面
   - 打开下载面板
3. 关闭应用
4. 检查控制台日志

**预期结果**:
```
[BrowserPage] onUnmounted - starting cleanup
[BrowserPage] Cleaning up 30 event listeners
[BrowserPage] Closing 5 webviews
[useBrowserSidebar] Cleaning up resources
[BrowserPage] onUnmounted - cleanup complete
```

**验证**:
- 所有组件都输出清理日志
- 没有"Failed to cleanup"错误
- 应用正常退出

---

## 自动化测试脚本

### 内存泄漏检测脚本
```typescript
// tests/memory-leak.test.ts
import { test, expect } from '@playwright/test';

test('should not leak memory on tab creation/destruction', async ({ page }) => {
  await page.goto('http://localhost:1421');
  
  // 获取初始内存
  const initialMemory = await page.evaluate(() => {
    return (performance as any).memory?.usedJSHeapSize || 0;
  });
  
  // 创建和销毁标签页 10 次
  for (let i = 0; i < 10; i++) {
    // 创建标签页
    await page.click('.tab-new');
    await page.waitForTimeout(100);
    
    // 关闭标签页
    await page.click('.tab-close');
    await page.waitForTimeout(100);
  }
  
  // 强制垃圾回收
  await page.evaluate(() => {
    if ((window as any).gc) {
      (window as any).gc();
    }
  });
  
  await page.waitForTimeout(1000);
  
  // 获取最终内存
  const finalMemory = await page.evaluate(() => {
    return (performance as any).memory?.usedJSHeapSize || 0;
  });
  
  // 内存增长应该小于 20%
  const memoryGrowth = (finalMemory - initialMemory) / initialMemory;
  expect(memoryGrowth).toBeLessThan(0.2);
});
```

---

## 性能基准

### 内存使用基准
| 场景 | 初始内存 | 峰值内存 | 稳定内存 | 增长率 |
|------|---------|---------|---------|--------|
| 空白应用 | 50MB | 50MB | 50MB | 0% |
| 5个标签页 | 50MB | 120MB | 80MB | 60% |
| 20次创建/销毁 | 50MB | 150MB | 55MB | 10% |

### 资源清理基准
| 资源类型 | 创建数量 | 清理数量 | 清理率 |
|---------|---------|---------|--------|
| 事件监听器 | 30 | 30 | 100% |
| Webview | 5 | 5 | 100% |
| 定时器 | 10 | 10 | 100% |
| AbortController | 2 | 2 | 100% |

---

## 问题排查指南

### 问题1: 事件监听器未清理
**症状**: 控制台显示"Cleaning up 0 event listeners"

**排查**:
1. 检查 `eventListeners.push(unlisten)` 是否被调用
2. 检查 `listen().then()` 是否正确链接
3. 添加日志确认 unlisten 函数被保存

**修复**: 确保所有 `listen()` 调用都保存返回值

---

### 问题2: Webview 未关闭
**症状**: 系统资源监视器显示进程未终止

**排查**:
1. 检查 `closeTabWebview()` 是否被调用
2. 检查 webview 标签名是否正确
3. 添加日志确认关闭操作执行

**修复**: 确保在 `onUnmounted` 中遍历所有标签页

---

### 问题3: 定时器继续执行
**症状**: 组件卸载后仍有回调执行

**排查**:
1. 检查 `clearTimeout()` 是否被调用
2. 检查定时器 ID 是否正确保存
3. 添加日志确认清理执行

**修复**: 使用 `ResourceManager` 管理所有定时器

---

## 测试报告模板

```markdown
# 资源清理测试报告

**测试日期**: YYYY-MM-DD
**测试人员**: [姓名]
**应用版本**: [版本号]

## 测试结果摘要
- ✅ 事件监听器清理: PASS
- ✅ Webview 资源释放: PASS
- ✅ 定时器清理: PASS
- ✅ AbortController 清理: PASS
- ✅ 内存泄漏压力测试: PASS
- ✅ 组件卸载完整性: PASS

## 详细结果
[详细测试数据和截图]

## 发现的问题
[列出所有发现的问题]

## 建议
[改进建议]
```

---

## 持续监控

### 1. 添加性能监控
```typescript
// 在应用中添加性能监控
if (import.meta.env.DEV) {
  setInterval(() => {
    const memory = (performance as any).memory;
    if (memory) {
      console.log('[Memory]', {
        used: Math.round(memory.usedJSHeapSize / 1024 / 1024) + 'MB',
        total: Math.round(memory.totalJSHeapSize / 1024 / 1024) + 'MB',
      });
    }
  }, 30000); // 每30秒记录一次
}
```

### 2. 添加资源追踪
```typescript
// 追踪资源创建和销毁
const resourceTracker = {
  created: 0,
  destroyed: 0,
  get active() {
    return this.created - this.destroyed;
  }
};
```

---

## 总结

本测试计划涵盖了所有关键的资源清理场景。通过系统的测试，可以确保：
1. ✅ 所有事件监听器被正确移除
2. ✅ 所有 webview 资源被释放
3. ✅ 所有定时器被清除
4. ✅ 所有异步操作被取消
5. ✅ 长时间运行无内存泄漏

定期执行这些测试可以及早发现和修复资源泄漏问题。
