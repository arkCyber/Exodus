/**
 * Exodus Browser — Chrome bookmark bar UI strings (locale-aware).
 */

import { logStartup } from '@/lib/startupLog';
import type { BookmarkGroupValidationError } from './bookmarkGroups';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('bookmarkBarUi module loaded');

/** Bookmark bar UI locale (subset of AppLocale; non-zh uses English copy). */
export type BookmarkBarLocale = AppLocale;

export interface BookmarkBarStrings {
  barAriaLabel: string;
  sidePanel: string;
  apps: string;
  bookmarkGroups: string;
  createNewGroup: string;
  createGroupShortcut: string;
  groupBookmarkCount: (count: number) => string;
  closeGroupsMenu: string;
  newGroupDialogTitle: string;
  newGroupNameLabel: string;
  newGroupColorLabel: string;
  newGroupSave: string;
  newGroupCancel: string;
  groupNameError: (code: BookmarkGroupValidationError) => string;
  contextMoveToGroup: string;
  moreBookmarks: string;
  allBookmarks: string;
  openBookmarksManager: string;
  closeFolderMenu: string;
  closeOverflowMenu: string;
  closeAllBookmarksMenu: string;
  emptyFolder: string;
  emptyAllBookmarks: string;
  openBookmarksSidebar: string;
  folderTitle: (name: string) => string;
  contextOpen: string;
  contextOpenInNewTab: string;
  contextOpenInNewWindow: string;
  contextOpenInIncognito: string;
  contextEdit: string;
  contextDelete: string;
  contextCopyUrl: string;
  contextAddBookmark: string;
  contextBookmarkManager: string;
  contextShowBookmarkBar: string;
  contextHideBookmarkBar: string;
}

/** @deprecated Import from `@/lib/bookmarkGroups` — re-export for compatibility. */
export { RESERVED_BOOKMARK_BAR_FOLDERS } from './bookmarkGroups';

const EN_STRINGS: BookmarkBarStrings = {
  barAriaLabel: 'Bookmark bar',
  sidePanel: 'Side panel',
  apps: 'Apps',
  bookmarkGroups: 'Bookmark groups',
  createNewGroup: 'Create new bookmark group',
  createGroupShortcut: '⌃⌘P',
  groupBookmarkCount: (count) =>
    count === 1 ? '1 bookmark' : `${count} bookmarks`,
  closeGroupsMenu: 'Close bookmark groups menu',
  newGroupDialogTitle: 'New bookmark group',
  newGroupNameLabel: 'Name',
  newGroupColorLabel: 'Color',
  newGroupSave: 'Create',
  newGroupCancel: 'Cancel',
  groupNameError: (code) => {
    switch (code) {
      case 'empty':
        return 'Enter a group name.';
      case 'reserved':
        return 'This name is reserved. Choose another name.';
      case 'exists':
        return 'This group already exists.';
      case 'too_long':
        return 'Name is too long (max 64 characters).';
      case 'invalid_chars':
        return 'Name contains invalid characters.';
      default:
        return 'Invalid group name.';
    }
  },
  contextMoveToGroup: 'Move to group',
  moreBookmarks: 'More bookmarks',
  allBookmarks: 'All bookmarks',
  openBookmarksManager: 'Open bookmarks manager',
  closeFolderMenu: 'Close folder menu',
  closeOverflowMenu: 'Close overflow menu',
  closeAllBookmarksMenu: 'Close all bookmarks menu',
  emptyFolder: 'This folder is empty',
  emptyAllBookmarks: 'No bookmarks yet',
  openBookmarksSidebar: 'Open bookmarks sidebar',
  folderTitle: (name) => `Folder: ${name}`,
  contextOpen: 'Open',
  contextOpenInNewTab: 'Open in new tab',
  contextOpenInNewWindow: 'Open in new window',
  contextOpenInIncognito: 'Open in incognito window',
  contextEdit: 'Edit…',
  contextDelete: 'Delete',
  contextCopyUrl: 'Copy URL',
  contextAddBookmark: 'Add page…',
  contextBookmarkManager: 'Bookmark manager',
  contextShowBookmarkBar: 'Show bookmarks bar',
  contextHideBookmarkBar: 'Hide bookmarks bar',
};

const ZH_STRINGS: BookmarkBarStrings = {
  barAriaLabel: '书签栏',
  sidePanel: '侧边栏',
  apps: '应用',
  bookmarkGroups: '书签分组',
  createNewGroup: '创建新书签分组',
  createGroupShortcut: '⌃⌘P',
  groupBookmarkCount: (count) => `${count} 个书签`,
  closeGroupsMenu: '关闭书签分组菜单',
  newGroupDialogTitle: '新建书签分组',
  newGroupNameLabel: '名称',
  newGroupColorLabel: '颜色',
  newGroupSave: '创建',
  newGroupCancel: '取消',
  groupNameError: (code) => {
    switch (code) {
      case 'empty':
        return '请输入分组名称。';
      case 'reserved':
        return '该名称为系统保留，请换一个名称。';
      case 'exists':
        return '该分组已存在。';
      case 'too_long':
        return '名称过长（最多 64 个字符）。';
      case 'invalid_chars':
        return '名称包含无效字符。';
      default:
        return '分组名称无效。';
    }
  },
  contextMoveToGroup: '移至分组',
  moreBookmarks: '更多书签',
  allBookmarks: '所有书签',
  openBookmarksManager: '打开书签管理器',
  closeFolderMenu: '关闭文件夹菜单',
  closeOverflowMenu: '关闭溢出菜单',
  closeAllBookmarksMenu: '关闭所有书签菜单',
  emptyFolder: '此文件夹为空',
  emptyAllBookmarks: '暂无书签',
  openBookmarksSidebar: '打开书签侧边栏',
  folderTitle: (name) => `文件夹：${name}`,
  contextOpen: '打开',
  contextOpenInNewTab: '在新标签页中打开',
  contextOpenInNewWindow: '在新窗口中打开',
  contextOpenInIncognito: '在隐身窗口中打开',
  contextEdit: '修改…',
  contextDelete: '删除',
  contextCopyUrl: '复制网址',
  contextAddBookmark: '添加网页…',
  contextBookmarkManager: '书签管理器',
  contextShowBookmarkBar: '显示书签栏',
  contextHideBookmarkBar: '隐藏书签栏',
};

/**
 * Resolve bookmark bar locale from browser language or explicit override.
 */
export function resolveBookmarkBarLocale(explicit?: BookmarkBarLocale): BookmarkBarLocale {
  return resolveAppLocale(explicit);
}

/**
 * Localized bookmark bar strings for Chrome parity labels.
 * Full Chinese copy; other locales use English until extended.
 */
export function bookmarkBarStrings(locale?: BookmarkBarLocale): BookmarkBarStrings {
  return resolveBookmarkBarLocale(locale) === 'zh' ? ZH_STRINGS : EN_STRINGS;
}
