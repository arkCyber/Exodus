/**
 * Exodus Browser — RAG (Retrieval-Augmented Generation) client
 * Provides interface to RAG service for semantic search and page indexing
 * Aerospace-level error handling and input validation.
 */

import { invoke } from '@tauri-apps/api/core';

// Aerospace-level security validation patterns
const VALID_ID_PATTERN = /^[a-zA-Z0-9_-]+$/;
const VALID_URL_PATTERN = /^https?:\/\/.+/;
const VALID_TITLE_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const VALID_FOLDER_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const VALID_QUERY_PATTERN = /^[a-zA-Z0-9_\s\-.,!?@#$%^&*()+=\[\]{};:'"<>/\\|`~]+$/;
const MAX_URL_LENGTH = 2000;
const MAX_TITLE_LENGTH = 500;
const MAX_FOLDER_LENGTH = 100;
const MAX_QUERY_LENGTH = 1000;
const MAX_CONTENT_LENGTH = 10_000_000;
const MAX_EMBEDDING_DIMENSIONS = 4096;

/**
 * Aerospace-level validation for ID format.
 */
function validateId(id: string): boolean {
  if (!id || typeof id !== 'string') {
    console.error('[RagClient] Invalid ID');
    return false;
  }
  return VALID_ID_PATTERN.test(id);
}

/**
 * Aerospace-level validation for URL format.
 */
function validateUrl(url: string): boolean {
  if (!url || typeof url !== 'string') {
    console.error('[RagClient] Invalid URL');
    return false;
  }
  if (url.length > MAX_URL_LENGTH) {
    console.error('[RagClient] URL too long');
    return false;
  }
  return VALID_URL_PATTERN.test(url);
}

/**
 * Aerospace-level validation for title format.
 */
function validateTitle(title: string): boolean {
  if (!title || typeof title !== 'string') {
    console.error('[RagClient] Invalid title');
    return false;
  }
  if (title.length > MAX_TITLE_LENGTH) {
    console.error('[RagClient] Title too long');
    return false;
  }
  return VALID_TITLE_PATTERN.test(title);
}

/**
 * Aerospace-level validation for folder name format.
 */
function validateFolder(folder: string): boolean {
  if (!folder || typeof folder !== 'string') {
    console.error('[RagClient] Invalid folder name');
    return false;
  }
  if (folder.length > MAX_FOLDER_LENGTH) {
    console.error('[RagClient] Folder name too long');
    return false;
  }
  return VALID_FOLDER_PATTERN.test(folder);
}

/**
 * Aerospace-level validation for query format.
 */
function validateQuery(query: string): boolean {
  if (!query || typeof query !== 'string') {
    console.error('[RagClient] Invalid query');
    return false;
  }
  if (query.length > MAX_QUERY_LENGTH) {
    console.error('[RagClient] Query too long');
    return false;
  }
  // Skip pattern validation for queries to allow more flexibility
  return true;
}

/**
 * Aerospace-level validation for embedding vector.
 */
function validateEmbedding(embedding: number[]): boolean {
  if (!Array.isArray(embedding)) {
    console.error('[RagClient] Invalid embedding (not an array)');
    return false;
  }
  if (embedding.length === 0) {
    console.error('[RagClient] Empty embedding vector');
    return false;
  }
  if (embedding.length > MAX_EMBEDDING_DIMENSIONS) {
    console.error('[RagClient] Embedding vector too large');
    return false;
  }
  // Validate all elements are numbers
  for (const val of embedding) {
    if (typeof val !== 'number' || isNaN(val)) {
      console.error('[RagClient] Invalid embedding value (not a number)');
      return false;
    }
  }
  return true;
}

/**
 * Aerospace-level validation for embedding vector with specific error messages.
 */
function validateEmbeddingWithErrors(embedding: number[]): void {
  if (!Array.isArray(embedding)) {
    throw new Error('Invalid embedding vector');
  }
  if (embedding.length === 0) {
    throw new Error('Embedding vector cannot be empty');
  }
  if (embedding.length > MAX_EMBEDDING_DIMENSIONS) {
    throw new Error('Embedding vector too large');
  }
  // Validate all elements are numbers
  for (const val of embedding) {
    if (typeof val !== 'number' || isNaN(val)) {
      throw new Error('Invalid embedding vector');
    }
  }
}

/** RAG search result with relevance score */
export type RagSearchResult = {
  page: {
    id: string;
    url: string;
    title: string;
    content: string;
    timestamp: string;
    embedding?: number[];
  };
  score: number;
  matchedText: string;
};

/** RAG page entry */
export type RagPage = {
  id: string;
  url: string;
  title: string;
  content: string;
  timestamp: string;
  embedding?: number[];
};

/** Start RAG service */
export async function startRagService(dataDir: string): Promise<string> {
  return invoke<string>('rag_service_start', { dataDir });
}

/** Stop RAG service */
export async function stopRagService(): Promise<boolean> {
  return invoke<boolean>('rag_service_stop');
}

/** Store a page in RAG database */
export async function storeRagPage(url: string, title: string, content: string): Promise<string> {
  // Aerospace-level input validation
  if (!url || typeof url !== 'string' || url.trim().length === 0) {
    console.error('[RagClient] URL cannot be empty in storeRagPage');
    throw new Error('URL cannot be empty');
  }
  if (!validateUrl(url)) {
    console.error('[RagClient] Invalid URL in storeRagPage:', url);
    throw new Error('Invalid URL format');
  }
  if (!title || typeof title !== 'string' || title.trim().length === 0) {
    console.error('[RagClient] Title cannot be empty in storeRagPage');
    throw new Error('Title cannot be empty');
  }
  if (!validateTitle(title)) {
    console.error('[RagClient] Invalid title in storeRagPage:', title);
    throw new Error('Invalid title format');
  }
  if (!content || typeof content !== 'string' || content.trim().length === 0) {
    console.error('[RagClient] Content cannot be empty in storeRagPage');
    throw new Error('Content cannot be empty');
  }
  if (content.length > MAX_CONTENT_LENGTH) {
    console.error('[RagClient] Content too large in storeRagPage:', content.length);
    throw new Error('Content too large (max 10MB)');
  }
  
  return invoke<string>('rag_store_page', { url, title, content });
}

/** Generate embedding and store page */
export async function generateAndStoreRagPage(url: string, title: string, content: string): Promise<string> {
  // Aerospace-level input validation
  if (!url || typeof url !== 'string' || url.trim().length === 0) {
    console.error('[RagClient] URL cannot be empty in generateAndStoreRagPage');
    throw new Error('URL cannot be empty');
  }
  if (!validateUrl(url)) {
    console.error('[RagClient] Invalid URL in generateAndStoreRagPage:', url);
    throw new Error('Invalid URL format');
  }
  if (!title || typeof title !== 'string' || title.trim().length === 0) {
    console.error('[RagClient] Title cannot be empty in generateAndStoreRagPage');
    throw new Error('Title cannot be empty');
  }
  if (!validateTitle(title)) {
    console.error('[RagClient] Invalid title in generateAndStoreRagPage:', title);
    throw new Error('Invalid title format');
  }
  if (!content || typeof content !== 'string' || content.trim().length === 0) {
    console.error('[RagClient] Content cannot be empty in generateAndStoreRagPage');
    throw new Error('Content cannot be empty');
  }
  if (content.length > MAX_CONTENT_LENGTH) {
    console.error('[RagClient] Content too large in generateAndStoreRagPage:', content.length);
    throw new Error('Content too large (max 10MB)');
  }
  
  return invoke<string>('rag_generate_embedding', { url, title, content });
}

/** Search pages by keyword */
export async function searchRagPages(query: string): Promise<RagPage[]> {
  // Aerospace-level input validation
  if (!query || typeof query !== 'string' || query.trim().length === 0) {
    console.error('[RagClient] Query cannot be empty in searchRagPages');
    throw new Error('Query cannot be empty');
  }
  if (query.length > MAX_QUERY_LENGTH) {
    console.error('[RagClient] Query too long in searchRagPages:', query.length);
    throw new Error('Query too long (max 1000 characters)');
  }
  
  const result = await invoke<string>('rag_search_pages', { query });
  return JSON.parse(result);
}

/** Semantic search using vector embeddings */
export async function searchRagSemantic(queryEmbedding: number[]): Promise<RagSearchResult[]> {
  // Aerospace-level input validation with specific error messages
  validateEmbeddingWithErrors(queryEmbedding);
  
  const result = await invoke<string>('rag_search_semantic', { queryEmbedding });
  return JSON.parse(result);
}

/** Hybrid search (keyword + semantic) */
export async function searchRagHybrid(query: string, queryEmbedding?: number[]): Promise<RagSearchResult[]> {
  // Aerospace-level input validation
  if (!query || typeof query !== 'string' || query.trim().length === 0) {
    console.error('[RagClient] Query cannot be empty in searchRagHybrid');
    throw new Error('Query cannot be empty');
  }
  if (query.length > MAX_QUERY_LENGTH) {
    console.error('[RagClient] Query too long in searchRagHybrid:', query.length);
    throw new Error('Query too long (max 1000 characters)');
  }
  if (queryEmbedding) {
    if (!validateEmbedding(queryEmbedding)) {
      console.error('[RagClient] Invalid embedding in searchRagHybrid');
      throw new Error('Invalid embedding vector');
    }
  }
  
  const result = await invoke<string>('rag_search_hybrid', { query, queryEmbedding });
  return JSON.parse(result);
}

/** Add bookmark to RAG database */
export async function addRagBookmark(url: string, title: string, folder: string = ''): Promise<string> {
  // Aerospace-level input validation
  if (!url || typeof url !== 'string' || url.trim().length === 0) {
    console.error('[RagClient] URL cannot be empty in addRagBookmark');
    throw new Error('URL cannot be empty');
  }
  if (!validateUrl(url)) {
    console.error('[RagClient] Invalid URL in addRagBookmark:', url);
    throw new Error('URL must start with http:// or https://');
  }
  if (!title || typeof title !== 'string' || title.trim().length === 0) {
    console.error('[RagClient] Title cannot be empty in addRagBookmark');
    throw new Error('Title cannot be empty');
  }
  if (!validateTitle(title)) {
    console.error('[RagClient] Invalid title in addRagBookmark:', title);
    throw new Error('Invalid title format');
  }
  if (folder && !validateFolder(folder)) {
    console.error('[RagClient] Invalid folder in addRagBookmark:', folder);
    throw new Error('Invalid folder format');
  }
  
  const result = await invoke<string>('rag_add_bookmark', { url, title, folder });
  return result;
}

/** List bookmarks from RAG database */
export async function listRagBookmarks(): Promise<any[]> {
  const result = await invoke<string>('rag_list_bookmarks');
  return JSON.parse(result);
}

/** Record visit in RAG database */
export async function recordRagVisit(url: string, title: string): Promise<string> {
  // Aerospace-level input validation
  if (!url || typeof url !== 'string' || url.trim().length === 0) {
    console.error('[RagClient] URL cannot be empty in recordRagVisit');
    throw new Error('URL cannot be empty');
  }
  if (!validateUrl(url)) {
    console.error('[RagClient] Invalid URL in recordRagVisit:', url);
    throw new Error('URL must start with http:// or https://');
  }
  if (!title || typeof title !== 'string' || title.trim().length === 0) {
    console.error('[RagClient] Title cannot be empty in recordRagVisit');
    throw new Error('Title cannot be empty');
  }
  if (!validateTitle(title)) {
    console.error('[RagClient] Invalid title in recordRagVisit:', title);
    throw new Error('Invalid title format');
  }
  
  const result = await invoke<string>('rag_record_visit', { url, title });
  return result;
}

/** Search visits by query */
export async function searchRagVisits(query: string): Promise<any[]> {
  // Aerospace-level input validation
  if (!query || typeof query !== 'string' || query.trim().length === 0) {
    console.error('[RagClient] Query cannot be empty in searchRagVisits');
    throw new Error('Query cannot be empty');
  }
  if (query.length > MAX_QUERY_LENGTH) {
    console.error('[RagClient] Query too long in searchRagVisits:', query.length);
    throw new Error('Query too long (max 1000 characters)');
  }
  
  const result = await invoke<string>('rag_search_visits', { query });
  return JSON.parse(result);
}
