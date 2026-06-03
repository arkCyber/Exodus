# 原生截图功能代码审计报告

## 功能概述
使用 Rust `xcap` 库实现原生屏幕截图，在菜单打开时显示 webview 内容占位符。

## 代码流程

### 1. 触发条件
- **文件**: `src/views/BrowserPage.vue`
- **函数**: `onMenuOpened()`
- **触发**: 用户点击菜单按钮时
- **条件**: `useNativeWebview.value && !isNewTabUrl && !isChromeInternalUrl`

### 2. 执行顺序
```
用户点击菜单
  ↓
onMenuOpened() 调用
  ↓
1. 调用 Rust: browser_capture_screenshot
  ↓
2. 渲染到 Canvas: renderScreenshotToCanvas()
  ↓
3. 移动 webview 到屏幕外: moveWebviewOffScreen()
  ↓
4. 显示截图占位符
```

### 3. Rust 后端 (src-tauri/src/browser.rs)

#### 函数: `browser_capture_screenshot`

**步骤 1: 获取 webview 尺寸**
```rust
// 通过 JavaScript 获取 viewport 尺寸
window.innerWidth, window.innerHeight
// 预期: 1280x572 (逻辑像素)
```

**步骤 2: 获取窗口信息**
```rust
let window_pos = main_window.outer_position();  // (200, 200)
let window_size = main_window.outer_size();     // 2560x1440
let scale_factor = main_window.scale_factor();  // 2.0 (Retina)
```

**步骤 3: 计算截图区域**
```rust
let titlebar_height = 28;   // macOS 标题栏
let addressbar_height = 48; // 地址栏
let webview_offset_y = 76;  // 总偏移

// 逻辑坐标
let logical_x = 200;
let logical_y = 200 + 76 = 276;
let logical_width = 1280;
let logical_height = 572;
```

**步骤 4: 捕获屏幕**
```rust
// xcap 使用逻辑坐标，返回物理像素图像
let image = primary_monitor.capture_region(200, 276, 1280, 572);
// 返回: 2560x1144 (物理像素，2x)
```

**步骤 5: 返回数据**
```rust
// JSON 格式
{
  "width": 2560,
  "height": 1144,
  "data": "base64_encoded_rgba_pixels"
}
```

### 4. 前端渲染 (src/views/BrowserPage.vue)

#### 函数: `renderScreenshotToCanvas`

**步骤 1: 解析数据**
```javascript
const { width, height, data } = JSON.parse(screenshotData);
// width: 2560, height: 1144
```

**步骤 2: 设置 Canvas**
```javascript
// 内部分辨率（物理像素）
canvas.width = 2560;
canvas.height = 1144;

// 计算 CSS 尺寸（逻辑像素）
const dpr = window.devicePixelRatio; // 2.0
const cssWidth = 2560 / 2.0 = 1280;
const cssHeight = 1144 / 2.0 = 572;

// 设置 CSS 尺寸
canvas.style.width = '1280px';
canvas.style.height = '572px';
```

**步骤 3: 解码并渲染**
```javascript
// Base64 → Uint8Array
const bytes = atob(data);

// 创建 ImageData
const imageData = new ImageData(bytes, 2560, 1144);

// 渲染到 Canvas
ctx.putImageData(imageData, 0, 0);
```

## 关键日志点

### Rust 日志
```
browser_capture_screenshot called with label: exodus-tab-XXX
Got content webview for label: XXX (Xms)
Webview viewport: 1280x572, scroll area: 1280x1218 (Xms)
Logical coordinates: pos=(200, 276), size=1280x572
Capturing region: x=200, y=276, w=1280, h=572 (monitor: 1496x967, scale=2) (Xms)
Screenshot captured: 2560x1144 in Xms (total: Xms)
Screenshot data created, width=2560, height=1144, data length=15619416, total time: Xms
```

### 前端日志
```
[BrowserPage] Capturing screenshot for tab: XXX
[BrowserPage] Screenshot captured, length: XXXXXX
[BrowserPage] Parsed screenshot data: 2560 x 1144 in X.X ms
[BrowserPage] Canvas resolution: 2560 x 1144 DPR: 2 CSS size: 1280 x 572
[BrowserPage] Decoded base64 in XX.X ms
[BrowserPage] Screenshot rendered in X.X ms, total: XX.X ms
[BrowserPage] Screenshot rendered, now moving webview
[BrowserPage] Webview moved off-screen for menu
```

## DOM 结构

```html
<div class="webview-container">
  <!-- 截图占位符 (v-if="showMenu && webviewScreenshot") -->
  <div class="webview-screenshot-placeholder">
    <canvas ref="screenshotCanvas" class="screenshot-canvas" />
  </div>
  
  <!-- 白色占位符 (v-else-if="showMenu && !webviewScreenshot") -->
  <div class="webview-white-placeholder">
    <div class="placeholder-text">Page Title</div>
  </div>
  
  <!-- Native webview host -->
  <div class="native-webview-host" />
</div>
```

## CSS 样式

```css
.webview-screenshot-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
  background: #fff;
  pointer-events: none;
}

.screenshot-canvas {
  display: block;
  position: absolute;
  top: 0;
  left: 0;
  /* 尺寸由 JavaScript 动态设置 */
  image-rendering: auto;
  image-rendering: -webkit-optimize-contrast;
}
```

## 性能指标

### 预期时间
- **获取 webview 尺寸**: ~5ms
- **截图捕获**: ~150-200ms
- **Base64 编码**: ~50ms
- **前端解码**: ~40-50ms
- **Canvas 渲染**: ~10-15ms
- **总时间**: ~250-320ms

### 数据大小
- **物理像素**: 2560 × 1144 = 2,928,640 像素
- **RGBA 数据**: 2,928,640 × 4 = 11,714,560 字节 (~11.2 MB)
- **Base64 编码**: ~15,619,416 字符 (~15.6 MB)

## 测试步骤

1. **启动应用**
   ```bash
   pnpm tauri dev
   ```

2. **导航到网页**
   - 访问任何非 chrome:// 或 about:blank 页面
   - 例如: https://stackoverflow.com

3. **打开菜单**
   - 点击右上角菜单按钮（三个点）
   - 或使用快捷键

4. **检查日志**
   - 查看终端中的 Rust 日志
   - 打开浏览器开发者工具查看前端日志

5. **验证显示**
   - 截图应该显示页面内容
   - 尺寸应该完全匹配 webview 区域
   - 无缩放变形
   - 清晰锐利

## 已知问题和解决方案

### ✅ 已解决
1. **边界超出错误** - 使用逻辑坐标，不手动应用 scale_factor
2. **空白截图** - 先截图再移动 webview
3. **缩放变形** - 使用 DPR 计算精确的 CSS 尺寸
4. **性能问题** - 使用原始 RGBA 数据，避免 PNG 压缩

### ⚠️ 待验证
1. 不同窗口尺寸下的表现
2. 多显示器环境
3. 不同 DPR 值（1x, 1.5x, 2x, 3x）
4. 长页面滚动后的截图

## 配置参数

### 可调整参数
```rust
// src-tauri/src/browser.rs
let titlebar_height = 28;   // macOS 标题栏高度
let addressbar_height = 48; // 地址栏高度
```

### 性能优化选项
```javascript
// src/views/BrowserPage.vue
const ctx = canvas.getContext('2d', {
  alpha: false,              // 禁用透明度
  desynchronized: true,      // 异步渲染
  willReadFrequently: false  // 优化写入
});
```

## 依赖项

### Rust
- `xcap = "0.9.6"` - 屏幕截图库
- `base64` - Base64 编码
- `serde_json` - JSON 序列化

### 前端
- Vue 3 reactive system
- Canvas API
- atob() - Base64 解码
