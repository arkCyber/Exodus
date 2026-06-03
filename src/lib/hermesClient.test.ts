/**
 * Exodus Browser — hermesClient unit tests (invoke wiring).
 */

import { describe, expect, it, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
  hermesAnalyzePage,
  hermesPlanAgentAction,
  hermesPlanAutomationSteps,
  hermesRunStrategySteps,
  hermesUsedAllama,
} from '$lib/hermesClient';

describe('hermesClient', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('hermesAnalyzePage maps camelCase DTO', async () => {
    vi.mocked(invoke).mockResolvedValue({
      taskId: 'task-1',
      backend: 'allama-http',
      answer: 'mock-chat reply',
      raw: { status: 'success' },
    });
    const result = await hermesAnalyzePage({
      question: 'Summarize',
      pageContext: 'Title: Test',
      model: 'gemma4-e2b',
    });
    expect(invoke).toHaveBeenCalledWith('hermes_analyze_page', {
      question: 'Summarize',
      pageContext: 'Title: Test',
      currentUrl: null,
      tabId: null,
      model: 'gemma4-e2b',
    });
    expect(result.taskId).toBe('task-1');
    expect(hermesUsedAllama(result)).toBe(true);
  });

  it('hermesPlanAutomationSteps invokes plan command', async () => {
    vi.mocked(invoke).mockResolvedValue({
      backend: 'heuristic',
      goal: 'scroll then links',
      steps: [
        {
          kind: 'dom',
          actionJson: '{"type":"Scroll","params":{"direction":"Down","distance":500}}',
        },
      ],
    });
    const plan = await hermesPlanAutomationSteps({
      goal: 'scroll then links',
      currentUrl: 'https://example.com',
    });
    expect(invoke).toHaveBeenCalledWith('hermes_plan_automation_steps', {
      goal: 'scroll then links',
      pageContext: null,
      currentUrl: 'https://example.com',
      model: null,
    });
    expect(plan.steps.length).toBeGreaterThan(0);
  });

  it('hermesPlanAgentAction invokes plan command', async () => {
    vi.mocked(invoke).mockResolvedValue({
      kind: 'dom',
      actionJson: '{"type":"Scroll","params":{"direction":"Down","distance":500}}',
      message: 'DOM action planned',
    });
    const plan = await hermesPlanAgentAction('scroll down', 'https://example.com');
    expect(invoke).toHaveBeenCalledWith('hermes_plan_agent_action', {
      command: 'scroll down',
      currentUrl: 'https://example.com',
    });
    expect(plan.kind).toBe('dom');
    expect(plan.actionJson).toContain('Scroll');
  });

  it('hermesRunStrategySteps invokes run command', async () => {
    vi.mocked(invoke).mockResolvedValue([
      { kind: 'dom', actionJson: '{"type":"Scroll"}' },
    ]);
    const results = await hermesRunStrategySteps([
      {
        id: '1',
        stepType: 'Automation',
        action: 'scroll',
        parameters: { command: 'scroll down' },
      },
    ]);
    expect(invoke).toHaveBeenCalledWith('hermes_run_strategy_steps', {
      steps: expect.any(Array),
    });
    expect(results.length).toBe(1);
  });

  it('hermesUsedAllama is false for stub backend', () => {
    expect(
      hermesUsedAllama({
        taskId: 't',
        backend: 'stub',
        answer: 'offline',
        raw: {},
      }),
    ).toBe(false);
  });
});
