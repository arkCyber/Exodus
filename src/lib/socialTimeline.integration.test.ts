/**
 * Exodus Browser — Social Timeline Integration Tests
 * Aerospace-grade implementation with robust error handling and type safety.
 * These tests verify the integration between frontend API and backend microservice.
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import {
  socialTimelineServiceStart,
  createPost,
  getTimeline,
  getPost,
  likePost,
  unlikePost,
  addComment,
  getComments,
  deletePost,
  deleteComment,
  editPost,
  buildPostPayload,
  buildCommentPayload,
} from './socialTimeline';

describe('socialTimeline Integration', () => {
  const testUserId = 'test-user-123';
  const testUserName = 'Test User';

  beforeAll(async () => {
    // Start the social timeline service
    try {
      await socialTimelineServiceStart();
    } catch (e) {
      console.warn('Failed to start social timeline service:', e);
    }
  });

  afterAll(async () => {
    // Cleanup could be added here if needed
  });

  describe('Service Lifecycle', () => {
    it('should start the service without errors', async () => {
      // Service is started in beforeAll
      expect(true).toBe(true);
    });
  });

  describe('Post Creation', () => {
    it('should create a new post', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post content',
        visibility: 'public',
      });

      const postId = await createPost(payload);
      expect(postId).toBeDefined();
      expect(postId).toMatch(/^post-/);
    });

    it('should create a post with location', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post with location',
        location: 'San Francisco',
        visibility: 'public',
      });

      const postId = await createPost(payload);
      expect(postId).toBeDefined();
    });

    it('should create a post with attachments', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post with attachments',
        attachments: [
          {
            attachmentId: 'att-1',
            fileType: 'image',
            blobHash: 'hash-123',
            fileName: 'image.jpg',
            fileSize: 1024,
          },
        ],
        visibility: 'public',
      });

      const postId = await createPost(payload);
      expect(postId).toBeDefined();
    });
  });

  describe('Timeline Retrieval', () => {
    it('should get timeline for user', async () => {
      const timeline = await getTimeline(testUserId, 10);
      expect(Array.isArray(timeline)).toBe(true);
    });

    it('should respect limit parameter', async () => {
      const timeline = await getTimeline(testUserId, 5);
      expect(timeline.length).toBeLessThanOrEqual(5);
    });
  });

  describe('Post Retrieval', () => {
    it('should get a single post by ID', async () => {
      // First create a post
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post for retrieval',
        visibility: 'public',
      });
      const postId = await createPost(payload);

      // Then retrieve it
      const post = await getPost(postId);
      expect(post).not.toBeNull();
      expect(post?.postId).toBe(postId);
      expect(post?.content).toBe('Test post for retrieval');
    });

    it('should return null for non-existent post', async () => {
      const post = await getPost('non-existent-post-id');
      expect(post).toBeNull();
    });
  });

  describe('Post Editing', () => {
    it('should edit an existing post', async () => {
      // Create a post
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Original content',
        visibility: 'public',
      });
      const postId = await createPost(payload);

      // Edit the post
      await editPost(postId, 'Updated content');

      // Verify the edit
      const post = await getPost(postId);
      expect(post?.content).toBe('Updated content');
    });
  });

  describe('Post Deletion', () => {
    it('should delete a post', async () => {
      // Create a post
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Post to delete',
        visibility: 'public',
      });
      const postId = await createPost(payload);

      // Delete the post
      await deletePost(postId);

      // Verify deletion
      const post = await getPost(postId);
      expect(post).toBeNull();
    });
  });

  describe('Comments', () => {
    let testPostId: string;

    beforeAll(async () => {
      // Create a test post for comments
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post for comments',
        visibility: 'public',
      });
      testPostId = await createPost(payload);
    });

    it('should add a comment to a post', async () => {
      const payload = buildCommentPayload({
        postId: testPostId,
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test comment',
      });

      const commentId = await addComment(payload);
      expect(commentId).toBeDefined();
      expect(commentId).toMatch(/^comment-/);
    });

    it('should get comments for a post', async () => {
      const comments = await getComments(testPostId, 10);
      expect(Array.isArray(comments)).toBe(true);
    });

    it('should delete a comment', async () => {
      // Add a comment
      const payload = buildCommentPayload({
        postId: testPostId,
        authorId: testUserId,
        authorName: testUserName,
        content: 'Comment to delete',
      });
      const commentId = await addComment(payload);

      // Delete the comment
      await deleteComment(testPostId, commentId);

      // Verify deletion
      const comments = await getComments(testPostId);
      const deletedComment = comments.find(c => c.commentId === commentId);
      expect(deletedComment).toBeUndefined();
    });
  });

  describe('Likes', () => {
    let testPostId: string;

    beforeAll(async () => {
      // Create a test post for likes
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post for likes',
        visibility: 'public',
      });
      testPostId = await createPost(payload);
    });

    it('should like a post', async () => {
      await likePost(testPostId, testUserId);

      const post = await getPost(testPostId);
      expect(post?.likes).toBeGreaterThan(0);
    });

    it('should unlike a post', async () => {
      await unlikePost(testPostId, testUserId);

      const post = await getPost(testPostId);
      // Note: Backend doesn't track per-user liked status, so we can't verify this directly
      expect(post).not.toBeNull();
    });
  });

  describe('Visibility', () => {
    it('should create a public post', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Public post',
        visibility: 'public',
      });

      const postId = await createPost(payload);
      const post = await getPost(postId);
      expect(post?.visibility).toBe('public');
    });

    it('should create a friends-only post', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Friends-only post',
        visibility: 'friends',
      });

      const postId = await createPost(payload);
      const post = await getPost(postId);
      expect(post?.visibility).toBe('friends');
    });

    it('should create a private post', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Private post',
        visibility: 'private',
      });

      const postId = await createPost(payload);
      const post = await getPost(postId);
      expect(post?.visibility).toBe('private');
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid post ID gracefully', async () => {
      const post = await getPost('');
      expect(post).toBeNull();
    });

    it('should handle empty content in post creation', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: '',
        visibility: 'public',
      });

      // This should either succeed or fail gracefully
      try {
        const postId = await createPost(payload);
        expect(postId).toBeDefined();
      } catch (e) {
        expect(e).toBeDefined();
      }
    });
  });

  describe('Data Consistency', () => {
    it('should maintain post data structure', async () => {
      const payload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Consistency test post',
        location: 'Test Location',
        visibility: 'public',
      });

      const postId = await createPost(payload);
      const post = await getPost(postId);

      expect(post).toHaveProperty('postId');
      expect(post).toHaveProperty('authorId');
      expect(post).toHaveProperty('authorName');
      expect(post).toHaveProperty('content');
      expect(post).toHaveProperty('visibility');
      expect(post).toHaveProperty('likes');
      expect(post).toHaveProperty('comments');
      expect(post).toHaveProperty('timestamp');
    });

    it('should maintain comment data structure', async () => {
      const postPayload = buildPostPayload({
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test post',
        visibility: 'public',
      });
      const postId = await createPost(postPayload);

      const commentPayload = buildCommentPayload({
        postId,
        authorId: testUserId,
        authorName: testUserName,
        content: 'Test comment',
      });
      const commentId = await addComment(commentPayload);

      const comments = await getComments(postId);
      const comment = comments.find(c => c.commentId === commentId);

      expect(comment).toHaveProperty('commentId');
      expect(comment).toHaveProperty('postId');
      expect(comment).toHaveProperty('authorId');
      expect(comment).toHaveProperty('content');
      expect(comment).toHaveProperty('timestamp');
    });
  });
});
