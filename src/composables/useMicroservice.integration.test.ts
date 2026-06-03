import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { useMicroservice, useRagService } from './useMicroservice';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => true,
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

vi.mock('lodash-es', () => ({
  throttle: vi.fn((fn) => fn),
}));

describe('useMicroservice Integration Tests', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  describe('JSON-RPC 2.0 Protocol', () => {
    it('sends valid JSON-RPC 2.0 request', async () => {
      const mockResponse = {
        jsonrpc: '2.0',
        result: { success: true },
        id: 1,
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const service = useMicroservice({ name: 'test-service' });
      const result = await service.callMethod('test_method', { param: 'value' });

      expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
        serviceName: 'test-service',
        request: expect.stringContaining('"jsonrpc":"2.0"'),
      });
      expect(result).toEqual({ success: true });
    });

    it('handles JSON-RPC error responses', async () => {
      const mockResponse = {
        jsonrpc: '2.0',
        error: {
          code: -32601,
          message: 'Method not found',
        },
        id: 1,
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const service = useMicroservice({ name: 'test-service' });

      await expect(service.callMethod('unknown_method')).rejects.toThrow('RPC Error (-32601): Method not found');
    });

    it('generates unique request IDs', async () => {
      const mockResponse = {
        jsonrpc: '2.0',
        result: { success: true },
        id: 1,
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const service = useMicroservice({ name: 'test-service' });

      await service.callMethod('method1');
      await service.callMethod('method2');

      const calls = vi.mocked(invoke).mock.calls;
      const firstRequest = calls[0]?.[1] as { request?: string } | undefined;
      const secondRequest = calls[1]?.[1] as { request?: string } | undefined;
      if (!firstRequest?.request || !secondRequest?.request) {
        throw new Error('Expected request property in invoke args');
      }
      const parsedFirst = JSON.parse(firstRequest.request);
      const parsedSecond = JSON.parse(secondRequest.request);

      expect(parsedFirst.id).not.toBe(parsedSecond.id);
    });
  });

  describe('Retry Logic', () => {
    it('retries on failure with exponential backoff', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Connection failed'));

      const service = useMicroservice({ name: 'test-service', retries: 3 });

      const startTime = Date.now();
      await expect(service.callMethod('test_method')).rejects.toThrow();
      const endTime = Date.now();

      // Should have tried 3 times with exponential backoff
      expect(invoke).toHaveBeenCalledTimes(3);
      // Total time should be at least 100ms + 200ms = 300ms (backoff times)
      expect(endTime - startTime).toBeGreaterThanOrEqual(300);
    });

    it('succeeds on retry after initial failure', async () => {
      let callCount = 0;
      vi.mocked(invoke).mockImplementation(async () => {
        callCount++;
        if (callCount === 1) {
          throw new Error('Connection failed');
        }
        return {
          jsonrpc: '2.0',
          result: { success: true },
          id: 1,
        };
      });

      const service = useMicroservice({ name: 'test-service', retries: 3 });

      const result = await service.callMethod('test_method');

      expect(result).toEqual({ success: true });
      expect(invoke).toHaveBeenCalledTimes(2);
    });

    it('respects custom retry count', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Connection failed'));

      const service = useMicroservice({ name: 'test-service', retries: 5 });

      await expect(service.callMethod('test_method')).rejects.toThrow();

      expect(invoke).toHaveBeenCalledTimes(5);
    });
  });

  describe('State Management', () => {
    afterEach(() => {
      vi.useRealTimers();
    });

    it('sets loading state during call', async () => {
      vi.useFakeTimers();
      vi.mocked(invoke).mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(
              () =>
                resolve({
                  jsonrpc: '2.0',
                  result: { success: true },
                  id: 1,
                }),
              100,
            );
          }),
      );

      const service = useMicroservice({ name: 'test-service' });
      expect(service.loading.value).toBe(false);

      const promise = service.callMethod('test_method');
      expect(service.loading.value).toBe(true);

      await vi.advanceTimersByTimeAsync(100);
      await promise;
      expect(service.loading.value).toBe(false);
      vi.useRealTimers();
    });

    it('sets error state on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Test error'));

      const service = useMicroservice({ name: 'test-service' });

      await expect(service.callMethod('test_method')).rejects.toThrow();

      expect(service.error.value).toBe('Test error');
    });

    it('clears error state on successful call', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('First error'));

      const service = useMicroservice({ name: 'test-service', retries: 1 });

      await expect(service.callMethod('test_method')).rejects.toThrow();
      expect(service.error.value).toBe('First error');

      vi.mocked(invoke).mockResolvedValue({
        jsonrpc: '2.0',
        result: { success: true },
        id: 1,
      });

      await service.callMethod('test_method');
      expect(service.error.value).toBe(null);
    });

    it('updates data state with result', async () => {
      const mockResult = { data: 'test' };
      vi.mocked(invoke).mockResolvedValue({
        jsonrpc: '2.0',
        result: mockResult,
        id: 1,
      });

      const service = useMicroservice<{ data: string }>({ name: 'test-service' });

      await service.callMethod('test_method');

      expect(service.data.value).toEqual(mockResult);
    });
  });
});

describe('useRagService Integration Tests', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('calls store_page method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: { id: 'page-123' },
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.storePage('https://example.com', 'Example', 'Content');

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"store_page"'),
    });
    expect(result).toEqual({ id: 'page-123' });
  });

  it('calls search_pages method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: [{ url: 'https://example.com', title: 'Example' }],
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.searchPages('test query');

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"search_pages"'),
    });
    expect(result).toEqual([{ url: 'https://example.com', title: 'Example' }]);
  });

  it('calls add_bookmark method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: { id: 'bookmark-123' },
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.addBookmark('https://example.com', 'Example', 'Favorites');

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"add_bookmark"'),
    });
    expect(result).toEqual({ id: 'bookmark-123' });
  });

  it('calls list_bookmarks method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: [{ url: 'https://example.com', title: 'Example' }],
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.listBookmarks();

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"list_bookmarks"'),
    });
    expect(result).toEqual([{ url: 'https://example.com', title: 'Example' }]);
  });

  it('calls record_visit method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: { id: 'visit-123' },
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.recordVisit('https://example.com', 'Example');

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"record_visit"'),
    });
    expect(result).toEqual({ id: 'visit-123' });
  });

  it('calls search_visits method correctly', async () => {
    vi.mocked(invoke).mockResolvedValue({
      jsonrpc: '2.0',
      result: [{ url: 'https://example.com', title: 'Example' }],
      id: 1,
    });

    const ragService = useRagService();
    const result = await ragService.searchVisits('test query');

    expect(invoke).toHaveBeenCalledWith('invoke_microservice', {
      serviceName: 'rag-service',
      request: expect.stringContaining('"method":"search_visits"'),
    });
    expect(result).toEqual([{ url: 'https://example.com', title: 'Example' }]);
  });
});
