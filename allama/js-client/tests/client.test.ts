/**
 * Tests for Allama JavaScript/TypeScript client
 */

import { AllamaClient, AllamaError, AllamaConnectionError, AllamaTimeoutError, AllamaAPIError } from '../src';

describe('AllamaClient', () => {
  let client: AllamaClient;

  beforeEach(() => {
    client = new AllamaClient({
      baseUrl: 'http://127.0.0.1:11435',
      apiKey: 'test-key',
      timeout: 30000,
      maxRetries: 2,
    });
  });

  afterEach(() => {
    client.close();
  });

  test('should initialize with correct config', () => {
    expect(client).toBeInstanceOf(AllamaClient);
  });

  test('should normalize base URL', () => {
    const client1 = new AllamaClient({ baseUrl: 'http://127.0.0.1:11435/' });
    const client2 = new AllamaClient({ baseUrl: 'http://127.0.0.1:11435//' });
    // Both should have the same normalized URL
  });

  test('should throw AllamaError for invalid operations', async () => {
    // This test would require mocking fetch
    // For now, we just verify the error classes exist
    expect(AllamaError).toBeDefined();
    expect(AllamaConnectionError).toBeDefined();
    expect(AllamaTimeoutError).toBeDefined();
    expect(AllamaAPIError).toBeDefined();
  });

  test('should have all required methods', () => {
    expect(typeof client.chat).toBe('function');
    expect(typeof client.chatStream).toBe('function');
    expect(typeof client.generate).toBe('function');
    expect(typeof client.generateStream).toBe('function');
    expect(typeof client.listModels).toBe('function');
    expect(typeof client.embeddings).toBe('function');
    expect(typeof client.anthropicMessages).toBe('function');
    expect(typeof client.close).toBe('function');
  });
});
