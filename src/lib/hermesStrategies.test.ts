/**
 * Hermes strategy template tests (builtins + localStorage custom save).
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  actionJsonFromStepResult,
  buildCustomTemplateFromCommand,
  buildStepsFromAgentCommand,
  deleteCustomHermesTemplate,
  getCustomHermesTemplates,
  HERMES_STRATEGY_TEMPLATES,
  isBuiltinHermesTemplate,
  listHermesStrategyTemplates,
  upsertCustomHermesTemplate,
} from '$lib/hermesStrategies';

const STORAGE_KEY = 'exodus.hermes.strategy.templates';

function mockLocalStorage() {
  const store: Record<string, string> = {};
  const ls = {
    getItem: (k: string) => store[k] ?? null,
    setItem: (k: string, v: string) => {
      store[k] = v;
    },
    removeItem: (k: string) => {
      delete store[k];
    },
    clear: () => {
      for (const k of Object.keys(store)) delete store[k];
    },
    get length() {
      return Object.keys(store).length;
    },
    key: (_i: number) => null,
  };
  vi.stubGlobal('localStorage', ls);
  return store;
}

describe('hermesStrategies', () => {
  beforeEach(() => {
    mockLocalStorage();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('builtins include page-scan', () => {
    expect(HERMES_STRATEGY_TEMPLATES.some((t) => t.id === 'page-scan')).toBe(true);
    expect(isBuiltinHermesTemplate('page-scan')).toBe(true);
  });

  it('listHermesStrategyTemplates returns builtins', () => {
    const all = listHermesStrategyTemplates();
    expect(all.length).toBeGreaterThanOrEqual(3);
  });

  it('actionJsonFromStepResult reads actionJson', () => {
    const json = actionJsonFromStepResult({
      actionJson: '{"type":"Scroll"}',
      kind: 'dom',
    });
    expect(json).toContain('Scroll');
  });

  it('buildStepsFromAgentCommand handles plan: and ask:', () => {
    const plan = buildStepsFromAgentCommand('plan: scroll down then links');
    expect(plan.length).toBe(2);
    expect(plan[0].parameters.command).toBe('scroll down');

    const ask = buildStepsFromAgentCommand('ask: what is this page?');
    expect(ask[0].stepType).toBe('Analysis');
  });

  it('upsert and delete custom templates in localStorage', () => {
    const tpl = buildCustomTemplateFromCommand('Test flow', 'desc', 'scroll down');
    upsertCustomHermesTemplate(tpl);
    expect(getCustomHermesTemplates().some((t) => t.id === tpl.id)).toBe(true);
    expect(listHermesStrategyTemplates().some((t) => t.id === tpl.id)).toBe(true);

    expect(deleteCustomHermesTemplate(tpl.id)).toBe(true);
    expect(getCustomHermesTemplates().some((t) => t.id === tpl.id)).toBe(false);
    expect(deleteCustomHermesTemplate('page-scan')).toBe(false);
  });

  it('cannot upsert builtin id', () => {
    expect(() =>
      upsertCustomHermesTemplate({
        id: 'page-scan',
        name: 'Hack',
        description: '',
        steps: [],
      }),
    ).toThrow();
  });

  it('persists via localStorage key', () => {
    const tpl = buildCustomTemplateFromCommand('Persist', '', 'links');
    upsertCustomHermesTemplate(tpl);
    const raw = localStorage.getItem(STORAGE_KEY);
    expect(raw).toContain('Persist');
  });
});
