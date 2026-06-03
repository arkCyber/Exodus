/**
 * Exodus Browser — bookmark bar UI string tests.
 */

import { describe, expect, it } from 'vitest';
import { bookmarkBarStrings, resolveBookmarkBarLocale } from './bookmarkBarUi';

describe('bookmarkBarUi', () => {
  it('resolveBookmarkBarLocale honors explicit locale', () => {
    expect(resolveBookmarkBarLocale('zh')).toBe('zh');
    expect(resolveBookmarkBarLocale('en')).toBe('en');
  });

  it('returns English strings by default in test env', () => {
    const strings = bookmarkBarStrings('en');
    expect(strings.allBookmarks).toBe('All bookmarks');
    expect(strings.openBookmarksManager).toBe('Open bookmarks manager');
  });

  it('returns Chinese strings for zh locale', () => {
    const strings = bookmarkBarStrings('zh');
    expect(strings.allBookmarks).toBe('所有书签');
    expect(strings.openBookmarksManager).toBe('打开书签管理器');
    expect(strings.sidePanel).toBe('侧边栏');
    expect(strings.apps).toBe('应用');
    expect(strings.contextDelete).toBe('删除');
    expect(strings.barAriaLabel).toBe('书签栏');
  });

  it('formats folder title', () => {
    expect(bookmarkBarStrings('en').folderTitle('Work')).toBe('Folder: Work');
    expect(bookmarkBarStrings('zh').folderTitle('工作')).toBe('文件夹：工作');
  });

  it('formats bookmark group count in Chinese', () => {
    expect(bookmarkBarStrings('zh').groupBookmarkCount(1)).toBe('1 个书签');
    expect(bookmarkBarStrings('zh').createNewGroup).toBe('创建新书签分组');
  });

  it('maps validation error codes to English messages', () => {
    const strings = bookmarkBarStrings('en');
    expect(strings.groupNameError('reserved')).toContain('reserved');
    expect(strings.groupNameError('exists')).toContain('already exists');
    expect(strings.groupNameError('too_long')).toContain('64');
  });
});
