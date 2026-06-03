/**
 * Allama client implementation
 */

import fetch from 'node-fetch';
import {
  AllamaError,
  AllamaConnectionError,
  AllamaTimeoutError,
  AllamaAPIError,
} from './exceptions';
import type {
  ClientConfig,
  ChatOptions,
  GenerateOptions,
  EmbeddingsOptions,
  AnthropicMessagesOptions,
  ChatResponse,
  GenerateResponse,
  EmbeddingsResponse,
  ModelsResponse,
} from './types';

export class AllamaClient {
  private baseUrl: string;
  private apiKey?: string;
  private timeout: number;
  private maxRetries: number;
  private headers: Record<string, string>;

  constructor(config: ClientConfig = {}) {
    this.baseUrl = (config.baseUrl || 'http://127.0.0.1:11435').replace(/\/$/, '');
    this.apiKey = config.apiKey;
    this.timeout = config.timeout || 60000;
    this.maxRetries = config.maxRetries || 3;
    this.headers = {
      'Content-Type': 'application/json',
    };

    if (this.apiKey) {
      this.headers['Authorization'] = `Bearer ${this.apiKey}`;
    }
  }

  private async request<T>(
    method: string,
    endpoint: string,
    data?: any,
    stream = false
  ): Promise<T> {
    const response = await this._fetch(method, endpoint, data);
    if (stream) {
      return this.streamResponse<T>(response) as any;
    } else {
      return await response.json() as T;
    }
  }

  private async _fetch(
    method: string,
    endpoint: string,
    data?: any
  ): Promise<any> {
    const url = `${this.baseUrl}${endpoint}`;
    const options: any = {
      method,
      headers: this.headers,
      body: data ? JSON.stringify(data) : undefined,
    };

    let lastError: Error | null = null;

    for (let attempt = 0; attempt < this.maxRetries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);
        options.signal = controller.signal;

        const response = await fetch(url, options);
        clearTimeout(timeoutId);

        if (!response.ok) {
          let errorData: any;
          try {
            errorData = await response.json();
          } catch {
            errorData = { raw: await response.text() };
          }

          throw new AllamaAPIError(
            `API error: ${response.status}`,
            response.status,
            errorData
          );
        }

        return response;
      } catch (error: any) {
        lastError = error;

        if (error instanceof AllamaAPIError) {
          throw error; // Don't retry API errors
        }

        if (error.name === 'AbortError') {
          lastError = new AllamaTimeoutError('Request timeout');
        } else if (error.name === 'TypeError' && error.message.includes('fetch')) {
          lastError = new AllamaConnectionError(`Connection error: ${error.message}`);
        }

        if (attempt < this.maxRetries - 1) {
          await this.delay(2 ** attempt * 1000); // Exponential backoff
          continue;
        }

        throw lastError;
      }
    }

    throw lastError || new AllamaError('Unknown error');
  }

  private async *streamResponse<T>(response: any): AsyncIterableIterator<T> {
    const reader = response.body?.getReader();
    if (!reader) throw new AllamaError('No response body');

    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const data = line.slice(6);
          if (data === '[DONE]') return;
          try {
            yield JSON.parse(data) as T;
          } catch {
            // Skip invalid JSON
          }
        }
      }
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Chat completion (OpenAI-compatible)
   */
  async chat(options: ChatOptions): Promise<ChatResponse> {
    const data = {
      ...options,
      stream: false,
    };
    return this.request<ChatResponse>('POST', '/v1/chat/completions', data, false);
  }

  /**
   * Streaming chat completion
   */
  async chatStream(options: ChatOptions): Promise<AsyncIterableIterator<ChatResponse>> {
    const data = {
      ...options,
      stream: true,
    };
    return this.streamResponse<ChatResponse>(await this._fetch('POST', '/v1/chat/completions', data));
  }

  /**
   * Text generation (Ollama-compatible)
   */
  async generate(options: GenerateOptions): Promise<GenerateResponse> {
    const data = {
      model: options.model,
      prompt: options.prompt,
      stream: false,
      options: {
        temperature: options.temperature || 0.7,
        num_predict: options.max_tokens || 512,
        top_p: options.top_p || 0.9,
      },
    };
    return this.request<GenerateResponse>('POST', '/api/generate', data, false);
  }

  /**
   * Streaming text generation
   */
  async generateStream(options: GenerateOptions): Promise<AsyncIterableIterator<GenerateResponse>> {
    const data = {
      model: options.model,
      prompt: options.prompt,
      stream: true,
      options: {
        temperature: options.temperature || 0.7,
        num_predict: options.max_tokens || 512,
        top_p: options.top_p || 0.9,
      },
    };
    return this.streamResponse<GenerateResponse>(await this._fetch('POST', '/api/generate', data));
  }

  /**
   * List available models
   */
  async listModels(): Promise<ModelsResponse> {
    return this.request<ModelsResponse>('GET', '/api/tags', undefined, false);
  }

  /**
   * Get embeddings (OpenAI-compatible)
   */
  async embeddings(options: EmbeddingsOptions): Promise<EmbeddingsResponse> {
    const data = {
      model: options.model,
      input: options.input,
    };

    return this.request<EmbeddingsResponse>('POST', '/v1/embeddings', data, false);
  }

  /**
   * Anthropic API compatibility (/v1/messages)
   */
  async anthropicMessages(options: AnthropicMessagesOptions): Promise<any> {
    const data = {
      ...options,
      stream: false,
    };
    return this.request('POST', '/v1/messages', data, false);
  }

  /**
   * Anthropic API streaming
   */
  async anthropicMessagesStream(options: AnthropicMessagesOptions): Promise<AsyncIterableIterator<any>> {
    const data = {
      ...options,
      stream: true,
    };
    return this.streamResponse<any>(await this._fetch('POST', '/v1/messages', data));
  }

  /**
   * Close the client (no-op for HTTP client, but provided for API compatibility)
   */
  close(): void {
    // No-op for HTTP client
  }
}
