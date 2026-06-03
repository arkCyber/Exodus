/**
 * Unit tests for agent command parsing helpers.
 */

import { describe, expect, it } from 'vitest';
import {
  AGENT_PRESETS,
  agentActionReturnsValue,
  parseAgentCommandInput,
  summarizeCompressedDom,
} from './agentActions';

describe('AGENT_PRESETS', () => {
  it('contains scroll actions', () => {
    expect(AGENT_PRESETS.some((p) => p.id === 'scroll-down')).toBe(true);
    expect(AGENT_PRESETS.some((p) => p.id === 'scroll-up')).toBe(true);
  });

  it('contains content extraction actions', () => {
    expect(AGENT_PRESETS.some((p) => p.id === 'get-content')).toBe(true);
    expect(AGENT_PRESETS.some((p) => p.id === 'extract-links')).toBe(true);
  });

  it('all presets have valid action JSON', () => {
    AGENT_PRESETS.forEach((preset) => {
      expect(() => JSON.parse(JSON.stringify(preset.action))).not.toThrow();
    });
  });
});

describe('parseAgentCommandInput', () => {
  it('parses scroll down', () => {
    const json = parseAgentCommandInput('scroll down');
    expect(json).toContain('Scroll');
    expect(json).toContain('Down');
  });

  it('passes through JSON', () => {
    const raw = '{"type":"GetContent"}';
    expect(parseAgentCommandInput(raw)).toBe(raw);
  });
});

describe('agentActionReturnsValue', () => {
  it('detects GetContent', () => {
    expect(agentActionReturnsValue('{"type":"GetContent"}')).toBe(true);
  });
});

describe('summarizeCompressedDom', () => {
  it('summarizes dom json', () => {
    const summary = summarizeCompressedDom(
      JSON.stringify({
        title: 'Example',
        interactive_elements: ['#a'],
        nodes: [{}, {}],
      }),
    );
    expect(summary).toContain('Example');
    expect(summary).toContain('2 nodes');
  });
});
