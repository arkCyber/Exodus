<!--
  Exodus Browser — Social Timeline (朋友圈) Component
  Aerospace-grade implementation with robust error handling and type safety.
-->
<template>
  <div class="social-timeline" :class="{ 'dark-mode': settings.theme === 'Dark' }">
    <!-- Timeline Header -->
    <div class="timeline-header">
      <div class="user-profile">
        <div class="user-avatar-large">
          <img :src="userAvatar" :alt="localName" />
        </div>
        <div class="user-info">
          <h3 class="user-name">{{ localName }}</h3>
          <p class="user-status">{{ localNode }}</p>
        </div>
      </div>
      <button type="button" class="create-post-button" @click="showCreatePost = true">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        发布动态
      </button>
    </div>

    <!-- Timeline Posts -->
    <div class="timeline-posts">
      <div v-if="loading" class="loading-state">
        <div class="spinner"></div>
        <p>Loading timeline...</p>
      </div>
      <div v-else-if="posts.length === 0" class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
          <circle cx="9" cy="7" r="4"></circle>
          <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
          <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
        </svg>
        <p>No posts yet</p>
        <button type="button" class="primary-button" @click="showCreatePost = true">Create your first post</button>
      </div>
      <div v-else>
        <div v-for="post in posts" :key="post.postId" class="post-card">
          <!-- Post Header -->
          <div class="post-header">
            <div class="post-author">
              <img :src="getAvatarUrl(post.authorId)" :alt="post.authorName" class="author-avatar" />
              <div class="author-info">
                <h4 class="author-name">{{ post.authorName }}</h4>
                <p class="post-time">{{ formatPostTime(post.timestamp) }}</p>
              </div>
            </div>
            <button v-if="post.authorId === localUserId" type="button" class="post-menu-button" @click="showPostMenu(post, $event)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="1"></circle>
                <circle cx="12" cy="5" r="1"></circle>
                <circle cx="12" cy="19" r="1"></circle>
              </svg>
            </button>
          </div>

          <!-- Post Content -->
          <div class="post-content">
            <p class="post-text">{{ post.content }}</p>
            <p v-if="post.location" class="post-location">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"></path>
                <circle cx="12" cy="10" r="3"></circle>
              </svg>
              {{ post.location }}
            </p>
          </div>

          <!-- Post Attachments -->
          <div v-if="post.attachments.length > 0" class="post-attachments">
            <div v-for="attachment in post.attachments" :key="attachment.attachmentId" class="attachment-item">
              <img v-if="attachment.fileType === 'image'" :src="getAttachmentUrl(attachment)" :alt="attachment.fileName" />
              <div v-else class="file-attachment">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
                  <polyline points="13 2 13 9 20 9"></polyline>
                </svg>
                <span>{{ attachment.fileName }}</span>
              </div>
            </div>
          </div>

          <!-- Post Actions -->
          <div class="post-actions">
            <button type="button" class="action-button" :class="{ liked: post.likedByUser }" @click="toggleLike(post)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path>
              </svg>
              <span>{{ post.likes }}</span>
            </button>
            <button type="button" class="action-button" @click="showComments(post)">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"></path>
              </svg>
              <span>{{ post.comments }}</span>
            </button>
          </div>

          <!-- Post Comments Preview -->
          <div v-if="post.commentsPreview && post.commentsPreview.length > 0" class="comments-preview">
            <div v-for="comment in post.commentsPreview.slice(0, 2)" :key="comment.commentId" class="comment-item">
              <span class="comment-author">{{ comment.authorName }}:</span>
              <span class="comment-text">{{ comment.content }}</span>
            </div>
            <button v-if="post.comments > 2" type="button" class="view-all-comments" @click="showComments(post)">
              View all {{ post.comments }} comments
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Post Dialog -->
    <div v-if="showCreatePost" class="modal-overlay" @click.self="showCreatePost = false">
      <div class="modal-content create-post-content">
        <h3>Create Post</h3>
        <div class="form-group">
          <textarea 
            v-model="newPostContent" 
            class="post-textarea" 
            placeholder="What's on your mind?" 
            rows="4"
            @input="autoResizeTextarea"
          ></textarea>
        </div>
        <div class="form-group">
          <label>Location (Optional)</label>
          <input v-model="newPostLocation" type="text" class="form-input" placeholder="Add location" />
        </div>
        <div class="form-group">
          <label>Visibility</label>
          <select v-model="newPostVisibility" class="form-select">
            <option value="public">Public</option>
            <option value="friends">Friends Only</option>
            <option value="private">Private</option>
          </select>
        </div>
        <div class="modal-actions">
          <button type="button" class="secondary-button" @click="showCreatePost = false">Cancel</button>
          <button type="button" class="primary-button" @click="createPost" :disabled="!newPostContent.trim()">Post</button>
        </div>
      </div>
    </div>

    <!-- Comments Dialog -->
    <div v-if="showCommentsDialog" class="modal-overlay" @click.self="showCommentsDialog = false">
      <div class="modal-content comments-content">
        <h3>Comments</h3>
        <div class="comments-list">
          <div v-if="commentsLoading" class="loading-state">
            <div class="spinner"></div>
            <p>Loading comments...</p>
          </div>
          <div v-else-if="comments.length === 0" class="empty-state">
            <p>No comments yet</p>
          </div>
          <div v-else>
            <div v-for="comment in comments" :key="comment.commentId" class="comment-card">
              <div class="comment-header">
                <img :src="getAvatarUrl(comment.authorId)" :alt="comment.authorName" class="comment-avatar" />
                <div class="comment-author-info">
                  <h4 class="comment-author-name">{{ comment.authorName }}</h4>
                  <p class="comment-time">{{ formatPostTime(comment.timestamp) }}</p>
                </div>
                <button v-if="comment.authorId === localUserId" type="button" class="comment-menu-button" @click="deleteComment(comment.commentId)">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  </svg>
                </button>
              </div>
              <p class="comment-text">{{ comment.content }}</p>
            </div>
          </div>
        </div>
        <div class="add-comment">
          <input v-model="newComment" type="text" class="comment-input" placeholder="Write a comment..." @keydown.enter="addComment" />
          <button type="button" class="send-comment-button" @click="addComment" :disabled="!newComment.trim()">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="22" y1="2" x2="11" y2="13"></line>
              <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Post Menu -->
    <div v-if="postMenu.visible" class="context-menu" :style="{ left: postMenu.x + 'px', top: postMenu.y + 'px' }">
      <button v-if="postMenu.post?.authorId === localUserId" type="button" class="context-menu-item" @click="editPost">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
        </svg>
        编辑
      </button>
      <button v-if="postMenu.post?.authorId === localUserId" type="button" class="context-menu-item danger" @click="deletePost">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
        删除
      </button>
      <button type="button" class="context-menu-item" @click="hidePostMenu">取消</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  socialTimelineServiceStart,
  getTimeline,
  createPost as createPostApi,
  likePost,
  unlikePost,
  getComments,
  addComment as addCommentApi,
  deleteComment as deleteCommentApi,
  deletePost as deletePostApi,
  editPost as editPostApi,
  buildPostPayload,
  buildCommentPayload,
  type TimelinePost,
  type SocialComment,
} from '$lib/socialTimeline';
import { resolveLocalIdentity } from '$lib/imSession';
import { logInfo, logError } from '@/lib/logger';

const emit = defineEmits<{ status: [message: string] }>();

const localUserId = ref('exodus-local-user');
const localName = ref('You');
const localNode = ref('');

const posts = ref<TimelinePost[]>([]);
const loading = ref(false);
const showCreatePost = ref(false);
const newPostContent = ref('');
const newPostLocation = ref('');
const newPostVisibility = ref<'public' | 'friends' | 'private'>('public');

const showCommentsDialog = ref(false);
const comments = ref<SocialComment[]>([]);
const commentsLoading = ref(false);
const currentPostId = ref('');
const newComment = ref('');

const postMenu = ref<{ visible: boolean; x: number; y: number; post: TimelinePost | null }>({
  visible: false,
  x: 0,
  y: 0,
  post: null,
});

const settings = ref({
  theme: 'Light',
});

const userAvatar = computed(() => getAvatarUrl(localNode.value));

function getAvatarUrl(nodeId: string): string {
  return `https://api.dicebear.com/7.x/avataaars/svg?seed=${nodeId}`;
}

function getAttachmentUrl(_attachment: any): string {
  // Placeholder for actual CDN URL
  return `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200"><rect fill="#ddd" width="200" height="200"/><text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" fill="#666">Image</text></svg>')}`;
}

function formatPostTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'Just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}

function onStatus(msg: string): void {
  emit('status', msg);
}

function normalizeError(e: unknown): Error {
  return e instanceof Error ? e : new Error(String(e));
}

async function bootstrap(): Promise<void> {
  try {
    await socialTimelineServiceStart();
    const id = await resolveLocalIdentity();
    localUserId.value = id.userId;
    localName.value = id.displayName;
    localNode.value = id.nodeId;
    await loadTimeline();
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to bootstrap', error);
    onStatus(`Failed to initialize: ${error.message}`);
  }
}

async function loadTimeline(): Promise<void> {
  loading.value = true;
  try {
    posts.value = await getTimeline(localUserId.value);
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to load timeline', error);
    onStatus(`Failed to load timeline: ${error.message}`);
  } finally {
    loading.value = false;
  }
}

async function createPost(): Promise<void> {
  if (!newPostContent.value.trim()) return;

  try {
    const payload = buildPostPayload({
      authorId: localUserId.value,
      authorName: localName.value,
      content: newPostContent.value.trim(),
      location: newPostLocation.value.trim() || undefined,
      visibility: newPostVisibility.value,
    });
    await createPostApi(payload);
    newPostContent.value = '';
    newPostLocation.value = '';
    newPostVisibility.value = 'public';
    showCreatePost.value = false;
    await loadTimeline();
    onStatus('Post created');
    logInfo('SocialTimeline', 'Post created');
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to create post', error);
    onStatus(`Failed to create post: ${error.message}`);
  }
}

async function showComments(post: TimelinePost): Promise<void> {
  currentPostId.value = post.postId;
  showCommentsDialog.value = true;
  commentsLoading.value = true;
  try {
    comments.value = await getComments(post.postId);
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to load comments', error);
    onStatus(`Failed to load comments: ${error.message}`);
  } finally {
    commentsLoading.value = false;
  }
}

async function addComment(): Promise<void> {
  if (!newComment.value.trim() || !currentPostId.value) return;

  try {
    const payload = buildCommentPayload({
      postId: currentPostId.value,
      authorId: localUserId.value,
      authorName: localName.value,
      content: newComment.value.trim(),
    });
    await addCommentApi(payload);
    newComment.value = '';
    await showComments({ postId: currentPostId } as unknown as TimelinePost);
    // Update post comment count
    const post = posts.value.find(p => p.postId === currentPostId.value);
    if (post) post.comments++;
    onStatus('Comment added');
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to add comment', error);
    onStatus(`Failed to add comment: ${error.message}`);
  }
}

async function deleteComment(commentId: string): Promise<void> {
  if (!currentPostId.value) {
    onStatus('No post selected for comment delete');
    return;
  }
  try {
    await deleteCommentApi(currentPostId.value, commentId);
    comments.value = comments.value.filter(c => c.commentId !== commentId);
    const post = posts.value.find(p => p.postId === currentPostId.value);
    if (post) post.comments--;
    onStatus('Comment deleted');
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to delete comment', error);
    onStatus(`Failed to delete comment: ${error.message}`);
  }
}

async function toggleLike(post: TimelinePost): Promise<void> {
  try {
    if (post.likedByUser) {
      await unlikePost(post.postId, localUserId.value);
      post.likes--;
      post.likedByUser = false;
    } else {
      await likePost(post.postId, localUserId.value);
      post.likes++;
      post.likedByUser = true;
    }
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to toggle like', error);
    onStatus(`Failed to like post: ${error.message}`);
  }
}

function showPostMenu(post: TimelinePost, event: MouseEvent): void {
  postMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    post,
  };
}

function hidePostMenu(): void {
  postMenu.value = {
    visible: false,
    x: 0,
    y: 0,
    post: null,
  };
}

async function deletePost(): Promise<void> {
  if (!postMenu.value.post) return;

  try {
    await deletePostApi(postMenu.value.post.postId);
    posts.value = posts.value.filter(p => p.postId !== postMenu.value.post?.postId);
    hidePostMenu();
    onStatus('Post deleted');
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to delete post', error);
    onStatus(`Failed to delete post: ${error.message}`);
  }
}

async function editPost(): Promise<void> {
  if (!postMenu.value.post) return;
  const post = postMenu.value.post;
  hidePostMenu();
  const next = window.prompt('Edit post', post.content);
  if (next == null || next.trim() === post.content) return;
  try {
    await editPostApi(post.postId, next.trim());
    const idx = posts.value.findIndex((p) => p.postId === post.postId);
    if (idx >= 0) {
      posts.value[idx] = { ...posts.value[idx], content: next.trim(), editedAt: Date.now() };
    }
    onStatus('Post updated');
  } catch (e) {
    const error = normalizeError(e);
    logError('SocialTimeline', 'Failed to edit post', error);
    onStatus(`Failed to edit post: ${error.message}`);
  }
}

function autoResizeTextarea(): void {
  const textarea = document.querySelector('.post-textarea') as HTMLTextAreaElement;
  if (textarea) {
    textarea.style.height = 'auto';
    textarea.style.height = `${textarea.scrollHeight}px`;
  }
}

onMounted(() => {
  void bootstrap();
});
</script>

<style scoped>
.social-timeline {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f5f5f5;
  overflow: hidden;
}

.dark-mode.social-timeline {
  background: #1a1a1a;
}

.timeline-header {
  padding: 16px;
  background: white;
  border-bottom: 1px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dark-mode .timeline-header {
  background: #2a2a2a;
  border-bottom-color: #404040;
}

.user-profile {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-avatar-large {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  overflow: hidden;
}

.user-avatar-large img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-info h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.user-info p {
  margin: 0;
  font-size: 12px;
  color: #666;
}

.dark-mode .user-info p {
  color: #aaa;
}

.create-post-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 20px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.create-post-button:hover {
  background: #0056b3;
}

.timeline-posts {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: #666;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e0e0e0;
  border-top-color: #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.post-card {
  background: white;
  border-radius: 12px;
  padding: 16px;
  margin-bottom: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.dark-mode .post-card {
  background: #2a2a2a;
}

.post-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.post-author {
  display: flex;
  align-items: center;
  gap: 12px;
}

.author-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  object-fit: cover;
}

.author-info h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.author-info p {
  margin: 0;
  font-size: 12px;
  color: #666;
}

.dark-mode .author-info p {
  color: #aaa;
}

.post-menu-button {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  color: #666;
}

.post-menu-button:hover {
  color: #333;
}

.dark-mode .post-menu-button {
  color: #aaa;
}

.dark-mode .post-menu-button:hover {
  color: #fff;
}

.post-content {
  margin-bottom: 12px;
}

.post-text {
  margin: 0 0 8px 0;
  line-height: 1.5;
  white-space: pre-wrap;
}

.post-location {
  display: flex;
  align-items: center;
  gap: 4px;
  margin: 0;
  font-size: 12px;
  color: #666;
}

.dark-mode .post-location {
  color: #aaa;
}

.post-attachments {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 8px;
  margin-bottom: 12px;
}

.attachment-item img {
  width: 100%;
  border-radius: 8px;
  object-fit: cover;
}

.file-attachment {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
}

.dark-mode .file-attachment {
  background: #3a3a3a;
}

.post-actions {
  display: flex;
  gap: 16px;
  padding-top: 12px;
  border-top: 1px solid #e0e0e0;
}

.dark-mode .post-actions {
  border-top-color: #404040;
}

.action-button {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  cursor: pointer;
  color: #666;
  font-size: 14px;
}

.action-button:hover {
  color: #007bff;
}

.action-button.liked {
  color: #e91e63;
}

.dark-mode .action-button {
  color: #aaa;
}

.dark-mode .action-button:hover {
  color: #4dabf7;
}

.comments-preview {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e0e0e0;
}

.dark-mode .comments-preview {
  border-top-color: #404040;
}

.comment-item {
  margin-bottom: 8px;
  font-size: 13px;
}

.comment-author {
  font-weight: 600;
  color: #333;
}

.dark-mode .comment-author {
  color: #fff;
}

.comment-text {
  color: #666;
}

.dark-mode .comment-text {
  color: #aaa;
}

.view-all-comments {
  background: none;
  border: none;
  cursor: pointer;
  color: #007bff;
  font-size: 13px;
  margin-top: 8px;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
}

.dark-mode .modal-content {
  background: #2a2a2a;
}

.modal-content h3 {
  margin: 0 0 16px 0;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  font-size: 14px;
}

.post-textarea,
.form-input,
.form-select {
  width: 100%;
  padding: 10px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  font-size: 14px;
  font-family: inherit;
}

.dark-mode .post-textarea,
.dark-mode .form-input,
.dark-mode .form-select {
  background: #3a3a3a;
  border-color: #404040;
  color: #fff;
}

.post-textarea {
  resize: none;
  min-height: 80px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.primary-button,
.secondary-button {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
}

.primary-button {
  background: #007bff;
  color: white;
}

.primary-button:hover {
  background: #0056b3;
}

.primary-button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.secondary-button {
  background: #e0e0e0;
  color: #333;
}

.dark-mode .secondary-button {
  background: #404040;
  color: #fff;
}

.secondary-button:hover {
  background: #d0d0d0;
}

.dark-mode .secondary-button:hover {
  background: #505050;
}

.comments-list {
  max-height: 400px;
  overflow-y: auto;
  margin-bottom: 16px;
}

.comment-card {
  padding: 12px;
  border-bottom: 1px solid #e0e0e0;
}

.dark-mode .comment-card {
  border-bottom-color: #404040;
}

.comment-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.comment-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  object-fit: cover;
}

.comment-author-info {
  flex: 1;
  margin-left: 8px;
}

.comment-author-name {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
}

.comment-time {
  margin: 0;
  font-size: 11px;
  color: #666;
}

.dark-mode .comment-time {
  color: #aaa;
}

.comment-menu-button {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  color: #666;
}

.comment-text {
  margin: 0;
  line-height: 1.5;
  font-size: 14px;
}

.add-comment {
  display: flex;
  gap: 8px;
}

.comment-input {
  flex: 1;
  padding: 10px;
  border: 1px solid #e0e0e0;
  border-radius: 20px;
  font-size: 14px;
}

.dark-mode .comment-input {
  background: #3a3a3a;
  border-color: #404040;
  color: #fff;
}

.send-comment-button {
  width: 40px;
  height: 40px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.send-comment-button:hover {
  background: #0056b3;
}

.send-comment-button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.context-menu {
  position: fixed;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  z-index: 1001;
  min-width: 120px;
}

.dark-mode .context-menu {
  background: #2a2a2a;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 16px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  color: #333;
  text-align: left;
}

.dark-mode .context-menu-item {
  color: #fff;
}

.context-menu-item:hover {
  background: #f5f5f5;
}

.dark-mode .context-menu-item:hover {
  background: #3a3a3a;
}

.context-menu-item.danger {
  color: #e91e63;
}

.context-menu-item.danger:hover {
  background: #fce4ec;
}
</style>
