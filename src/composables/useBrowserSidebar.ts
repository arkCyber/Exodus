/**
 * Exodus Browser — right sidebar state (AI chat, memory, bookmarks, agent, P2P room).
 */
import { shellLog } from '@/lib/diagnosticLog';
import { ref, computed, onUnmounted, type Ref } from 'vue';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
  AiChatMessage,
  BookmarkItem,
  HistoryPage,
  IndexedPage,
  SidebarPanel,
} from '@/lib/browserTypes';
import { groupHistoryByDate, type HistoryGroup } from '@/lib/historyGroups';
import { groupIndexedByDate } from '@/lib/indexedMemory';
import { checkSidebarAiOnline, streamSidebarChat } from '@/lib/sidebarAiChat';
import { isNewTabUrl } from '@/lib/newTabPage';
import {
  agentActionReturnsValue,
  parseAgentCommandInput,
  summarizeCompressedDom,
} from '@/lib/agentActions';
import {
  hermesAnalyzePage,
  hermesPlanAgentAction,
  hermesPlanAutomationSteps,
  hermesRunStrategySteps,
  hermesSyncAgentContext,
} from '@/lib/hermesClient';
import {
  listHermesStrategyTemplates,
  actionJsonFromStepResult,
} from '@/lib/hermesStrategies';
import { evalInTab, evalTabReturning, getTabHtml } from '@/lib/exodusBrowser';
import { loadMergedBrowsingHistory } from '@/lib/historyManager';

export type UseBrowserSidebarOptions = {
  getCurrentUrl: () => string;
  getActiveTabLabel: () => string;
  useNativeWebview: Ref<boolean>;
  getContentDocument: () => Document | null | undefined;
  navigate: (url: string) => void | Promise<void>;
  onStatus: (message: string) => void;
  aiPort: Ref<number>;
  aiModel: Ref<string>;
  loadBookmarks: () => Promise<void>;
  getBookmarks: () => BookmarkItem[];
  removeBookmark: (id: string) => Promise<void>;
  updateBookmarkFolder: (id: string, folder: string) => Promise<void>;
};

/**
 * Sidebar UI state and handlers (ported from Svelte +page.svelte).
 */
export function useBrowserSidebar(options: UseBrowserSidebarOptions) {
  const sidebarOpen = ref(false);
  const sidebarPanel = ref<SidebarPanel>('ai');
  const agentPanelOpen = ref(false);

  const aiChatHistory = ref<AiChatMessage[]>([]);
  const aiChatInput = ref('');
  const chatStreamBuffer = ref('');
  const aiStreamMode = ref<'none' | 'chat' | 'summary'>('none');
  const isLoading = ref(false);
  const aiOnline = ref(false);
  const aiResponse = ref('');

  const indexedPages = ref<IndexedPage[]>([]);
  const historyPages = ref<HistoryPage[]>([]);

  const agentCommand = ref('');
  const agentLog = ref<string[]>([]);
  const agentDomSummary = ref('');
  const isAgentExecuting = ref(false);

  const p2pRoomId = ref('lobby');
  const bookmarkSearchQuery = ref('');
  const memorySearchQuery = ref('');

  let chatAbortController: AbortController | null = null;
  const pendingTimeouts: number[] = [];

  const indexedMemoryGroups = computed(() => groupIndexedByDate(indexedPages.value));
  const historyGroups = computed(() => groupHistoryByDate(historyPages.value));

  /** Filter history groups by sidebar search query (Firefox history search). */
  function filterHistoryGroupsByQuery(groups: HistoryGroup[], query: string): HistoryGroup[] {
    const q = query.trim().toLowerCase();
    if (!q) return groups;
    return groups
      .map((g) => ({
        ...g,
        pages: g.pages.filter(
          (p) =>
            (p.title ?? '').toLowerCase().includes(q) || p.url.toLowerCase().includes(q),
        ),
      }))
      .filter((g) => g.pages.length > 0);
  }

  const filteredIndexedMemoryGroups = computed(() =>
    filterHistoryGroupsByQuery(indexedMemoryGroups.value, memorySearchQuery.value),
  );
  const filteredHistoryGroups = computed(() =>
    filterHistoryGroupsByQuery(historyGroups.value, memorySearchQuery.value),
  );
  const indexedCount = computed(() => indexedPages.value.length);
  const historyCount = computed(() => historyPages.value.length);

  const filteredBookmarks = computed(() => {
    const q = bookmarkSearchQuery.value.trim().toLowerCase();
    const list = options.getBookmarks();
    if (!q) return list;
    return list.filter(
      (b) =>
        b.title.toLowerCase().includes(q) ||
        b.url.toLowerCase().includes(q) ||
        (b.folder ?? '').toLowerCase().includes(q),
    );
  });

  const canAnnouncePage = computed(() => {
    const url = options.getCurrentUrl();
    return (
      !isNewTabUrl(url) &&
      (url.startsWith('http://') || url.startsWith('https://'))
    );
  });

  function openPanel(panel: SidebarPanel): void {
    shellLog.info('openPanel called with', panel);
    sidebarPanel.value = panel;
    agentPanelOpen.value = false;
    sidebarOpen.value = true;
    shellLog.info('Set sidebarPanel to', panel, 'sidebarOpen to', true);
    if (panel === 'tabs') {
      /* vertical tab strip lives in sidebar; no extra load */
    } else if (panel === 'synced' || panel === 'reading') {
      /* child panels load on mount */
    } else if (panel === 'customize') {
      /* preferences UI only */
    }
    if (panel === 'memory') {
      void loadIndexedMemory();
      void refreshHistory();
    }
    if (panel === 'bookmarks') {
      void options.loadBookmarks();
    }
  }

  function toggleSidebar(): void {
    sidebarOpen.value = !sidebarOpen.value;
  }

  function closeSidebar(): void {
    sidebarOpen.value = false;
  }

  function toggleAgentPanel(): void {
    agentPanelOpen.value = !agentPanelOpen.value;
    if (agentPanelOpen.value) {
      sidebarPanel.value = 'ai';
    }
  }

  function agentBackToAi(): void {
    agentPanelOpen.value = false;
    sidebarPanel.value = 'ai';
  }

  async function probeAiOnline(): Promise<void> {
    aiOnline.value = await checkSidebarAiOnline(options.aiPort.value);
  }

  async function loadIndexedMemory(): Promise<void> {
    if (!isTauri()) return;
    try {
      const pages = (await invoke('get_history')) as IndexedPage[];
      indexedPages.value = pages;
    } catch (error) {
      shellLog.error('Failed to load indexed memory', error);
      indexedPages.value = [];
    }
  }

  async function refreshHistory(): Promise<void> {
    try {
      historyPages.value = await loadMergedBrowsingHistory();
    } catch (error) {
      shellLog.error('Failed to load history', error);
      historyPages.value = [];
    }
  }

  async function removeIndexedPage(id: string): Promise<void> {
    try {
      await invoke('delete_indexed_page', { id });
      indexedPages.value = indexedPages.value.filter((p) => p.id !== id);
      options.onStatus('Removed from indexed memory');
    } catch (error) {
      shellLog.error('Failed to remove indexed page', error);
      options.onStatus('Failed to remove page');
    }
  }

  async function clearIndexedMemory(): Promise<void> {
    try {
      await invoke('clear_rag_data');
      indexedPages.value = [];
      options.onStatus('Local memory cleared');
    } catch (error) {
      shellLog.error('Failed to clear local memory', error);
      options.onStatus('Failed to clear local memory');
    }
  }

  async function clearBrowsingHistory(): Promise<void> {
    try {
      await invoke('clear_visit_history');
      historyPages.value = [];
      options.onStatus('Browsing history cleared');
    } catch (error) {
      shellLog.error('Failed to clear browsing history', error);
      options.onStatus('Failed to clear browsing history');
    }
  }

  function cancelChat(): void {
    chatAbortController?.abort();
    chatStreamBuffer.value = '';
    isLoading.value = false;
    aiStreamMode.value = 'none';
    chatAbortController = null;
  }

  async function sendAiChat(): Promise<void> {
    const prompt = aiChatInput.value.trim();
    if (!prompt || isLoading.value) return;
    aiChatInput.value = '';
    const historyWithUser: AiChatMessage[] = [
      ...aiChatHistory.value,
      { role: 'user', content: prompt },
    ];
    aiChatHistory.value = historyWithUser;
    chatStreamBuffer.value = '';
    aiStreamMode.value = 'chat';
    isLoading.value = true;
    sidebarPanel.value = 'ai';
    agentPanelOpen.value = false;
    sidebarOpen.value = true;

    if (!aiOnline.value) {
      aiChatHistory.value = [
        ...aiChatHistory.value,
        {
          role: 'assistant',
          content: 'Error: Allama is offline — open Settings and start Allama (port 11435)',
        },
      ];
      isLoading.value = false;
      aiStreamMode.value = 'none';
      return;
    }

    chatAbortController?.abort();
    chatAbortController = new AbortController();
    const signal = chatAbortController.signal;

    await streamSidebarChat(
      historyWithUser,
      { port: options.aiPort.value, model: options.aiModel.value, signal },
      {
        onChunk: (content) => {
          chatStreamBuffer.value += content;
        },
        onDone: () => {
          if (chatStreamBuffer.value.trim()) {
            aiChatHistory.value = [
              ...aiChatHistory.value,
              { role: 'assistant', content: chatStreamBuffer.value },
            ];
          }
          chatStreamBuffer.value = '';
          isLoading.value = false;
          aiStreamMode.value = 'none';
          chatAbortController = null;
        },
        onError: (message) => {
          if (signal.aborted) {
            chatStreamBuffer.value = '';
            isLoading.value = false;
            aiStreamMode.value = 'none';
            chatAbortController = null;
            return;
          }
          aiChatHistory.value = [
            ...aiChatHistory.value,
            { role: 'assistant', content: `Error: ${message}` },
          ];
          chatStreamBuffer.value = '';
          isLoading.value = false;
          aiStreamMode.value = 'none';
          chatAbortController = null;
        },
      },
    );
  }

  function addAgentLog(line: string): void {
    agentLog.value = [...agentLog.value, line];
  }

  async function runAgentAction(actionJson: string): Promise<void> {
    isAgentExecuting.value = true;
    addAgentLog(`Executing: ${actionJson.substring(0, 120)}`);
    const currentUrl = options.getCurrentUrl();
    const label = options.getActiveTabLabel();

    try {
      const jsCode = await invoke<string>('execute_agent_action_with_context', {
        actionJson,
        currentUrl,
      });
      const returnsValue = agentActionReturnsValue(actionJson);

      if (options.useNativeWebview.value) {
        if (returnsValue) {
          const result = await evalTabReturning(label, jsCode);
          const preview = result.length > 1200 ? `${result.slice(0, 1200)}…` : result;
          addAgentLog(preview || '(empty)');
        } else {
          await evalInTab(label, jsCode);
          addAgentLog(`Ran: ${jsCode.substring(0, 80)}`);
        }
      } else {
        const doc = options.getContentDocument();
        if (doc) {
          // eslint-disable-next-line no-new-func
          const fn = new Function(jsCode);
          fn.call(doc.defaultView);
        }
        addAgentLog(returnsValue ? 'Ran (iframe)' : `Ran: ${jsCode.substring(0, 80)}`);
      }
    } catch (error) {
      addAgentLog(`Execution failed: ${error}`);
    } finally {
      isAgentExecuting.value = false;
    }
  }

  async function compressCurrentDom(): Promise<void> {
    try {
      const currentUrl = options.getCurrentUrl();
      const html = options.useNativeWebview.value
        ? await getTabHtml(options.getActiveTabLabel())
        : options.getContentDocument()?.documentElement.outerHTML || '';

      if (!html) {
        addAgentLog('Cannot read DOM');
        return;
      }

      const compressed = await invoke<string>('compress_dom', { html, url: currentUrl });
      agentDomSummary.value = summarizeCompressedDom(compressed);
      addAgentLog(agentDomSummary.value);
      try {
        await hermesSyncAgentContext({
          currentUrl,
          tabId: options.getActiveTabLabel(),
          pageContext: agentDomSummary.value,
        });
      } catch (syncErr) {
        shellLog.debug('hermes context sync', syncErr);
      }
    } catch (error) {
      addAgentLog(`DOM compression failed: ${error}`);
    }
  }

  async function askAgentWithAllama(question: string): Promise<void> {
    agentPanelOpen.value = false;
    sidebarPanel.value = 'ai';
    sidebarOpen.value = true;
    const prompt = agentDomSummary.value
      ? `Page context:\n${agentDomSummary.value}\n\nUser question: ${question}`
      : question;
    aiChatInput.value = '';
    const historyWithUser: AiChatMessage[] = [
      ...aiChatHistory.value,
      { role: 'user', content: prompt },
    ];
    aiChatHistory.value = historyWithUser;
    chatStreamBuffer.value = '';
    aiStreamMode.value = 'chat';
    isLoading.value = true;

    if (!aiOnline.value) {
      aiChatHistory.value = [
        ...aiChatHistory.value,
        {
          role: 'assistant',
          content: 'Error: Allama is offline — open Settings and start Allama (port 11435)',
        },
      ];
      isLoading.value = false;
      aiStreamMode.value = 'none';
      return;
    }

    try {
      const hermes = await hermesAnalyzePage({
        question,
        pageContext: agentDomSummary.value || undefined,
        currentUrl: options.getCurrentUrl(),
        tabId: options.getActiveTabLabel(),
        model: options.aiModel.value,
      });
      if (hermes.answer?.trim()) {
        aiChatHistory.value = [
          ...aiChatHistory.value,
          { role: 'assistant', content: hermes.answer.trim() },
        ];
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      aiChatHistory.value = [
        ...aiChatHistory.value,
        { role: 'assistant', content: `Error: ${msg}` },
      ];
    } finally {
      isLoading.value = false;
      aiStreamMode.value = 'none';
    }
  }

  async function runHermesStepPlans(
    steps: Array<{ actionJson?: string; message?: string }>,
  ): Promise<void> {
    for (const step of steps) {
      if (!step.actionJson) continue;
      addAgentLog(step.message ?? 'Hermes step');
      await runAgentAction(step.actionJson);
      await new Promise((r) => {
        const timerId = setTimeout(r, 400);
        pendingTimeouts.push(timerId);
      });
    }
  }

  async function executeAgentCommand(): Promise<void> {
    const cmd = agentCommand.value.trim();
    if (!cmd) return;
    agentCommand.value = '';
    const currentUrl = options.getCurrentUrl();

    const planMatch = /^plan[:\s]+(.+)$/i.exec(cmd);
    if (planMatch?.[1]?.trim()) {
      const goal = planMatch[1].trim();
      try {
        addAgentLog(`Hermes planning: ${goal}`);
        const multi = await hermesPlanAutomationSteps({
          goal,
          pageContext: agentDomSummary.value || undefined,
          currentUrl,
          model: options.aiModel.value,
        });
        addAgentLog(`Plan [${multi.backend}]: ${multi.steps.length} step(s)`);
        await runHermesStepPlans(multi.steps);
        return;
      } catch (error) {
        addAgentLog(`Plan failed: ${error}`);
        return;
      }
    }

    try {
      const plan = await hermesPlanAgentAction(cmd, currentUrl);
      if (plan.kind === 'ask' && plan.askPrompt?.trim()) {
        addAgentLog('Hermes → page analysis');
        await askAgentWithAllama(plan.askPrompt.trim());
        return;
      }
      if (plan.kind === 'dom' && plan.actionJson) {
        addAgentLog(plan.message ?? 'Hermes → DOM action');
        await runAgentAction(plan.actionJson);
        return;
      }
      if (plan.kind === 'none' && plan.message) {
        addAgentLog(plan.message);
        return;
      }
    } catch (error) {
      shellLog.debug('Hermes plan fallback', error);
    }

    const askMatch = /^ask[:\s]+(.+)$/i.exec(cmd);
    if (askMatch?.[1]?.trim()) {
      await askAgentWithAllama(askMatch[1].trim());
      return;
    }

    const actionJson =
      parseAgentCommandInput(cmd) ?? (cmd.startsWith('{') ? cmd : null);
    if (!actionJson) {
      addAgentLog('Use JSON or: scroll down / scroll up / get text / links / ask: question');
      return;
    }
    await runAgentAction(actionJson);
  }

  async function runAgentHermesStrategy(templateId: string): Promise<void> {
    const template = listHermesStrategyTemplates().find((t) => t.id === templateId);
    if (!template) {
      addAgentLog(`Unknown strategy: ${templateId}`);
      return;
    }
    isAgentExecuting.value = true;
    try {
      await hermesSyncAgentContext({
        currentUrl: options.getCurrentUrl(),
        tabId: options.getActiveTabLabel(),
        pageContext: agentDomSummary.value,
      });
      addAgentLog(`Strategy: ${template.name} (${template.steps.length} steps)`);
      const results = await hermesRunStrategySteps(template.steps);
      for (let i = 0; i < results.length; i++) {
        const row = results[i] as Record<string, unknown>;
        const msg =
          typeof row.message === 'string'
            ? row.message
            : `Step ${i + 1}/${results.length}`;
        const actionJson = actionJsonFromStepResult(row);
        if (actionJson) {
          addAgentLog(msg);
          await runAgentAction(actionJson);
          await new Promise((r) => setTimeout(r, 400));
        } else if (typeof row.answer === 'string' && row.answer.trim()) {
          addAgentLog(row.answer.trim().slice(0, 2000));
        }
        await new Promise((r) => {
          const timerId = setTimeout(r, 400);
          pendingTimeouts.push(timerId);
        });
      }
    } catch (error) {
      addAgentLog(`Strategy failed: ${error}`);
    } finally {
      isAgentExecuting.value = false;
    }
  }

  function onAgentPreset(actionJson: string): void {
    void runAgentAction(actionJson);
  }

  async function initSidebar(): Promise<void> {
    // Fire-and-forget IPC — avoids blocking shell paint (macOS busy cursor).
    void probeAiOnline();
    void loadIndexedMemory();
    void refreshHistory();
  }

  // Cleanup on unmount
  onUnmounted(() => {
    shellLog.info('Cleaning up resources');
    
    // Abort ongoing chat
    if (chatAbortController) {
      chatAbortController.abort();
      chatAbortController = null;
    }
    
    // Clear all pending timeouts
    for (const timerId of pendingTimeouts) {
      clearTimeout(timerId);
    }
    pendingTimeouts.length = 0;
    
    shellLog.info('Cleanup complete');
  });

  return {
    sidebarOpen,
    sidebarPanel,
    agentPanelOpen,
    aiChatHistory,
    aiChatInput,
    chatStreamBuffer,
    aiStreamMode,
    isLoading,
    aiOnline,
    aiResponse,
    indexedPages,
    historyPages,
    indexedMemoryGroups,
    historyGroups,
    filteredIndexedMemoryGroups,
    filteredHistoryGroups,
    indexedCount,
    historyCount,
    agentCommand,
    agentLog,
    agentDomSummary,
    isAgentExecuting,
    p2pRoomId,
    bookmarkSearchQuery,
    memorySearchQuery,
    filteredBookmarks,
    canAnnouncePage,
    openPanel,
    toggleSidebar,
    closeSidebar,
    toggleAgentPanel,
    agentBackToAi,
    probeAiOnline,
    loadIndexedMemory,
    refreshHistory,
    removeIndexedPage,
    clearIndexedMemory,
    clearBrowsingHistory,
    cancelChat,
    sendAiChat,
    compressCurrentDom,
    executeAgentCommand,
    runAgentHermesStrategy,
    onAgentPreset,
    initSidebar,
  };
}
