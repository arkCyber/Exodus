/**
 * Exodus Browser — Local Pocket (本地文章保存) functionality
 * Frontend integration for Pocket Tauri commands
 */

import { invoke } from '@tauri-apps/api/core';

export type SavedArticle = {
  id: string;
  url: string;
  title: string;
  content: string;
  excerpt: string;
  author: string | null;
  tags: string[];
  saved_at: string;
  read_at: string | null;
  is_favorite: boolean;
  is_archived: boolean;
  reading_time_minutes: number;
  word_count: number;
};

export type SaveArticleRequest = {
  url: string;
  title: string;
  content: string;
  author: string | null;
  tags: string[];
};

export type UpdateArticleRequest = {
  id: string;
  title: string | null;
  tags: string[] | null;
  is_favorite: boolean | null;
  is_archived: boolean | null;
};

export type SearchArticlesRequest = {
  query: string;
  limit: number | null;
  offset: number | null;
};

export type PocketStats = {
  total_articles: number;
  unread_articles: number;
  favorite_articles: number;
  archived_articles: number;
  total_word_count: number;
  total_reading_time_minutes: number;
};

/**
 * Save an article to local pocket
 */
export async function pocketSaveArticle(request: SaveArticleRequest): Promise<SavedArticle> {
  return invoke('pocket_save_article', { request });
}

/**
 * List all saved articles
 */
export async function pocketListArticles(): Promise<SavedArticle[]> {
  return invoke('pocket_list_articles');
}

/**
 * Get a specific article by ID
 */
export async function pocketGetArticle(id: string): Promise<SavedArticle | null> {
  return invoke('pocket_get_article', { id });
}

/**
 * Update an article
 */
export async function pocketUpdateArticle(request: UpdateArticleRequest): Promise<SavedArticle> {
  return invoke('pocket_update_article', { request });
}

/**
 * Mark article as read
 */
export async function pocketMarkAsRead(id: string): Promise<void> {
  return invoke('pocket_mark_as_read', { id });
}

/**
 * Delete an article
 */
export async function pocketDeleteArticle(id: string): Promise<void> {
  return invoke('pocket_delete_article', { id });
}

/**
 * Search articles
 */
export async function pocketSearchArticles(request: SearchArticlesRequest): Promise<SavedArticle[]> {
  return invoke('pocket_search_articles', { request });
}

/**
 * Get articles by tag
 */
export async function pocketGetArticlesByTag(tag: string): Promise<SavedArticle[]> {
  return invoke('pocket_get_articles_by_tag', { tag });
}

/**
 * Get all tags
 */
export async function pocketGetAllTags(): Promise<string[]> {
  return invoke('pocket_get_all_tags');
}

/**
 * Get pocket statistics
 */
export async function pocketGetStats(): Promise<PocketStats> {
  return invoke('pocket_get_stats');
}
