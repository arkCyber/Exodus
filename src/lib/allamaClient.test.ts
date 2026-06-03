/**
 * Tests for Allama browser HTTP client.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  ALLAMA_DEFAULT_PORT,
  allamaBaseUrl,
  allamaChat,
  allamaGenerate,
  allamaHealth,
  allamaEmbed,
  checkEmbeddingsOnline,
  streamAllamaChatCompletions,
} from './allamaClient';

describe('allamaClient', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('builds default base URL on port 11435', () => {
    expect(allamaBaseUrl()).toBe('http://127.0.0.1:11435');
    expect(allamaBaseUrl(9999)).toBe('http://127.0.0.1:9999');
  });

  it('allamaHealth returns true when /api/tags succeeds', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: true }),
    );
    await expect(allamaHealth()).resolves.toBe(true);
    expect(fetch).toHaveBeenCalledWith(
      `${allamaBaseUrl(ALLAMA_DEFAULT_PORT)}/api/tags`,
      expect.objectContaining({ method: 'GET' }),
    );
  });

  it('allamaHealth returns false on network error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));
    await expect(allamaHealth()).resolves.toBe(false);
  });

  it('allamaChat parses assistant message', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({
          message: { role: 'assistant', content: 'hello from allama' },
        }),
      }),
    );
    const text = await allamaChat([{ role: 'user', content: 'hi' }]);
    expect(text).toBe('hello from allama');
  });

  it('allamaGenerate parses response field', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ response: 'generated text' }),
      }),
    );
    const text = await allamaGenerate('prompt');
    expect(text).toBe('generated text');
  });

  it('streamAllamaChatCompletions parses SSE chunks', async () => {
    const encoder = new TextEncoder();
    const body = [
      'data: {"choices":[{"delta":{"content":"Hel"}}]}\n',
      'data: {"choices":[{"delta":{"content":"lo"}}]}\n',
      'data: [DONE]\n',
    ].join('');
    const stream = new ReadableStream({
      start(controller) {
        controller.enqueue(encoder.encode(body));
        controller.close();
      },
    });
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({ ok: true, body: stream }),
    );

    const chunks: string[] = [];
    let done = false;
    await streamAllamaChatCompletions(
      [{ role: 'user', content: 'hi' }],
      { model: 'exodus-default' },
      {
        onChunk: (c) => chunks.push(c),
        onDone: () => {
          done = true;
        },
        onError: (e) => {
          throw new Error(e);
        },
      },
    );
    expect(chunks.join('')).toBe('Hello');
    expect(done).toBe(true);
  });

  it('checkEmbeddingsOnline returns true when vector is returned', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ data: [{ embedding: [0.1, 0.2] }] }),
      }),
    );
    await expect(checkEmbeddingsOnline(11435, 'nomic-embed-text')).resolves.toBe(true);
  });

  it('allamaEmbed returns embedding vector', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ data: [{ embedding: [1, 2, 3] }] }),
      }),
    );
    await expect(allamaEmbed('hello')).resolves.toEqual([1, 2, 3]);
  });
});
