/**
 * Exodus Browser — Hermes strategy templates for the agent panel.
 */

import { parseAgentCommandInput } from '$lib/agentActions';

/** Single step passed to `hermes_run_strategy_steps` (matches Rust StrategyStep). */
export type HermesStrategyStep = {
  id: string;
  stepType: string;
  action: string;
  parameters: Record<string, unknown>;
  condition?: string | null;
};

/** Built-in multi-step template. */
export type HermesStrategyTemplate = {
  id: string;
  name: string;
  description: string;
  steps: HermesStrategyStep[];
};

const STORAGE_KEY = 'exodus.hermes.strategy.templates';

/** Aerospace-grade default templates (English labels per project convention). */
export const HERMES_STRATEGY_TEMPLATES: HermesStrategyTemplate[] = [
  {
    id: 'page-scan',
    name: 'Page scan',
    description: 'Scroll, extract links, capture page text',
    steps: [
      {
        id: 'scan-1',
        stepType: 'Automation',
        action: 'Scroll down',
        parameters: { command: 'scroll down' },
      },
      {
        id: 'scan-2',
        stepType: 'DataExtraction',
        action: 'Extract links',
        parameters: { extract: 'links' },
      },
      {
        id: 'scan-3',
        stepType: 'DataExtraction',
        action: 'Get page text',
        parameters: { extract: 'content' },
      },
    ],
  },
  {
    id: 'page-audit-ai',
    name: 'Page audit (AI)',
    description: 'Scroll, read text, Allama summary',
    steps: [
      {
        id: 'audit-1',
        stepType: 'Automation',
        action: 'Scroll down',
        parameters: { command: 'scroll down' },
      },
      {
        id: 'audit-2',
        stepType: 'DataExtraction',
        action: 'Get page text',
        parameters: { extract: 'content' },
      },
      {
        id: 'audit-3',
        stepType: 'Analysis',
        action: 'Summarize page',
        parameters: {
          use_allama: 'true',
          prompt: 'Summarize the main points of this page in 3 bullet points.',
        },
      },
    ],
  },
  {
    id: 'quick-scroll',
    name: 'Quick scroll',
    description: 'Scroll down twice',
    steps: [
      {
        id: 'qs-1',
        stepType: 'Automation',
        action: 'Scroll down',
        parameters: { command: 'scroll down' },
      },
      {
        id: 'qs-2',
        stepType: 'Automation',
        action: 'Scroll down again',
        parameters: { command: 'scroll down' },
      },
    ],
  },
];

/** Built-in template ids (not deletable from localStorage). */
export const BUILTIN_HERMES_STRATEGY_IDS = new Set(
  HERMES_STRATEGY_TEMPLATES.map((t) => t.id),
);

/**
 * Load user-saved templates from localStorage (merged with builtins by id).
 */
export function loadSavedHermesTemplates(): HermesStrategyTemplate[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as HermesStrategyTemplate[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * Persist custom templates (builtins are not written).
 */
export function saveCustomHermesTemplates(custom: HermesStrategyTemplate[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(custom));
  } catch {
    /* ignore quota */
  }
}

/**
 * Whether a template id is a built-in (not user-saved).
 */
export function isBuiltinHermesTemplate(id: string): boolean {
  return BUILTIN_HERMES_STRATEGY_IDS.has(id);
}

/**
 * Slug for custom template ids (`custom-my-flow`).
 */
export function slugifyHermesTemplateId(name: string): string {
  const base = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 48);
  return `custom-${base || 'strategy'}-${Date.now().toString(36)}`;
}

/**
 * Build strategy steps from the agent command input (NL, JSON, plan:, ask:).
 */
export function buildStepsFromAgentCommand(cmd: string): HermesStrategyStep[] {
  const trimmed = cmd.trim();
  if (!trimmed) {
    throw new Error('Enter a command before saving a strategy');
  }

  const askMatch = /^ask[:\s]+(.+)$/i.exec(trimmed);
  if (askMatch?.[1]?.trim()) {
    return [
      {
        id: 'step-1',
        stepType: 'Analysis',
        action: 'Ask AI',
        parameters: {
          use_allama: 'true',
          prompt: askMatch[1].trim(),
        },
      },
    ];
  }

  const planMatch = /^plan[:\s]+(.+)$/i.exec(trimmed);
  if (planMatch?.[1]?.trim()) {
    const parts = planMatch[1]
      .split(/\s+then\s+/i)
      .map((p) => p.trim())
      .filter(Boolean);
    if (parts.length === 0) {
      throw new Error('plan: needs at least one step');
    }
    return parts.map((part, i) => ({
      id: `step-${i + 1}`,
      stepType: 'Automation',
      action: part,
      parameters: { command: part },
    }));
  }

  const lower = trimmed.toLowerCase();
  if (lower.includes('link')) {
    return [
      {
        id: 'step-1',
        stepType: 'DataExtraction',
        action: 'Extract links',
        parameters: { extract: 'links' },
      },
    ];
  }
  if (lower.includes('content') || lower.includes('text')) {
    return [
      {
        id: 'step-1',
        stepType: 'DataExtraction',
        action: 'Get page text',
        parameters: { extract: 'content' },
      },
    ];
  }

  const actionJson =
    parseAgentCommandInput(trimmed) ?? (trimmed.startsWith('{') ? trimmed : null);
  if (actionJson) {
    return [
      {
        id: 'step-1',
        stepType: 'Automation',
        action: trimmed.slice(0, 80),
        parameters: { action_json: actionJson },
      },
    ];
  }

  return [
    {
      id: 'step-1',
      stepType: 'Automation',
      action: trimmed.slice(0, 80),
      parameters: { command: trimmed },
    },
  ];
}

/**
 * Create a custom template from the current agent command.
 */
export function buildCustomTemplateFromCommand(
  name: string,
  description: string,
  command: string,
): HermesStrategyTemplate {
  const steps = buildStepsFromAgentCommand(command);
  return {
    id: slugifyHermesTemplateId(name),
    name: name.trim() || 'Custom strategy',
    description: description.trim() || `Saved from: ${command.slice(0, 60)}`,
    steps,
  };
}

/**
 * User-only templates (excludes builtins).
 */
export function getCustomHermesTemplates(): HermesStrategyTemplate[] {
  return loadSavedHermesTemplates().filter((t) => !isBuiltinHermesTemplate(t.id));
}

/**
 * Insert or replace a custom template in localStorage.
 */
export function upsertCustomHermesTemplate(template: HermesStrategyTemplate): void {
  if (isBuiltinHermesTemplate(template.id)) {
    throw new Error('Cannot overwrite built-in strategy templates');
  }
  const custom = getCustomHermesTemplates().filter((t) => t.id !== template.id);
  custom.push(template);
  saveCustomHermesTemplates(custom);
}

/**
 * Remove a custom template by id.
 */
export function deleteCustomHermesTemplate(id: string): boolean {
  if (isBuiltinHermesTemplate(id)) {
    return false;
  }
  const next = getCustomHermesTemplates().filter((t) => t.id !== id);
  if (next.length === getCustomHermesTemplates().length) {
    return false;
  }
  saveCustomHermesTemplates(next);
  return true;
}

/**
 * All templates: builtins + custom (custom overrides same id).
 */
export function listHermesStrategyTemplates(): HermesStrategyTemplate[] {
  const custom = loadSavedHermesTemplates();
  const byId = new Map<string, HermesStrategyTemplate>();
  for (const t of HERMES_STRATEGY_TEMPLATES) {
    byId.set(t.id, t);
  }
  for (const t of custom) {
    byId.set(t.id, t);
  }
  return Array.from(byId.values());
}

/**
 * Extract `actionJson` from a Hermes step result object for DOM execution.
 */
export function actionJsonFromStepResult(step: Record<string, unknown>): string | undefined {
  const direct = step.actionJson;
  if (typeof direct === 'string' && direct.length > 0) {
    return direct;
  }
  return undefined;
}
