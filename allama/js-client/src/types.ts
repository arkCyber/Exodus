/**
 * Type definitions for Allama client
 */

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ChatOptions {
  model: string;
  messages: Message[];
  stream?: boolean;
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  [key: string]: any;
}

export interface GenerateOptions {
  model: string;
  prompt: string;
  stream?: boolean;
  temperature?: number;
  max_tokens?: number;
  top_p?: number;
  [key: string]: any;
}

export interface EmbeddingsOptions {
  model: string;
  input: string | string[];
  [key: string]: any;
}

export interface AnthropicMessagesOptions {
  model: string;
  messages: Message[];
  max_tokens?: number;
  stream?: boolean;
  system?: string;
  temperature?: number;
  top_p?: number;
  top_k?: number;
  stop_sequences?: string[];
  tools?: any[];
  [key: string]: any;
}

export interface ClientConfig {
  baseUrl?: string;
  apiKey?: string;
  timeout?: number;
  maxRetries?: number;
}

export interface ChatResponse {
  message: {
    role: string;
    content: string;
  };
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface GenerateResponse {
  model: string;
  response: string;
  done: boolean;
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export interface EmbeddingsResponse {
  object: string;
  data: Array<{
    object: string;
    embedding: number[];
    index: number;
  }>;
  model: string;
  usage: {
    prompt_tokens: number;
    total_tokens: number;
  };
}

export interface ModelsResponse {
  models: Array<{
    name: string;
    modified_at: string;
    size: number;
  }>;
}
