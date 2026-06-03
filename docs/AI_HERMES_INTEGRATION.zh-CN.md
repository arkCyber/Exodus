# AI 推理与 Hermes 智能体集成

**中文版** · [English](./AI_HERMES_INTEGRATION.md)

## 统一技术栈

| 层级 | 组件 | 作用 |
|------|------|------|
| 进程 | **Allama**（`ai_port`，默认 11435） | 本地 GGUF 推理（Ollama 兼容 HTTP） |
| Rust | **InferenceEngine** | 模型注册表；路由到 Allama HTTP 或嵌入式 stub 网关 |
| Rust | **HermesAgent** | 任务/策略层；Analysis 任务经 `AllamaHttpClient` 调 Allama |
| UI | **侧边栏 AI** | `allamaClient.ts` — 直连 HTTP 流式 |
| UI | **智能体面板** | `hermes_analyze_page` → Hermes → Allama；失败时回退 `streamSidebarChat` |

可选 **exodus-core** Sidecar 与 Allama **共用** `ai_port` — 同一端口只能有一个监听进程。

## 智能体面板链路（已打通）

1. `compress_dom` 压缩当前页 DOM → `agentDomSummary`
2. **DOM 命令**（scroll / links / JSON）→ `hermes_plan_agent_action` → `execute_agent_action_with_context` → 注入 webview
3. 用户 `ask:` 或 **Ask AI** → `hermes_analyze_page`（Analysis + Allama）→ 失败时 `streamSidebarChat`
4. Hermes 任务类型 Navigation / FormFill / DataExtraction / Automation 产出 `actionJson` 供同一执行器使用
5. **多步规划** — `plan: scroll down then links` → `hermes_plan_automation_steps`（Allama 输出 JSON 步骤数组，离线时按 ` then ` 拆分）
6. **Compress DOM** 后自动 `hermes_sync_agent_context` 同步 URL / 标签 / DOM 摘要
7. **策略模板** — 下拉运行内置模板；**Save…** 将当前命令保存为自定义策略（localStorage）；自定义项可 **Delete**
8. **Inference 引擎 UI** — 设置 → Inference engine (Rust)：模型列表、加载/卸载、`inference_generate` / `inference_chat` 测试

配置变更：`set_ai_config` 更新端口时会同步 **Hermes**、**InferenceEngine**、扩展 shim。

## 测试

```bash
pnpm test:ai-hermes
pnpm test:extensions
pnpm verify
```

## 相关文档

- [ALLAMA_INTEGRATION.md](../ALLAMA_INTEGRATION.md)
- [EXTENSIONS_DEV.zh-CN.md](./EXTENSIONS_DEV.zh-CN.md)
