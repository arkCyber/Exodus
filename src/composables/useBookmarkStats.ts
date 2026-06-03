/**
 * Exodus Browser — Bookmark Statistics Composable
 * 
 * This composable provides statistics and analytics for bookmarks.
 */

import { computed } from 'vue';
import { useBookmarks } from './useBookmarks';
import { useTags } from './useTags';

export interface BookmarkStats {
  totalBookmarks: number;
  totalFolders: number;
  totalTags: number;
  bookmarksByFolder: Record<string, number>;
  bookmarksByTag: Record<string, number>;
  untaggedBookmarks: number;
  recentlyAdded: number;
  mostUsedTags: Array<{ name: string; count: number }>;
  largestFolders: Array<{ name: string; count: number }>;
}

export function useBookmarkStats() {
  const { bookmarks, folders } = useBookmarks();
  const { tags, getTagsForBookmark } = useTags();

  const totalBookmarks = computed(() => bookmarks.value.length);

  const totalFolders = computed(() => folders.value.length);

  const totalTags = computed(() => tags.value.length);

  const bookmarksByFolder = computed(() => {
    const stats: Record<string, number> = {};
    
    bookmarks.value.forEach(bookmark => {
      const folder = bookmark.folder || 'No folder';
      stats[folder] = (stats[folder] || 0) + 1;
    });
    
    return stats;
  });

  const bookmarksByTag = computed(() => {
    const stats: Record<string, number> = {};
    
    bookmarks.value.forEach(bookmark => {
      const bookmarkTags = getTagsForBookmark(bookmark.id);
      bookmarkTags.forEach(tag => {
        stats[tag.name] = (stats[tag.name] || 0) + 1;
      });
    });
    
    return stats;
  });

  const untaggedBookmarks = computed(() => {
    return bookmarks.value.filter(bookmark => {
      const bookmarkTags = getTagsForBookmark(bookmark.id);
      return bookmarkTags.length === 0;
    }).length;
  });

  const recentlyAdded = computed(() => {
    const oneWeekAgo = Date.now() - 7 * 24 * 60 * 60 * 1000;
    return bookmarks.value.filter(bookmark => {
      const createdAt = bookmark.createdAt || bookmark.created_at;
      if (!createdAt) return false;
      const timestamp = typeof createdAt === 'string' 
        ? new Date(createdAt).getTime() 
        : createdAt;
      return timestamp > oneWeekAgo;
    }).length;
  });

  const mostUsedTags = computed(() => {
    const tagCounts = bookmarksByTag.value;
    return Object.entries(tagCounts)
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
  });

  const largestFolders = computed(() => {
    const folderCounts = bookmarksByFolder.value;
    return Object.entries(folderCounts)
      .map(([name, count]) => ({ name, count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10);
  });

  const stats = computed<BookmarkStats>(() => ({
    totalBookmarks: totalBookmarks.value,
    totalFolders: totalFolders.value,
    totalTags: totalTags.value,
    bookmarksByFolder: bookmarksByFolder.value,
    bookmarksByTag: bookmarksByTag.value,
    untaggedBookmarks: untaggedBookmarks.value,
    recentlyAdded: recentlyAdded.value,
    mostUsedTags: mostUsedTags.value,
    largestFolders: largestFolders.value,
  }));

  // Helper function to get bookmark distribution by folder as percentage
  const getFolderDistribution = computed(() => {
    const total = totalBookmarks.value;
    if (total === 0) return [];
    
    return Object.entries(bookmarksByFolder.value)
      .map(([name, count]) => ({
        name,
        count,
        percentage: (count / total) * 100,
      }))
      .sort((a, b) => b.count - a.count);
  });

  // Helper function to get tag distribution as percentage
  const getTagDistribution = computed(() => {
    const total = Object.values(bookmarksByTag.value).reduce((sum, count) => sum + count, 0);
    if (total === 0) return [];
    
    return Object.entries(bookmarksByTag.value)
      .map(([name, count]) => ({
        name,
        count,
        percentage: (count / total) * 100,
      }))
      .sort((a, b) => b.count - a.count);
  });

  // Get bookmarks without any folder
  const bookmarksWithoutFolder = computed(() => {
    return bookmarks.value.filter(bookmark => !bookmark.folder);
  });

  // Get bookmarks with multiple tags
  const bookmarksWithMultipleTags = computed(() => {
    return bookmarks.value.filter(bookmark => {
      const bookmarkTags = getTagsForBookmark(bookmark.id);
      return bookmarkTags.length > 1;
    }).length;
  });

  // Average tags per bookmark
  const averageTagsPerBookmark = computed(() => {
    const totalTags = Object.values(bookmarksByTag.value).reduce((sum, count) => sum + count, 0);
    const totalBookmarksCount = totalBookmarks.value;
    if (totalBookmarksCount === 0) return 0;
    return (totalTags / totalBookmarksCount).toFixed(2);
  });

  return {
    // Raw stats
    totalBookmarks,
    totalFolders,
    totalTags,
    bookmarksByFolder,
    bookmarksByTag,
    untaggedBookmarks,
    recentlyAdded,
    mostUsedTags,
    largestFolders,
    
    // Computed stats object
    stats,
    
    // Helper distributions
    getFolderDistribution,
    getTagDistribution,
    
    // Additional metrics
    bookmarksWithoutFolder,
    bookmarksWithMultipleTags,
    averageTagsPerBookmark,
  };
}
