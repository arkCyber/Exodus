/**
 * Exodus Browser — RAG client tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as ragClient from './ragClient';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('RAG Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should start RAG service', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('/tmp/rag-service.sock');
    
    const result = await ragClient.startRagService('/tmp/data');
    expect(result).toBe('/tmp/rag-service.sock');
    expect(invoke).toHaveBeenCalledWith('rag_service_start', { dataDir: '/tmp/data' });
  });

  it('should stop RAG service', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue(true);
    
    const result = await ragClient.stopRagService();
    expect(result).toBe(true);
    expect(invoke).toHaveBeenCalledWith('rag_service_stop');
  });

  it('should store a page', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('page-id-123');
    
    const result = await ragClient.storeRagPage('https://example.com', 'Example', 'Content');
    expect(result).toBe('page-id-123');
    expect(invoke).toHaveBeenCalledWith('rag_store_page', {
      url: 'https://example.com',
      title: 'Example',
      content: 'Content'
    });
  });

  it('should search pages by keyword', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockPages = [
      {
        id: '1',
        url: 'https://example.com',
        title: 'Example',
        content: 'Example content',
        timestamp: '2026-01-01T00:00:00Z'
      }
    ];
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockPages));
    
    const result = await ragClient.searchRagPages('example');
    expect(result).toEqual(mockPages);
    expect(invoke).toHaveBeenCalledWith('rag_search_pages', { query: 'example' });
  });

  it('should perform semantic search', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResults = [
      {
        page: {
          id: '1',
          url: 'https://example.com',
          title: 'Example',
          content: 'Example content',
          timestamp: '2026-01-01T00:00:00Z',
          embedding: [0.1, 0.2, 0.3]
        },
        score: 0.95,
        matchedText: 'Example content...'
      }
    ];
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockResults));
    
    const embedding = [0.1, 0.2, 0.3];
    const result = await ragClient.searchRagSemantic(embedding);
    expect(result).toEqual(mockResults);
    expect(invoke).toHaveBeenCalledWith('rag_search_semantic', { queryEmbedding: embedding });
  });

  it('should perform hybrid search', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const mockResults = [
      {
        page: {
          id: '1',
          url: 'https://example.com',
          title: 'Example',
          content: 'Example content',
          timestamp: '2026-01-01T00:00:00Z'
        },
        score: 0.85,
        matchedText: 'Example content...'
      }
    ];
    vi.mocked(invoke).mockResolvedValue(JSON.stringify(mockResults));
    
    const result = await ragClient.searchRagHybrid('example');
    expect(result).toEqual(mockResults);
    expect(invoke).toHaveBeenCalledWith('rag_search_hybrid', { 
      query: 'example',
      queryEmbedding: undefined
    });
  });

  it('should add bookmark', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('bookmark-id-456');
    
    const result = await ragClient.addRagBookmark('https://example.com', 'Example', 'Work');
    expect(result).toBe('bookmark-id-456');
    expect(invoke).toHaveBeenCalledWith('rag_add_bookmark', {
      url: 'https://example.com',
      title: 'Example',
      folder: 'Work'
    });
  });

  it('should record visit', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockResolvedValue('visit-id-789');
    
    const result = await ragClient.recordRagVisit('https://example.com', 'Example');
    expect(result).toBe('visit-id-789');
    expect(invoke).toHaveBeenCalledWith('rag_record_visit', {
      url: 'https://example.com',
      title: 'Example'
    });
  });

  // Input validation tests
  it('should reject empty URL in storeRagPage', async () => {
    await expect(ragClient.storeRagPage('', 'Title', 'Content')).rejects.toThrow('URL cannot be empty');
  });

  it('should reject empty title in storeRagPage', async () => {
    await expect(ragClient.storeRagPage('https://example.com', '', 'Content')).rejects.toThrow('Title cannot be empty');
  });

  it('should reject empty content in storeRagPage', async () => {
    await expect(ragClient.storeRagPage('https://example.com', 'Title', '')).rejects.toThrow('Content cannot be empty');
  });

  it('should reject content too large in storeRagPage', async () => {
    const largeContent = 'x'.repeat(10_000_001);
    await expect(ragClient.storeRagPage('https://example.com', 'Title', largeContent)).rejects.toThrow('Content too large');
  });

  it('should reject empty URL in addRagBookmark', async () => {
    await expect(ragClient.addRagBookmark('', 'Title', 'Folder')).rejects.toThrow('URL cannot be empty');
  });

  it('should reject invalid URL protocol in addRagBookmark', async () => {
    await expect(ragClient.addRagBookmark('ftp://example.com', 'Title', 'Folder')).rejects.toThrow('URL must start with http:// or https://');
  });

  it('should reject empty query in searchRagPages', async () => {
    await expect(ragClient.searchRagPages('')).rejects.toThrow('Query cannot be empty');
  });

  it('should reject query too long in searchRagPages', async () => {
    const longQuery = 'x'.repeat(1001);
    await expect(ragClient.searchRagPages(longQuery)).rejects.toThrow('Query too long');
  });

  it('should reject empty embedding in searchRagSemantic', async () => {
    await expect(ragClient.searchRagSemantic([])).rejects.toThrow('Embedding vector cannot be empty');
  });

  it('should reject embedding too large in searchRagSemantic', async () => {
    const largeEmbedding = new Array(4097).fill(0.1);
    await expect(ragClient.searchRagSemantic(largeEmbedding)).rejects.toThrow('Embedding vector too large');
  });

  it('should reject empty query in searchRagHybrid', async () => {
    await expect(ragClient.searchRagHybrid('')).rejects.toThrow('Query cannot be empty');
  });

  it('should reject empty URL in recordRagVisit', async () => {
    await expect(ragClient.recordRagVisit('', 'Title')).rejects.toThrow('URL cannot be empty');
  });

  it('should reject invalid URL protocol in recordRagVisit', async () => {
    await expect(ragClient.recordRagVisit('ftp://example.com', 'Title')).rejects.toThrow('URL must start with http:// or https://');
  });

  it('should reject empty query in searchRagVisits', async () => {
    await expect(ragClient.searchRagVisits('')).rejects.toThrow('Query cannot be empty');
  });
});
