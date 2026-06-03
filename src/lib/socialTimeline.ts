/**
 * Exodus Browser — Social Timeline (朋友圈) Data Types and API
 * Aerospace-grade implementation with robust error handling and type safety.
 */

import { invoke } from '@tauri-apps/api/core';

// Data Types

export type SocialPost = {
  postId: string;
  authorId: string;
  authorName: string;
  authorAvatar?: string | null;
  content: string;
  attachments: PostAttachment[];
  tags: string[];
  location?: string | null;
  mentions: string[];
  visibility: 'public' | 'friends' | 'private';
  likes: number;
  comments: number;
  shares: number;
  likedByUser: boolean;
  timestamp: number;
  editedAt?: number | null;
  publicAccountId?: string | null;
};

export type PostAttachment = {
  attachmentId: string;
  fileType: 'image' | 'video' | 'audio' | 'file';
  blobHash: string;
  fileName: string;
  fileSize: number;
  thumbnailHash?: string | null;
  caption?: string | null;
};

export type SocialComment = {
  commentId: string;
  postId: string;
  authorId: string;
  authorName: string;
  authorAvatar?: string | null;
  content: string;
  parentId?: string | null;
  mentions: string[];
  timestamp: number;
  editedAt?: number | null;
  likes: number;
  replies: number;
};

export type SocialLike = {
  likeId: string;
  postId: string;
  userId: string;
  userName: string;
  timestamp: number;
};

export type TimelinePost = SocialPost & {
  commentsPreview?: SocialComment[];
  likesPreview?: SocialLike[];
};

// API Functions

/** Start the social timeline microservice. */
export async function socialTimelineServiceStart(): Promise<void> {
  await invoke('social_feed_service_start');
}

/** Create a new post. */
export async function createPost(postPayload: Record<string, unknown>): Promise<string> {
  return invoke<string>('social_post_create', postPayload);
}

/** Get timeline posts for a user. */
export async function getTimeline(userId: string, limit?: number): Promise<TimelinePost[]> {
  const raw = await invoke<Record<string, unknown>[]>('social_feed_get_timeline', {
    userId,
    limit: limit ?? 50,
  });
  const posts = raw as unknown as Record<string, unknown>[];
  return posts.map(mapTimelinePost);
}

/** Get a single post by ID. */
export async function getPost(postId: string): Promise<SocialPost | null> {
  try {
    const raw = await invoke<Record<string, unknown>>('social_post_get', { postId });
    return mapSocialPost(raw);
  } catch {
    return null;
  }
}

/** Like a post. */
export async function likePost(postId: string, userId: string): Promise<void> {
  const reaction = {
    reactionId: `reaction-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    targetId: postId,
    targetType: 'post',
    userId,
    reactionType: 'like',
    createdAt: Date.now(),
  };
  await invoke('social_reaction_add', reaction);
}

/** Unlike a post. */
export async function unlikePost(postId: string, userId: string): Promise<void> {
  await invoke('social_reaction_remove', { targetId: postId, targetType: 'post', userId });
}

/** Add a comment to a post. */
export async function addComment(commentPayload: Record<string, unknown>): Promise<string> {
  return invoke<string>('social_comment_add', commentPayload);
}

/** Get comments for a post. */
export async function getComments(postId: string, limit?: number): Promise<SocialComment[]> {
  const raw = await invoke<Record<string, unknown>[]>('social_comment_get', {
    postId,
    limit: limit ?? 50,
  });
  const comments = raw as unknown as Record<string, unknown>[];
  return comments.map(mapSocialComment);
}

/** Delete a post. */
export async function deletePost(postId: string): Promise<void> {
  await invoke('social_post_delete', { postId });
}

/** Delete a comment. */
export async function deleteComment(postId: string, commentId: string): Promise<void> {
  await invoke('social_comment_delete', { postId, commentId });
}

/** Edit a post. */
export async function editPost(postId: string, newContent: string): Promise<void> {
  const post = await getPost(postId);
  if (!post) throw new Error('Post not found');
  const updatedPost = {
    ...post,
    content: newContent,
    updatedAt: Date.now(),
  };
  await invoke('social_post_update', updatedPost);
}

/** Edit a comment. */
export async function editComment(
  postId: string,
  commentId: string,
  newContent: string,
): Promise<void> {
  await invoke('social_comment_update', { postId, commentId, content: newContent });
}

// Helper functions

function mapPostAttachment(raw: Record<string, unknown>): PostAttachment {
  return {
    attachmentId: String(raw.attachmentId ?? raw.attachment_id ?? ''),
    fileType: (raw.fileType ?? raw.file_type ?? 'file') as 'image' | 'video' | 'audio' | 'file',
    blobHash: String(raw.blobHash ?? raw.blob_hash ?? ''),
    fileName: String(raw.fileName ?? raw.file_name ?? ''),
    fileSize: Number(raw.fileSize ?? raw.file_size ?? 0),
    thumbnailHash: (raw.thumbnailHash ?? raw.thumbnail_hash) as string | null | undefined,
    caption: (raw.caption) as string | null | undefined,
  };
}

function mapSocialPost(raw: Record<string, unknown>): SocialPost {
  return {
    postId: String(raw.postId ?? raw.post_id ?? ''),
    authorId: String(raw.authorId ?? raw.author_id ?? ''),
    authorName: String(raw.authorName ?? raw.author_name ?? ''),
    authorAvatar: (raw.authorAvatar ?? raw.author_avatar) as string | null | undefined,
    content: String(raw.content ?? ''),
    attachments: Array.isArray(raw.attachments) ? raw.attachments.map(mapPostAttachment) : [],
    tags: Array.isArray(raw.tags) ? raw.tags.map(String) : [],
    location: (raw.location) as string | null | undefined,
    mentions: Array.isArray(raw.mentions) ? raw.mentions.map(String) : [],
    visibility: (raw.visibility ?? 'public') as 'public' | 'friends' | 'private',
    likes: Number(raw.likeCount ?? raw.like_count ?? 0),
    comments: Number(raw.commentCount ?? raw.comment_count ?? 0),
    shares: Number(raw.shareCount ?? raw.share_count ?? 0),
    likedByUser: false, // Backend doesn't track this per user
    timestamp: Number(raw.createdAt ?? raw.created_at ?? 0),
    editedAt: (raw.updatedAt ?? raw.updated_at) as number | null ?? null,
    publicAccountId: (raw.publicAccountId ?? raw.public_account_id) as string | null | undefined,
  };
}

function mapSocialComment(raw: Record<string, unknown>): SocialComment {
  return {
    commentId: String(raw.commentId ?? raw.comment_id ?? ''),
    postId: String(raw.postId ?? raw.post_id ?? ''),
    authorId: String(raw.authorId ?? raw.author_id ?? ''),
    authorName: String(raw.authorName ?? raw.author_name ?? ''),
    authorAvatar: (raw.authorAvatar ?? raw.author_avatar) as string | null | undefined,
    content: String(raw.content ?? ''),
    parentId: (raw.parentId ?? raw.parent_id) as string | null | undefined,
    mentions: Array.isArray(raw.mentions) ? raw.mentions.map(String) : [],
    timestamp: Number(raw.createdAt ?? raw.created_at ?? 0),
    editedAt: (raw.updatedAt ?? raw.updated_at) as number | null ?? null,
    likes: Number(raw.likeCount ?? raw.like_count ?? 0),
    replies: Number(raw.replyCount ?? raw.reply_count ?? 0),
  };
}

function mapSocialLike(raw: Record<string, unknown>): SocialLike {
  return {
    likeId: String(raw.likeId ?? raw.like_id ?? ''),
    postId: String(raw.postId ?? raw.post_id ?? ''),
    userId: String(raw.userId ?? raw.user_id ?? ''),
    userName: String(raw.userName ?? raw.user_name ?? ''),
    timestamp: Number(raw.timestamp ?? 0),
  };
}

function mapTimelinePost(raw: Record<string, unknown>): TimelinePost {
  const post = mapSocialPost(raw);
  return {
    ...post,
    commentsPreview: Array.isArray(raw.commentsPreview) 
      ? raw.commentsPreview.map(mapSocialComment) 
      : undefined,
    likesPreview: Array.isArray(raw.likesPreview) 
      ? raw.likesPreview.map(mapSocialLike) 
      : undefined,
  };
}

/** Build a snake_case post payload for `createPost`. */
export function buildPostPayload(params: {
  authorId: string;
  authorName: string;
  content: string;
  attachments?: PostAttachment[];
  location?: string;
  visibility?: 'public' | 'friends' | 'private';
}): Record<string, unknown> {
  const now = Date.now();
  return {
    post_id: `post-${now}-${Math.random().toString(36).substr(2, 9)}`,
    author_id: params.authorId,
    author_name: params.authorName,
    content: params.content,
    attachments: (params.attachments ?? []).map(a => ({
      attachment_id: a.attachmentId,
      attachment_type: a.fileType,
      blob_hash: a.blobHash,
      file_name: a.fileName,
      file_size: a.fileSize,
      thumbnail_hash: a.thumbnailHash,
      caption: a.caption,
    })),
    tags: [],
    location: params.location ?? null,
    mentions: [],
    visibility: params.visibility ?? 'public',
    likes: 0,
    comments: 0,
    liked_by_user: false,
    timestamp: now,
    edited_at: null,
    like_count: 0,
    comment_count: 0,
    share_count: 0,
    created_at: now,
    updated_at: now,
  };
}

/** Build a snake_case comment payload for `addComment`. */
export function buildCommentPayload(params: {
  postId: string;
  authorId: string;
  authorName: string;
  content: string;
  parentId?: string;
  replyToCommentId?: string;
}): Record<string, unknown> {
  const now = Date.now();
  const replyId = params.replyToCommentId ?? params.parentId ?? null;
  return {
    comment_id: `comment-${now}-${Math.random().toString(36).substr(2, 9)}`,
    post_id: params.postId,
    author_id: params.authorId,
    author_name: params.authorName,
    content: params.content,
    parent_id: replyId,
    reply_to_comment_id: replyId,
    mentions: [],
    timestamp: now,
    edited_at: null,
    created_at: now,
    updated_at: now,
    like_count: 0,
    reply_count: 0,
  };
}
