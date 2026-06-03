import { ref, shallowRef } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

/**
 * Simple throttle implementation
 */
function throttle<T extends (...args: any[]) => any>(func: T, wait: number): T {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let previous = 0;
  
  return ((...args: Parameters<T>) => {
    const now = Date.now();
    const remaining = wait - (now - previous);
    
    if (remaining <= 0 || remaining > wait) {
      if (timeout) {
        clearTimeout(timeout);
        timeout = null;
      }
      previous = now;
      func(...args);
    } else if (!timeout) {
      timeout = setTimeout(() => {
        previous = Date.now();
        timeout = null;
        func(...args);
      }, remaining);
    }
  }) as T;
}

/**
 * JSON-RPC 2.0 Request
 */
interface JsonRpcRequest {
  jsonrpc: '2.0';
  method: string;
  params?: any;
  id: string | number;
}

/**
 * JSON-RPC 2.0 Response
 */
interface JsonRpcResponse<T = any> {
  jsonrpc: '2.0';
  result?: T;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  id: string | number;
}

/**
 * Microservice configuration
 */
interface MicroserviceConfig {
  name: string;
  socketPath?: string;
  httpUrl?: string;
  timeout?: number;
  retries?: number;
}

/**
 * Composable for interacting with microservices
 * Implements JSON-RPC 2.0 client with retry logic
 */
export function useMicroservice<T = any>(config: MicroserviceConfig) {
  const data = shallowRef<T | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let requestId = 0;

  /**
   * Generate unique request ID
   */
  function generateId(): number {
    return ++requestId;
  }

  /**
   * Call a microservice method via Tauri command
   */
  async function callMethod<R = any>(method: string, params?: any): Promise<R> {
    loading.value = true;
    error.value = null;

    const maxRetries = config.retries || 3;
    let lastError: Error | null = null;

    try {
      for (let attempt = 0; attempt < maxRetries; attempt++) {
        try {
          const request: JsonRpcRequest = {
            jsonrpc: '2.0',
            method,
            params,
            id: generateId(),
          };

          const response = await invoke<JsonRpcResponse<R>>('invoke_microservice', {
            serviceName: config.name,
            request: JSON.stringify(request),
          });

          if (response.error) {
            throw new Error(`RPC Error (${response.error.code}): ${response.error.message}`);
          }

          if (response.result === undefined) {
            throw new Error('Invalid response: result is undefined');
          }

          data.value = response.result as T;
          return response.result;
        } catch (e) {
          lastError = e as Error;
          console.error(`Microservice call failed (attempt ${attempt + 1}/${maxRetries}):`, e);

          if (attempt < maxRetries - 1) {
            await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempt) * 100));
          }
        }
      }

      error.value = lastError?.message || 'Unknown error';
      throw lastError || new Error('Microservice call failed');
    } finally {
      loading.value = false;
    }
  }

  /**
   * Listen to microservice events with throttling
   * @param eventName - The event name to listen to
   * @param callback - Callback function (will be throttled to 16ms for 60FPS)
   */
  function listenThrottled<K>(eventName: string, callback: (payload: K) => void) {
    const throttledCallback = throttle(callback, 16); // 60 FPS rate limiting
    
    return listen<K>(eventName, (event) => {
      throttledCallback(event.payload as K);
    });
  }

  return {
    data,
    loading,
    error,
    callMethod,
    listenThrottled,
  };
}

/**
 * Composable for RAG service
 */
export function useRagService() {
  const service = useMicroservice({
    name: 'rag-service',
    socketPath: '/tmp/exodus-services/rag-service.sock',
  });

  return {
    ...service,
    async storePage(url: string, title: string, content: string) {
      return service.callMethod('store_page', { url, title, content });
    },
    async searchPages(query: string) {
      return service.callMethod('search_pages', { query });
    },
    async addBookmark(url: string, title: string, folder?: string) {
      return service.callMethod('add_bookmark', { url, title, folder });
    },
    async listBookmarks() {
      return service.callMethod('list_bookmarks');
    },
    async recordVisit(url: string, title: string) {
      return service.callMethod('record_visit', { url, title });
    },
    async searchVisits(query: string) {
      return service.callMethod('search_visits', { query });
    },
  };
}

/**
 * Composable for Servo rendering status
 * Uses shallowRef to avoid deep reactivity overhead
 */
export function useServoStatus() {
  const loadingProgress = ref(0);
  const rendering = ref(false);

  return {
    loadingProgress,
    rendering,
  };
}
