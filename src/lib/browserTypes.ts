/**
 * Exodus Browser — shared frontend types.
 */

import type { Webview } from '@tauri-apps/api/webview';

/** A single browser tab. */
export type BrowserTab = {
  id: string;
  title: string;
  url: string;
  webview: Webview | null;
  /** Cached favicon URL for the tab strip. */
  favicon?: string | null;
  /** Pinned tabs stay left and are harder to close. */
  pinned?: boolean;
};

/** Sidebar panel mode. */
export type SidebarPanel = 'ai' | 'memory' | 'bookmarks' | 'p2p' | 'pocket';

/** RAG-indexed page from `get_history` (local memory for /ask search). */
export type IndexedPage = {
  id: string;
  url: string;
  title: string;
  timestamp: string;
};

/** One message in the sidebar AI chat. */
export type AiChatMessage = {
  role: 'user' | 'assistant';
  content: string;
};

/** Quick link on the new-tab page. */
export type QuickLink = {
  title: string;
  url: string;
};

/** RAG search hit from the backend. */
export type SearchHit = {
  page: {
    url: string;
    title: string;
    timestamp: string;
  };
  score: number;
};

/** Bookmark row from the backend. */
export type BookmarkItem = {
  id: string;
  url: string;
  title: string;
  created_at: string;
  /** Empty = bookmark bar; non-empty = folder in bookmarks panel only. */
  folder?: string;
  /** Bar chip order (lower = left). */
  bar_order?: number;
};

/** Tracked file download. */
export type DownloadRecord = {
  id: string;
  url: string;
  filename: string;
  path?: string;
  status: 'pending' | 'downloading' | 'completed' | 'failed';
  progress: number;
  received: number;
  total: number;
};

/** Snapshot of a tab closed for restore (⌘⇧T). */
export type ClosedTabSnapshot = {
  title: string;
  url: string;
  pinned?: boolean;
};

/** Browsing history row (auto-recorded visits). */
export type HistoryPage = {
  id: string;
  url: string;
  title: string;
  timestamp: string;
  visit_count?: number;
};

/** Navigation state from the content webview. */
export type NavState = {
  url: string;
  can_go_back: boolean;
  can_go_forward: boolean;
};

/** Saved password row from `list_passwords`. */
export type PasswordEntry = {
  id: string;
  url: string;
  username: string;
  password: string;
  site_name: string;
  created_at: number;
  updated_at: number;
  use_count: number;
  strength?: string;
  breach_status?: string;
  notes?: string | null;
  custom_fields?: Record<string, string>;
};
