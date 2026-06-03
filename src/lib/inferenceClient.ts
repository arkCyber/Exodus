/**
 * Exodus Browser — Tauri InferenceEngine client (`inference_*` commands).
 */

import { invoke } from '@tauri-apps/api/core';

/** Model registry entry (matches Rust `ModelInfo`). */
export type InferenceModelInfo = {
  name: string;
  path: string;
  sizeBytes: number;
  quantization: string;
  parameters: string;
  contextLength: number;
  loaded: boolean;
  backend: string;
};

/** Text generation request (matches Rust `InferenceRequest`). */
export type InferenceGenerateRequest = {
  model: string;
  prompt: string;
  maxTokens?: number | null;
  temperature?: number | null;
  topP?: number | null;
  topK?: number | null;
  repeatPenalty?: number | null;
  stop?: string[] | null;
  stream?: boolean;
};

/** Chat request (matches Rust `ChatRequest`). */
export type InferenceChatRequest = {
  model: string;
  messages: Array<{ role: string; content: string }>;
  maxTokens?: number | null;
  temperature?: number | null;
  topP?: number | null;
  stream?: boolean;
};

/** Generation / chat response (matches Rust `InferenceResponse`). */
export type InferenceGenerateResponse = {
  success: boolean;
  text?: string | null;
  error?: string | null;
  model: string;
  tokensGenerated: number;
  tokensPerSecond: number;
  totalTimeMs: number;
  promptTokens: number;
};

/** Embedding request (matches Rust `EmbeddingRequest`). */
export type InferenceEmbedRequest = {
  model: string;
  text: string;
};

/** Embedding response (matches Rust `EmbeddingResponse`). */
export type InferenceEmbedResponse = {
  success: boolean;
  embedding?: number[] | null;
  error?: string | null;
  dimensions: number;
};

/** Engine config subset for UI (Rust `InferenceConfig` uses snake_case over IPC). */
export type InferenceConfigDto = {
  enabled: boolean;
  modelPath: string;
  backendType: string;
  maxContextLength: number;
  maxTokens: number;
  temperature: number;
  topP: number;
  topK: number;
  embeddingOnly: boolean;
};

type ModelInfoDto = {
  name: string;
  path: string;
  size_bytes: number;
  quantization: string;
  parameters: string;
  context_length: number;
  loaded: boolean;
  backend: string;
};

function mapModel(dto: ModelInfoDto): InferenceModelInfo {
  return {
    name: dto.name,
    path: typeof dto.path === 'string' ? dto.path : String(dto.path),
    sizeBytes: dto.size_bytes,
    quantization: dto.quantization,
    parameters: dto.parameters,
    contextLength: dto.context_length,
    loaded: dto.loaded,
    backend: String(dto.backend),
  };
}

/**
 * List registered models in the InferenceEngine registry.
 */
export async function inferenceListModels(): Promise<InferenceModelInfo[]> {
  const rows = await invoke<ModelInfoDto[]>('inference_list_models');
  return rows.map(mapModel);
}

/**
 * Load a model by name into the inference engine.
 */
export async function inferenceLoadModel(modelName: string): Promise<void> {
  await invoke('inference_load_model', { modelName });
}

/**
 * Unload the currently loaded model.
 */
export async function inferenceUnloadModel(): Promise<void> {
  await invoke('inference_unload_model');
}

/**
 * Name of the loaded model, if any.
 */
export async function inferenceGetLoadedModel(): Promise<string | null> {
  const name = await invoke<string | null>('inference_get_loaded_model');
  return name ?? null;
}

/**
 * Engine status string (Debug format from Rust).
 */
export async function inferenceGetStatus(): Promise<string> {
  return invoke<string>('inference_get_status');
}

/**
 * Engine statistics as JSON object.
 */
export async function inferenceGetStats(): Promise<Record<string, unknown>> {
  return invoke<Record<string, unknown>>('inference_get_stats');
}

/**
 * Read inference engine configuration.
 */
export async function inferenceGetConfig(): Promise<InferenceConfigDto> {
  const c = await invoke<{
    enabled: boolean;
    model_path: string;
    backend_type: string;
    max_context_length: number;
    max_tokens: number;
    temperature: number;
    top_p: number;
    top_k: number;
    embedding_only: boolean;
  }>('inference_get_config');
  return {
    enabled: c.enabled,
    modelPath: c.model_path,
    backendType: String(c.backend_type),
    maxContextLength: c.max_context_length,
    maxTokens: c.max_tokens,
    temperature: c.temperature,
    topP: c.top_p,
    topK: c.top_k,
    embeddingOnly: c.embedding_only,
  };
}

/**
 * One-shot text generation via InferenceEngine (routes to Allama HTTP when configured).
 */
export async function inferenceGenerate(
  request: InferenceGenerateRequest,
): Promise<InferenceGenerateResponse> {
  const dto = await invoke<{
    success: boolean;
    text?: string | null;
    error?: string | null;
    model: string;
    tokens_generated: number;
    tokens_per_second: number;
    total_time_ms: number;
    prompt_tokens: number;
  }>('inference_generate', {
    request: {
      model: request.model,
      prompt: request.prompt,
      max_tokens: request.maxTokens ?? null,
      temperature: request.temperature ?? null,
      top_p: request.topP ?? null,
      top_k: request.topK ?? null,
      repeat_penalty: null,
      stop: request.stop ?? null,
      stream: request.stream ?? false,
    },
  });
  return {
    success: dto.success,
    text: dto.text,
    error: dto.error,
    model: dto.model,
    tokensGenerated: dto.tokens_generated,
    tokensPerSecond: dto.tokens_per_second,
    totalTimeMs: dto.total_time_ms,
    promptTokens: dto.prompt_tokens,
  };
}

/**
 * Chat completion via InferenceEngine.
 */
export async function inferenceChat(
  request: InferenceChatRequest,
): Promise<InferenceGenerateResponse> {
  const dto = await invoke<{
    success: boolean;
    text?: string | null;
    error?: string | null;
    model: string;
    tokens_generated: number;
    tokens_per_second: number;
    total_time_ms: number;
    prompt_tokens: number;
  }>('inference_chat', {
    request: {
      model: request.model,
      messages: request.messages,
      max_tokens: request.maxTokens ?? null,
      temperature: request.temperature ?? null,
      top_p: request.topP ?? null,
      stream: request.stream ?? false,
    },
  });
  return {
    success: dto.success,
    text: dto.text,
    error: dto.error,
    model: dto.model,
    tokensGenerated: dto.tokens_generated,
    tokensPerSecond: dto.tokens_per_second,
    totalTimeMs: dto.total_time_ms,
    promptTokens: dto.prompt_tokens,
  };
}

/**
 * Text embeddings via InferenceEngine.
 */
export async function inferenceEmbed(
  request: InferenceEmbedRequest,
): Promise<InferenceEmbedResponse> {
  const dto = await invoke<{
    success: boolean;
    embedding?: number[] | null;
    error?: string | null;
    dimensions: number;
  }>('inference_embed', {
    request: {
      model: request.model,
      text: request.text,
    },
  });
  return {
    success: dto.success,
    embedding: dto.embedding,
    error: dto.error,
    dimensions: dto.dimensions,
  };
}
