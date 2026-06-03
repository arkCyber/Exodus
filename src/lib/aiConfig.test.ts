/**
 * Tests for AI config loader.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ALLAMA_DEFAULT_PORT } from './allamaClient';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { loadAiConfig } from './aiConfig';

describe('loadAiConfig', () => {
  afterEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('returns defaults when invoke fails', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('offline'));
    const cfg = await loadAiConfig();
    expect(cfg.ai_port).toBe(ALLAMA_DEFAULT_PORT);
    expect(cfg.spawn_allama).toBe(true);
  });

  it('fills missing ai_port with default', async () => {
    vi.mocked(invoke).mockResolvedValue({
      ai_model: 'exodus-default',
      embedding_model: 'nomic-embed-text',
    });
    const cfg = await loadAiConfig();
    expect(cfg.ai_port).toBe(ALLAMA_DEFAULT_PORT);
  });
});
