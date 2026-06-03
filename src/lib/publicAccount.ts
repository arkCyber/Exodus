/**
 * Exodus Browser — Public Account (公众号) API
 * Aerospace-grade implementation with robust error handling and type safety.
 */

import { invoke } from '@tauri-apps/api/core';

export type PublicAccount = {
  account_id: string;
  owner_id: string;
  name: string;
  description: string;
  avatar_url?: string;
  cover_url?: string;
  category: string;
  tags: string[];
  is_verified: boolean;
  follower_count: number;
  article_count: number;
  created_at: number;
  updated_at: number;
};

export type Article = {
  article_id: string;
  account_id: string;
  title: string;
  content: string;
  summary?: string;
  author_id: string;
  author_name: string;
  category: string;
  tags: string[];
  cover_url?: string;
  media_items: MediaItem[];
  published_at: number;
  updated_at: number;
  view_count: number;
  like_count: number;
  comment_count: number;
  share_count: number;
  is_published: boolean;
  is_scheduled: boolean;
  scheduled_time?: number;
};

export type Follower = {
  follower_id: string;
  account_id: string;
  followed_at: number;
};

export type ArticleAnalytics = {
  article_id: string;
  view_count: number;
  like_count: number;
  comment_count: number;
  share_count: number;
  read_time_avg: number;
  completion_rate: number;
};

export type AccountAnalytics = {
  account_id: string;
  follower_count: number;
  article_count: number;
  total_views: number;
  total_likes: number;
  total_comments: number;
  total_shares: number;
  engagement_rate: number;
  growth_rate: number;
};

export type MediaItem = {
  media_id: string;
  account_id: string;
  media_type: string;
  url: string;
  thumbnail_url?: string;
  title: string;
  description?: string;
  file_size: number;
  width?: number;
  height?: number;
  duration?: number;
  uploaded_at: number;
};

export type CustomMenuItem = {
  menu_id: string;
  account_id: string;
  title: string;
  action_type: string;
  action_data: string;
  icon_url?: string;
  order: number;
};

export type PushNotification = {
  notification_id: string;
  account_id: string;
  recipient_id: string;
  title: string;
  content: string;
  payload?: string;
  sent_at: number;
  read_at?: number;
};

// Service management
export async function publicAccountServiceStart(): Promise<void> {
  return invoke('public_account_service_start');
}

export async function publicAccountServiceStop(): Promise<void> {
  return invoke('public_account_service_stop');
}

// Account management
export async function publicAccountCreate(account: PublicAccount): Promise<string> {
  return invoke('public_account_create', { account });
}

export async function publicAccountGet(accountId: string): Promise<PublicAccount> {
  return invoke('public_account_get', { accountId });
}

export async function publicAccountUpdate(account: PublicAccount): Promise<string> {
  return invoke('public_account_update', { account });
}

export async function publicAccountList(): Promise<PublicAccount[]> {
  return invoke('public_account_list');
}

export async function publicAccountGetByOwner(ownerId: string): Promise<PublicAccount[]> {
  return invoke('public_account_get_by_owner', { ownerId });
}

export async function publicAccountSearch(query: string, limit?: number): Promise<PublicAccount[]> {
  return invoke('public_account_search', { query, limit });
}

// Article management
export async function publicAccountPublishArticle(article: Article): Promise<string> {
  return invoke('public_account_publish_article', { article });
}

export async function publicAccountScheduleArticle(article: Article, scheduledTime: number): Promise<string> {
  return invoke('public_account_schedule_article', { article, scheduledTime });
}

export async function publicAccountGetArticle(articleId: string): Promise<Article> {
  return invoke('public_account_get_article', { articleId });
}

export async function publicAccountListArticles(accountId: string): Promise<Article[]> {
  return invoke('public_account_list_articles', { accountId });
}

export async function publicAccountGetScheduledArticles(accountId: string): Promise<Article[]> {
  return invoke('public_account_get_scheduled_articles', { accountId });
}

export async function publicAccountSaveDraft(article: Article): Promise<string> {
  return invoke('public_account_save_draft', { article });
}

export async function publicAccountListDrafts(accountId: string): Promise<Article[]> {
  return invoke('public_account_list_drafts', { accountId });
}

export async function publicAccountDeleteDraft(articleId: string): Promise<boolean> {
  return invoke('public_account_delete_draft', { articleId });
}

export async function publicAccountGetTrendingArticles(limit?: number): Promise<Article[]> {
  return invoke('public_account_get_trending_articles', { limit });
}

export async function publicAccountGetArticlesByCategory(category: string, limit?: number): Promise<Article[]> {
  return invoke('public_account_get_articles_by_category', { category, limit });
}

export async function publicAccountRecommendArticles(userId: string, limit?: number): Promise<Article[]> {
  return invoke('public_account_recommend_articles', { userId, limit });
}

// Subscription management
export async function publicAccountSubscribe(followerId: string, accountId: string): Promise<boolean> {
  return invoke('public_account_subscribe', { followerId, accountId });
}

export async function publicAccountUnsubscribe(followerId: string, accountId: string): Promise<boolean> {
  return invoke('public_account_unsubscribe', { followerId, accountId });
}

export async function publicAccountGetFollowers(accountId: string): Promise<Follower[]> {
  return invoke('public_account_get_followers', { accountId });
}

export async function publicAccountGetSubscriptions(userId: string): Promise<string[]> {
  return invoke('public_account_get_subscriptions', { userId });
}

// Article interactions
export async function publicAccountRecordView(articleId: string, userId: string): Promise<boolean> {
  return invoke('public_account_record_view', { articleId, userId });
}

export async function publicAccountLikeArticle(articleId: string, userId: string): Promise<boolean> {
  return invoke('public_account_like_article', { articleId, userId });
}

// Analytics
export async function publicAccountGetAnalytics(accountId: string): Promise<AccountAnalytics> {
  return invoke('public_account_get_analytics', { accountId });
}

export async function publicAccountGetArticleAnalytics(articleId: string): Promise<ArticleAnalytics> {
  return invoke('public_account_get_article_analytics', { articleId });
}

export async function publicAccountGetRealtimeAnalytics(accountId: string): Promise<AccountAnalytics> {
  return invoke('public_account_get_realtime_analytics', { accountId });
}

// Media management
export async function publicAccountUploadMedia(media: MediaItem): Promise<string> {
  return invoke('public_account_upload_media', { media });
}

export async function publicAccountDeleteMedia(mediaId: string): Promise<boolean> {
  return invoke('public_account_delete_media', { mediaId });
}

export async function publicAccountGetMedia(mediaId: string): Promise<MediaItem> {
  return invoke('public_account_get_media', { mediaId });
}

export async function publicAccountListMedia(accountId: string): Promise<MediaItem[]> {
  return invoke('public_account_list_media', { accountId });
}

export async function publicAccountListMediaByType(accountId: string, mediaType: string): Promise<MediaItem[]> {
  return invoke('public_account_list_media_by_type', { accountId, mediaType });
}

// Custom menu
export async function publicAccountAddMenuItem(menu: CustomMenuItem): Promise<string> {
  return invoke('public_account_add_menu_item', { menu });
}

export async function publicAccountUpdateMenuItem(menu: CustomMenuItem): Promise<boolean> {
  return invoke('public_account_update_menu_item', { menu });
}

export async function publicAccountDeleteMenuItem(menuId: string): Promise<boolean> {
  return invoke('public_account_delete_menu_item', { menuId });
}

export async function publicAccountGetMenuItems(accountId: string): Promise<CustomMenuItem[]> {
  return invoke('public_account_get_menu_items', { accountId });
}

// Notifications
export async function publicAccountSendNotification(notification: PushNotification): Promise<string> {
  return invoke('public_account_send_notification', { notification });
}

export async function publicAccountMarkNotificationRead(notificationId: string): Promise<boolean> {
  return invoke('public_account_mark_notification_read', { notificationId });
}

export async function publicAccountGetNotifications(recipientId: string): Promise<PushNotification[]> {
  return invoke('public_account_get_notifications', { recipientId });
}

export async function publicAccountGetUnreadNotifications(recipientId: string): Promise<PushNotification[]> {
  return invoke('public_account_get_unread_notifications', { recipientId });
}
