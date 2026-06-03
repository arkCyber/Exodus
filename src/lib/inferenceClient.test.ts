/**
 * InferenceEngine Tauri client unit tests (invoke wiring).
 */

import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import {
  inferenceChat,
  inferenceGenerate,
  inferenceGetLoadedModel,
  inferenceListModels,
} from '$lib/inferenceClient';

describe('inferenceClient', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('inferenceListModels maps snake_case DTO', async () => {
    vi.mocked(invoke).mockResolvedValue([
      {
        name: 'exodus-default',
        path: '/models/x.gguf',
        size_bytes: 1000,
        quantization: 'q4',
        parameters: '7B',
        context_length: 2048,
        loaded: true,
        backend: 'Allama',
      },
    ]);
    const models = await inferenceListModels();
    expect(invoke).toHaveBeenCalledWith('inference_list_models');
    expect(models[0].name).toBe('exodus-default');
    expect(models[0].sizeBytes).toBe(1000);
    expect(models[0].loaded).toBe(true);
  });

  it('inferenceGetLoadedModel invokes command', async () => {
    vi.mocked(invoke).mockResolvedValue('gemma4-e2b');
    const name = await inferenceGetLoadedModel();
    expect(invoke).toHaveBeenCalledWith('inference_get_loaded_model');
    expect(name).toBe('gemma4-e2b');
  });

  it('inferenceGenerate passes snake_case request', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      text: 'hi',
      model: 'm',
      tokens_generated: 2,
      tokens_per_second: 10,
      total_time_ms: 50,
      prompt_tokens: 1,
    });
    const res = await inferenceGenerate({
      model: 'exodus-default',
      prompt: 'Hello',
      maxTokens: 8,
    });
    expect(invoke).toHaveBeenCalledWith('inference_generate', {
      request: expect.objectContaining({
        model: 'exodus-default',
        prompt: 'Hello',
        max_tokens: 8,
      }),
    });
    expect(res.success).toBe(true);
    expect(res.text).toBe('hi');
  });

  it('inferenceChat invokes chat command', async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      text: 'reply',
      model: 'm',
      tokens_generated: 1,
      tokens_per_second: 5,
      total_time_ms: 20,
      prompt_tokens: 1,
    });
    await inferenceChat({
      model: 'm',
      messages: [{ role: 'user', content: 'Hi' }],
    });
    expect(invoke).toHaveBeenCalledWith('inference_chat', {
      request: expect.objectContaining({
        messages: [{ role: 'user', content: 'Hi' }],
      }),
    });
  });
});
