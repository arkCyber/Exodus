#!/bin/bash

# Exodus 浏览器资源清理测试脚本
# 用于快速验证资源管理修复是否生效

echo "=========================================="
echo "Exodus 资源清理测试"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试计数器
PASSED=0
FAILED=0

# 测试函数
test_step() {
    echo -e "${YELLOW}测试:${NC} $1"
}

test_pass() {
    echo -e "${GREEN}✓ 通过${NC}"
    ((PASSED++))
    echo ""
}

test_fail() {
    echo -e "${RED}✗ 失败: $1${NC}"
    ((FAILED++))
    echo ""
}

# 检查文件是否存在
check_file() {
    if [ -f "$1" ]; then
        return 0
    else
        return 1
    fi
}

# 检查文件中是否包含特定内容
check_content() {
    if grep -q "$2" "$1"; then
        return 0
    else
        return 1
    fi
}

echo "1. 检查修复文件..."
echo "----------------------------"

# 检查 BrowserPage.vue
test_step "BrowserPage.vue 包含事件监听器清理"
if check_file "src/views/BrowserPage.vue" && \
   check_content "src/views/BrowserPage.vue" "const eventListeners: UnlistenFn\[\] = \[\]" && \
   check_content "src/views/BrowserPage.vue" "eventListeners.push(unlisten)"; then
    test_pass
else
    test_fail "未找到事件监听器清理代码"
fi

# 检查 watch 清理
test_step "BrowserPage.vue 包含 watch 监听器清理"
if check_content "src/views/BrowserPage.vue" "const watchStoppers: Array<() => void> = \[\]" && \
   check_content "src/views/BrowserPage.vue" "watchStoppers.push(watch"; then
    test_pass
else
    test_fail "未找到 watch 监听器清理代码"
fi

# 检查 webview 清理
test_step "BrowserPage.vue 包含 webview 清理"
if check_content "src/views/BrowserPage.vue" "closeTabWebview(tabWebviewLabel(tab.id))"; then
    test_pass
else
    test_fail "未找到 webview 清理代码"
fi

# 检查 useBrowserSidebar
test_step "useBrowserSidebar.ts 包含定时器清理"
if check_file "src/composables/useBrowserSidebar.ts" && \
   check_content "src/composables/useBrowserSidebar.ts" "const pendingTimeouts: number\[\] = \[\]" && \
   check_content "src/composables/useBrowserSidebar.ts" "clearTimeout(timerId)"; then
    test_pass
else
    test_fail "未找到定时器清理代码"
fi

# 检查 useBrowserSitePermissions
test_step "useBrowserSitePermissions.ts 包含自动清理"
if check_file "src/composables/useBrowserSitePermissions.ts" && \
   check_content "src/composables/useBrowserSitePermissions.ts" "onUnmounted"; then
    test_pass
else
    test_fail "未找到自动清理钩子"
fi

# 检查资源管理工具
test_step "resourceCleanup.ts 工具文件存在"
if check_file "src/lib/resourceCleanup.ts" && \
   check_content "src/lib/resourceCleanup.ts" "class ResourceManager"; then
    test_pass
else
    test_fail "未找到资源管理工具"
fi

echo ""
echo "2. 检查文档文件..."
echo "----------------------------"

# 检查文档
test_step "审计报告文档存在"
if check_file "MEMORY_LEAK_AUDIT_REPORT.md"; then
    test_pass
else
    test_fail "未找到审计报告"
fi

test_step "测试计划文档存在"
if check_file "RESOURCE_CLEANUP_TEST_PLAN.md"; then
    test_pass
else
    test_fail "未找到测试计划"
fi

test_step "修复总结文档存在"
if check_file "RESOURCE_CLEANUP_SUMMARY.md"; then
    test_pass
else
    test_fail "未找到修复总结"
fi

test_step "最终报告文档存在"
if check_file "FINAL_AUDIT_REPORT.md"; then
    test_pass
else
    test_fail "未找到最终报告"
fi

echo ""
echo "=========================================="
echo "测试结果"
echo "=========================================="
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    echo ""
    echo "下一步:"
    echo "1. 运行应用: pnpm tauri dev"
    echo "2. 打开开发者工具 (Cmd + Option + I)"
    echo "3. 创建和关闭标签页"
    echo "4. 检查控制台输出"
    echo ""
    echo "预期看到:"
    echo "  [BrowserPage] Cleaning up 30+ event listeners"
    echo "  [BrowserPage] Stopping 5 watch listeners"
    echo "  [BrowserPage] Closing X webviews"
    echo "  [BrowserPage] Cleanup complete"
    echo ""
    exit 0
else
    echo -e "${RED}✗ 有测试失败，请检查修复${NC}"
    echo ""
    exit 1
fi
