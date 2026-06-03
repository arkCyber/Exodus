# 内存泄漏审计报告 (Memory Leak Audit Report)

## 执行日期
2026-05-26

## 审计范围
全面审计 Exodus 浏览器的资源释放和内存泄漏问题

---

## 🔴 发现的严重问题

### 1. **事件监听器泄漏 (Critical)**
**位置**: `src/views/BrowserPage.vue`

**问题描述**:
- 发现 **30+ 个 Tauri 事件监听器**未被正确清理
- 所有 `listen()` 调用都返回 `UnlistenFn`，但大部分被忽略
- 原代码注释错误地声明"Menu event listeners don't need explicit cleanup"

**受影响的事件**:
```typescript
- exodus-new-tab
- exodus-new-incognito-window
- exodus-open-settings
- exodus-open-bookmarks
- exodus-bookmark-this-page
- exodus-open-downloads
- exodus-open-profile-settings
- exodus-print
- exodus-find
- exodus-zoom-in/out/reset
- exodus-quit
- exodus-quit-request
- exodus-cleanup-resources
- exodus-navigate
- exodus-reopen-closed-tab
- exodus-open-file
- exodus-focus-address-bar
- exodus-close-tab/window
- exodus-save-page
- exodus-find-next/previous
- exodus-reload/force-reload
- exodus-developer-tools
- exodus-select-next-tab/previous-tab
- exodus-move-tab-to-new-window
- exodus-help-docs
- exodus-report-issue
- exodus-search
- exodus-about
```

**影响**:
- 每次创建/销毁 BrowserPage 组件都会泄漏所有监听器
- 长时间运行会累积大量未释放的回调函数
- 可能导致内存占用持续增长

**修复方案**:
✅ 创建 `eventListeners` 数组存储所有 unlisten 函数
✅ 在 `onUnmounted` 中遍历并调用所有 unlisten 函数
✅ 清空数组释放引用

---

### 2. **Webview 资源未释放 (Critical)**
**位置**: `src/views/BrowserPage.vue` - `onUnmounted`

**问题描述**:
- 组件卸载时没有关闭所有 webview 实例
- 每个 webview 都占用大量内存和系统资源
- 标签页关闭时 webview 可能未被正确销毁

**修复方案**:
✅ 在 `onUnmounted` 中遍历所有标签页
✅ 调用 `closeTabWebview()` 关闭每个 webview
✅ 添加错误处理确保即使部分失败也能继续清理

---

### 3. **资源清理不完整 (Medium)**
**位置**: `exodus-cleanup-resources` 事件处理器

**问题描述**:
- 原实现只有空的注释，没有实际清理代码
- 缺少 webview 关闭逻辑
- 缺少缓存清理逻辑

**修复方案**:
✅ 添加 webview 关闭逻辑
✅ 添加 `invalidateWallpaperCache()` 清理缓存
✅ 保留会话保存逻辑

---

## ✅ 已实施的修复

### 1. 事件监听器管理
```typescript
// 添加存储数组
const eventListeners: UnlistenFn[] = [];

// 每个监听器都保存 unlisten 函数
void listen('event-name', handler).then((unlisten) => {
  eventListeners.push(unlisten);
}).catch((e) => {
  shellLog.error('listener failed', e);
});
```

### 2. 组件卸载清理
```typescript
onUnmounted(async () => {
  // 清理所有事件监听器
  for (const unlisten of eventListeners) {
    try {
      unlisten();
    } catch (e) {
      console.error('Failed to unlisten event:', e);
    }
  }
  eventListeners.length = 0;
  
  // 关闭所有 webviews
  if (useNativeWebview) {
    for (const tab of tabs.value) {
      if (tab.webview) {
        await closeTabWebview(tabWebviewLabel(tab.id));
      }
    }
  }
  
  // 其他清理...
});
```

### 3. 退出前资源清理
```typescript
void listen('exodus-cleanup-resources', async () => {
  // 关闭所有 webviews
  for (const tab of tabs.value) {
    if (tab.webview && useNativeWebview) {
      await closeTabWebview(tabWebviewLabel(tab.id));
    }
  }
  
  // 保存会话
  await browserSession.saveSession();
  
  // 清理缓存
  invalidateWallpaperCache();
});
```

---

## 📊 修复效果评估

### 内存泄漏修复
- ✅ **30+ 事件监听器泄漏** → 已修复
- ✅ **Webview 资源泄漏** → 已修复
- ✅ **缓存未清理** → 已修复

### 预期改进
- 🎯 组件销毁时内存完全释放
- 🎯 长时间运行不会累积泄漏
- 🎯 应用退出时资源正确清理
- 🎯 减少内存占用峰值

---

## 🔍 其他潜在问题（需要进一步审计）

### 1. Watch 监听器
**位置**: 多处使用 `watch()` 但未检查是否正确清理

**建议**:
```typescript
const stopWatch = watch(source, callback);
// 在 onUnmounted 中调用 stopWatch()
```

### 2. 定时器泄漏
**位置**: 需要检查所有 `setTimeout`/`setInterval` 使用

**建议**:
- 保存定时器 ID
- 在组件卸载时清除所有定时器

### 3. DOM 事件监听器
**位置**: 直接使用 `addEventListener` 的地方

**当前状态**:
- ✅ `online`/`offline` 事件已清理
- ✅ `beforeunload` 事件已清理
- ✅ `OPEN_WEBCHAT_EVENT` 已清理

### 4. 异步任务取消
**位置**: 长时间运行的 Promise 和异步操作

**建议**:
- 使用 AbortController 取消未完成的请求
- 检查组件是否已卸载再更新状态

---

## 📝 最佳实践建议

### 1. 事件监听器模式
```typescript
// ✅ 正确
const unlisteners: UnlistenFn[] = [];

onMounted(() => {
  listen('event', handler).then(ul => unlisteners.push(ul));
});

onUnmounted(() => {
  unlisteners.forEach(ul => ul());
});
```

```typescript
// ❌ 错误
onMounted(() => {
  void listen('event', handler); // 泄漏！
});
```

### 2. Webview 生命周期
```typescript
// ✅ 创建时
const wv = await createTabWebview(...);
tabs.value = tabs.value.map(t => 
  t.id === tabId ? { ...t, webview: wv } : t
);

// ✅ 销毁时
await closeTabWebview(tabWebviewLabel(tabId));
tabs.value = tabs.value.map(t =>
  t.id === tabId ? { ...t, webview: null } : t
);
```

### 3. 资源清理检查清单
- [ ] 所有事件监听器已移除
- [ ] 所有 webview 已关闭
- [ ] 所有定时器已清除
- [ ] 所有 watch 已停止
- [ ] 所有异步任务已取消
- [ ] 所有缓存已清理
- [ ] 所有 DOM 引用已释放

---

## 🎯 后续行动项

### 高优先级
1. ✅ 修复事件监听器泄漏
2. ✅ 修复 webview 资源泄漏
3. ⏳ 审计所有 composables 的资源清理
4. ⏳ 审计所有子组件的 onUnmounted

### 中优先级
5. ⏳ 添加内存泄漏检测工具
6. ⏳ 创建资源清理测试用例
7. ⏳ 文档化资源管理最佳实践

### 低优先级
8. ⏳ 性能监控和内存使用追踪
9. ⏳ 自动化内存泄漏检测

---

## 📈 测试建议

### 手动测试
1. 打开应用，创建多个标签页
2. 关闭所有标签页
3. 打开开发者工具 → Memory → Take Heap Snapshot
4. 重复步骤 1-2 多次
5. 再次 Take Heap Snapshot
6. 对比两次快照，检查是否有持续增长的对象

### 自动化测试
```typescript
describe('Memory Leak Tests', () => {
  it('should cleanup all event listeners on unmount', () => {
    const wrapper = mount(BrowserPage);
    const initialListeners = getEventListenerCount();
    wrapper.unmount();
    const finalListeners = getEventListenerCount();
    expect(finalListeners).toBe(initialListeners);
  });
});
```

---

## 总结

本次审计发现并修复了 **3 个严重的内存泄漏问题**：
1. ✅ 30+ 事件监听器未清理
2. ✅ Webview 资源未释放
3. ✅ 退出时资源清理不完整

所有修复已实施并经过代码审查。建议进行全面测试以验证修复效果。

---

**审计人员**: Cascade AI  
**审计日期**: 2026-05-26  
**文档版本**: 1.0
