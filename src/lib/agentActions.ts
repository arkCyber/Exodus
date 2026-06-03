/**
 * Exodus Browser — web agent command parsing and presets.
 */

/** Preset agent action for quick toolbar buttons. */
export type AgentPreset = {
  id: string;
  label: string;
  action: Record<string, unknown>;
};

export const AGENT_PRESETS: AgentPreset[] = [
  {
    id: 'scroll-down',
    label: 'Scroll down',
    action: { type: 'Scroll', params: { direction: 'Down', distance: 500 } },
  },
  {
    id: 'scroll-up',
    label: 'Scroll up',
    action: { type: 'Scroll', params: { direction: 'Up', distance: 500 } },
  },
  {
    id: 'get-content',
    label: 'Get page text',
    action: { type: 'GetContent' },
  },
  {
    id: 'extract-links',
    label: 'Extract links',
    action: { type: 'ExtractLinks' },
  },
];

/**
 * Parse user input into agent action JSON (natural language or raw JSON).
 */
export function parseAgentCommandInput(cmd: string): string | null {
  const trimmed = cmd.trim();
  if (!trimmed) return null;
  if (trimmed.startsWith('{')) return trimmed;

  const lower = trimmed.toLowerCase();
  if (lower.includes('scroll') && lower.includes('down')) {
    return JSON.stringify({ type: 'Scroll', params: { direction: 'Down', distance: 500 } });
  }
  if (lower.includes('scroll') && lower.includes('up')) {
    return JSON.stringify({ type: 'Scroll', params: { direction: 'Up', distance: 500 } });
  }
  if (lower.includes('link')) {
    return JSON.stringify({ type: 'ExtractLinks' });
  }
  if (lower.includes('content') || lower.includes('text')) {
    return JSON.stringify({ type: 'GetContent' });
  }
  return null;
}

/** Whether an action returns page data to display in the agent log. */
export function agentActionReturnsValue(actionJson: string): boolean {
  try {
    const parsed = JSON.parse(actionJson) as { type?: string };
    return parsed.type === 'GetContent' || parsed.type === 'ExtractLinks' || parsed.type === 'ExtractText';
  } catch {
    return false;
  }
}

/** Short summary from compressed DOM JSON returned by compress_dom. */
export function summarizeCompressedDom(json: string): string {
  try {
    const data = JSON.parse(json) as {
      title?: string;
      url?: string;
      interactive_elements?: string[];
      nodes?: unknown[];
    };
    const interactive = data.interactive_elements?.length ?? 0;
    const nodes = data.nodes?.length ?? 0;
    return `DOM: "${data.title ?? 'Untitled'}" · ${nodes} nodes · ${interactive} interactive`;
  } catch {
    return 'DOM compressed';
  }
}
