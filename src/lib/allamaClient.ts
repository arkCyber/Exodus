/**
 * Exodus Browser — browser-side Allama HTTP client (Ollama-compatible API on port 11435).
 */

/** Default Allama port (replaces Ollama 11434). */
export const ALLAMA_DEFAULT_PORT = 11435;

export type AllamaChatMessage = {
  role: 'system' | 'user' | 'assistant' | string;
  content: string;
};

export type AllamaChatOptions = {
  port?: number;
  model?: string;
  maxTokens?: number;
  temperature?: number;
  signal?: AbortSignal;
};

/** Callbacks for OpenAI-style SSE streaming from `/v1/chat/completions`. */
export type AllamaStreamCallbacks = {
  onChunk: (content: string) => void;
  onDone: () => void;
  onError: (message: string) => void;
};

/**
 * Base URL for Allama HTTP (`http://127.0.0.1:{port}`).
 */
export function allamaBaseUrl(port: number = ALLAMA_DEFAULT_PORT): string {
  return `http://127.0.0.1:${port}`;
}

/**
 * Probe Allama via `GET /api/tags`.
 */
export async function allamaHealth(port: number = ALLAMA_DEFAULT_PORT): Promise<boolean> {
  try {
    const res = await fetch(`${allamaBaseUrl(port)}/api/tags`, {
      method: 'GET',
      signal: AbortSignal.timeout(3000),
    });
    return res.ok;
  } catch {
    return false;
  }
}

/**
 * Non-streaming chat via `POST /api/chat`.
 */
export async function allamaChat(
  messages: AllamaChatMessage[],
  options: AllamaChatOptions = {},
): Promise<string> {
  const port = options.port ?? ALLAMA_DEFAULT_PORT;
  const body: Record<string, unknown> = {
    model: options.model ?? 'exodus-default',
    messages,
    stream: false,
  };
  if (options.maxTokens != null) {
    body.options = { num_predict: options.maxTokens };
  }
  if (options.temperature != null) {
    body.options = {
      ...(body.options as object),
      temperature: options.temperature,
    };
  }

  const res = await fetch(`${allamaBaseUrl(port)}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
    signal: options.signal,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Allama chat HTTP ${res.status}: ${text}`);
  }
  const data = (await res.json()) as { message?: { content?: string } };
  return data.message?.content ?? '';
}

/**
 * Non-streaming generate via `POST /api/generate`.
 */
export async function allamaGenerate(
  prompt: string,
  options: AllamaChatOptions = {},
): Promise<string> {
  const port = options.port ?? ALLAMA_DEFAULT_PORT;
  const res = await fetch(`${allamaBaseUrl(port)}/api/generate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      model: options.model ?? 'exodus-default',
      prompt,
      stream: false,
    }),
    signal: options.signal,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Allama generate HTTP ${res.status}: ${text}`);
  }
  const data = (await res.json()) as { response?: string };
  return data.response ?? '';
}

/**
 * Probe embeddings via `POST /v1/embeddings`.
 */
export async function checkEmbeddingsOnline(
  port: number = ALLAMA_DEFAULT_PORT,
  model = 'nomic-embed-text',
): Promise<boolean> {
  try {
    const res = await fetch(`${allamaBaseUrl(port)}/v1/embeddings`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ model, input: 'ping' }),
      signal: AbortSignal.timeout(4000),
    });
    if (!res.ok) return false;
    const data = (await res.json()) as { data?: Array<{ embedding?: number[] }> };
    return (data.data?.[0]?.embedding?.length ?? 0) > 0;
  } catch {
    return false;
  }
}

/**
 * Fetch embedding vector for semantic search / memory indexing.
 */
export async function allamaEmbed(
  text: string,
  options: { port?: number; model?: string } = {},
): Promise<number[]> {
  const port = options.port ?? ALLAMA_DEFAULT_PORT;
  const model = options.model ?? 'nomic-embed-text';
  const res = await fetch(`${allamaBaseUrl(port)}/v1/embeddings`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model, input: text.slice(0, 8000) }),
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`Allama embed HTTP ${res.status}: ${body}`);
  }
  const data = (await res.json()) as { data?: Array<{ embedding?: number[] }> };
  const vec = data.data?.[0]?.embedding;
  if (!vec?.length) throw new Error('Allama embed: empty vector');
  return vec;
}

/**
 * Stream chat via `POST /v1/chat/completions` (SSE, same shape as Tauri `ai_chat_stream`).
 */
export async function streamAllamaChatCompletions(
  messages: AllamaChatMessage[],
  options: AllamaChatOptions & { model: string },
  callbacks: AllamaStreamCallbacks,
): Promise<void> {
  const port = options.port ?? ALLAMA_DEFAULT_PORT;
  const url = `${allamaBaseUrl(port)}/v1/chat/completions`;
  let res: Response;
  try {
    res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: options.model,
        messages,
        stream: true,
      }),
      signal: options.signal,
    });
  } catch (e) {
    callbacks.onError(e instanceof Error ? e.message : String(e));
    return;
  }

  if (!res.ok) {
    const text = await res.text().catch(() => '');
    callbacks.onError(`Allama HTTP ${res.status}: ${text}`);
    return;
  }

  const reader = res.body?.getReader();
  if (!reader) {
    callbacks.onError('Allama stream: empty response body');
    return;
  }

  const decoder = new TextDecoder();
  let buffer = '';

  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });

      for (;;) {
        const lineEnd = buffer.indexOf('\n');
        if (lineEnd === -1) break;
        const line = buffer.slice(0, lineEnd).trim();
        buffer = buffer.slice(lineEnd + 1);
        if (!line.startsWith('data: ')) continue;
        const data = line.slice(6).trim();
        if (data === '[DONE]') {
          callbacks.onDone();
          return;
        }
        try {
          const parsed = JSON.parse(data) as {
            choices?: Array<{ delta?: { content?: string } }>;
          };
          const content = parsed.choices?.[0]?.delta?.content;
          if (content) callbacks.onChunk(content);
        } catch {
          /* ignore malformed SSE lines */
        }
      }
    }
    callbacks.onDone();
  } catch (e) {
    callbacks.onError(e instanceof Error ? e.message : String(e));
  }
}
