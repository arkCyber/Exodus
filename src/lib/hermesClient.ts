/**
 * Exodus Browser — Hermes agent Tauri client (links UI → Rust → Allama HTTP).
 */

import { invoke } from '@tauri-apps/api/core';

/** Result of `hermes_analyze_page` (Analysis task + Allama or stub). */
export type HermesAnalyzeResult = {
  taskId: string;
  backend: string;
  answer: string;
  raw: Record<string, unknown>;
};

/** Raw DTO from Tauri (camelCase). */
type HermesAnalyzeResultDto = {
  taskId: string;
  backend: string;
  answer: string;
  raw?: Record<string, unknown>;
};

/**
 * Page Q&A via Hermes Analysis → Allama on configured port (same as sidebar AI).
 */
export async function hermesAnalyzePage(options: {
  question: string;
  pageContext?: string;
  currentUrl?: string;
  tabId?: string;
  model?: string;
}): Promise<HermesAnalyzeResult> {
  const dto = await invoke<HermesAnalyzeResultDto>('hermes_analyze_page', {
    question: options.question,
    pageContext: options.pageContext ?? null,
    currentUrl: options.currentUrl ?? null,
    tabId: options.tabId ?? null,
    model: options.model ?? null,
  });
  return {
    taskId: dto.taskId,
    backend: dto.backend,
    answer: dto.answer,
    raw: dto.raw ?? {},
  };
}

/** Whether Hermes returned a real Allama HTTP answer (not offline stub). */
export function hermesUsedAllama(result: HermesAnalyzeResult): boolean {
  return result.backend === 'allama-http' && result.answer.trim().length > 0;
}

/** Hermes → web agent bridge plan (`dom` | `ask` | `none`). */
export type HermesActionPlan = {
  kind: string;
  actionJson?: string;
  javascript?: string;
  askPrompt?: string;
  message?: string;
};

/**
 * Plan a natural-language or JSON agent command (no task created).
 */
export async function hermesPlanAgentAction(
  command: string,
  currentUrl: string,
): Promise<HermesActionPlan> {
  return invoke<HermesActionPlan>('hermes_plan_agent_action', {
    command,
    currentUrl,
  });
}

/** Multi-step automation plan from Hermes + Allama. */
export type HermesAutomationPlan = {
  backend: string;
  goal: string;
  steps: HermesActionPlan[];
  rawLlm?: string;
};

/**
 * Plan several DOM steps from a goal (Allama JSON array, or split on \" then \").
 */
export async function hermesPlanAutomationSteps(options: {
  goal: string;
  pageContext?: string;
  currentUrl: string;
  model?: string;
}): Promise<HermesAutomationPlan> {
  return invoke<HermesAutomationPlan>('hermes_plan_automation_steps', {
    goal: options.goal,
    pageContext: options.pageContext ?? null,
    currentUrl: options.currentUrl,
    model: options.model ?? null,
  });
}

/**
 * Push agent panel URL/tab/DOM summary into Hermes session context.
 */
/** Strategy step for `hermes_run_strategy_steps`. */
export type HermesStrategyStep = {
  id: string;
  stepType: string;
  action: string;
  parameters: Record<string, unknown>;
  condition?: string | null;
};

/**
 * Execute a multi-step strategy template (no persisted strategy id).
 */
export async function hermesRunStrategySteps(
  steps: HermesStrategyStep[],
): Promise<Record<string, unknown>[]> {
  return invoke<Record<string, unknown>[]>('hermes_run_strategy_steps', { steps });
}

export async function hermesSyncAgentContext(options: {
  currentUrl?: string;
  tabId?: string;
  pageContext?: string;
}): Promise<void> {
  await invoke('hermes_sync_agent_context', {
    currentUrl: options.currentUrl ?? null,
    tabId: options.tabId ?? null,
    pageContext: options.pageContext ?? null,
  });
}
