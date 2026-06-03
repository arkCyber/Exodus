<script lang="ts">
  /**
   * Exodus Browser — bookmark bar with folders, overflow menu, drag-reorder, and drag-to-folder.
   */
  import type { BookmarkItem } from '$lib/browserTypes';
  import { faviconUrlFor } from '$lib/favicon';
  import { BOOKMARK_BAR_MAX } from '$lib/bookmarks';

  type Props = {
    visible: boolean;
    barBookmarks: BookmarkItem[];
    folderNames: string[];
    bookmarks: BookmarkItem[];
    onNavigate: (url: string) => void;
    onReorder?: (orderedIds: string[]) => void | Promise<void>;
    onMoveToFolder?: (bookmarkId: string, folder: string) => void | Promise<void>;
  };

  let {
    visible,
    barBookmarks,
    folderNames,
    bookmarks,
    onNavigate,
    onReorder,
    onMoveToFolder,
  }: Props = $props();

  let openFolderMenu = $state<string | null>(null);
  let showBookmarkOverflow = $state(false);
  let dragId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);
  let dragOverFolder = $state<string | null>(null);
  let dragOverBar = $state(false);
  let dragDepth = $state(0);

  /** Bookmarks inside a named folder. */
  function inFolder(name: string): BookmarkItem[] {
    return bookmarks.filter((b) => b.folder === name);
  }

  function clearDragHighlights() {
    dragOverId = null;
    dragOverFolder = null;
    dragOverBar = false;
  }

  function onDragEnterZone() {
    dragDepth += 1;
  }

  function onDragLeaveZone(event: DragEvent) {
    const related = event.relatedTarget as Node | null;
    if (related && event.currentTarget instanceof Node && event.currentTarget.contains(related)) {
      return;
    }
    dragDepth = Math.max(0, dragDepth - 1);
    if (dragDepth === 0) clearDragHighlights();
  }

  function onDragStart(id: string, event: DragEvent) {
    dragId = id;
    dragDepth = 0;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', id);
    }
  }

  function onDragOverChip(id: string, event: DragEvent) {
    event.preventDefault();
    if (dragId && dragId !== id) dragOverId = id;
  }

  function onDragOverFolderName(folderName: string, event: DragEvent) {
    event.preventDefault();
    if (!dragId || !onMoveToFolder) return;
    const dragged = bookmarks.find((b) => b.id === dragId);
    if (dragged?.folder === folderName) return;
    dragOverFolder = folderName;
  }

  function onDragOverBarZone(event: DragEvent) {
    event.preventDefault();
    if (dragId && onMoveToFolder) dragOverBar = true;
  }

  async function onDropChip(targetId: string, event: DragEvent) {
    event.preventDefault();
    const fromId = dragId ?? event.dataTransfer?.getData('text/plain');
    onDragEnd();
    if (!fromId || fromId === targetId || !onReorder) return;

    const ids = barBookmarks.map((b) => b.id);
    const fromIdx = ids.indexOf(fromId);
    const toIdx = ids.indexOf(targetId);
    if (toIdx < 0) return;

    if (fromIdx < 0) {
      const next = ids.filter((id) => id !== fromId);
      next.splice(toIdx, 0, fromId);
      await onReorder(next);
      return;
    }

    const next = [...ids];
    next.splice(fromIdx, 1);
    next.splice(toIdx, 0, fromId);
    await onReorder(next);
  }

  async function onDropFolder(folderName: string, event: DragEvent) {
    event.preventDefault();
    const fromId = dragId ?? event.dataTransfer?.getData('text/plain');
    onDragEnd();
    if (!fromId || !onMoveToFolder) return;
    const dragged = bookmarks.find((b) => b.id === fromId);
    if (dragged?.folder === folderName) return;
    await onMoveToFolder(fromId, folderName);
  }

  async function onDropBar(event: DragEvent) {
    event.preventDefault();
    const fromId = dragId ?? event.dataTransfer?.getData('text/plain');
    onDragEnd();
    if (!fromId || !onMoveToFolder) return;
    await onMoveToFolder(fromId, '');
  }

  function onDragEnd() {
    dragId = null;
    dragDepth = 0;
    clearDragHighlights();
  }
</script>

{#if visible && (barBookmarks.length > 0 || folderNames.length > 0)}
  <div
    class="bookmark-bar exodus-bookmark-bar"
    class:drag-over-bar={dragOverBar}
    role="navigation"
    aria-label="Bookmark bar"
    ondragenter={onDragEnterZone}
    ondragover={onDragOverBarZone}
    ondragleave={onDragLeaveZone}
    ondrop={(e) => void onDropBar(e)}
  >
    {#each barBookmarks.slice(0, BOOKMARK_BAR_MAX) as bm (bm.id)}
      <button
        type="button"
        class="bookmark-chip"
        class:drag-over={dragOverId === bm.id}
        draggable={Boolean(onReorder || onMoveToFolder)}
        ondragstart={(e) => onDragStart(bm.id, e)}
        ondragenter={onDragEnterZone}
        ondragover={(e) => onDragOverChip(bm.id, e)}
        ondragleave={onDragLeaveZone}
        ondrop={(e) => void onDropChip(bm.id, e)}
        ondragend={onDragEnd}
        onclick={() => onNavigate(bm.url)}
        title={bm.url}
      >
        {#if faviconUrlFor(bm.url)}
          <img class="bm-favicon" src={faviconUrlFor(bm.url)} alt="" width="16" height="16" />
        {/if}
        <span class="bm-label">{bm.title || bm.url}</span>
      </button>
    {/each}
    {#each folderNames as folderName (folderName)}
      <div class="bookmark-folder-wrap">
        <button
          type="button"
          class="bookmark-chip folder"
          class:drag-over={dragOverFolder === folderName}
          ondragenter={onDragEnterZone}
          ondragover={(e) => onDragOverFolderName(folderName, e)}
          ondragleave={onDragLeaveZone}
          ondrop={(e) => void onDropFolder(folderName, e)}
          onclick={() => (openFolderMenu = openFolderMenu === folderName ? null : folderName)}
          title="Drop bookmark here to move into folder"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"
            ><path d="M10 4H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-8l-2-2z" /></svg
          >
          <span class="bm-label">{folderName}</span>
        </button>
        {#if openFolderMenu === folderName}
          <button type="button" class="menu-backdrop" aria-label="Close" onclick={() => (openFolderMenu = null)}></button>
          <div
            class="bookmark-overflow-menu folder-menu"
            ondragenter={onDragEnterZone}
            ondragleave={onDragLeaveZone}
          >
            {#each inFolder(folderName) as bm (bm.id)}
              <button
                type="button"
                class="bookmark-overflow-item"
                draggable={Boolean(onMoveToFolder || onReorder)}
                ondragstart={(e) => onDragStart(bm.id, e)}
                ondragend={onDragEnd}
                onclick={() => {
                  openFolderMenu = null;
                  onNavigate(bm.url);
                }}
              >
                {#if faviconUrlFor(bm.url)}
                  <img class="bm-favicon" src={faviconUrlFor(bm.url)} alt="" width="16" height="16" />
                {/if}
                <span>{bm.title || bm.url}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
    {#if barBookmarks.length > BOOKMARK_BAR_MAX}
      <div class="bookmark-overflow-wrap">
        <button
          type="button"
          class="bookmark-overflow-btn"
          title="More bookmarks"
          onclick={() => (showBookmarkOverflow = !showBookmarkOverflow)}
          >>></button
        >
        {#if showBookmarkOverflow}
          <button type="button" class="menu-backdrop" aria-label="Close" onclick={() => (showBookmarkOverflow = false)}></button>
          <div class="bookmark-overflow-menu">
            {#each barBookmarks.slice(BOOKMARK_BAR_MAX) as bm (bm.id)}
              <button
                type="button"
                class="bookmark-overflow-item"
                draggable={Boolean(onReorder || onMoveToFolder)}
                ondragstart={(e) => onDragStart(bm.id, e)}
                ondragend={onDragEnd}
                onclick={() => {
                  showBookmarkOverflow = false;
                  onNavigate(bm.url);
                }}
              >
                {#if faviconUrlFor(bm.url)}
                  <img class="bm-favicon" src={faviconUrlFor(bm.url)} alt="" width="16" height="16" />
                {/if}
                <span>{bm.title || bm.url}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .bookmark-bar {
    display: flex;
    gap: 6px;
    padding: 4px 12px 6px;
    background: #222;
    border-bottom: 1px solid #333;
    overflow-x: auto;
    min-height: 32px;
  }

  .bookmark-bar.drag-over-bar {
    outline: 1px dashed #6366f1;
    outline-offset: -2px;
  }

  .bookmark-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: #333;
    border: 1px solid #444;
    color: #ccc;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    cursor: grab;
    white-space: nowrap;
    max-width: 180px;
  }

  .bookmark-chip.drag-over {
    border-color: #6366f1;
    box-shadow: 0 0 0 1px #6366f1;
  }

  .bookmark-chip:hover {
    background: #404040;
    border-color: #6366f1;
  }

  .bm-favicon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    border-radius: 2px;
  }

  .bm-label {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bookmark-folder-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .bookmark-chip.folder {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }

  .bookmark-chip.folder.drag-over {
    background: #3a3a5a;
  }

  .bookmark-overflow-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .bookmark-overflow-btn {
    background: #333;
    border: 1px solid #444;
    color: #aaa;
    padding: 4px 10px;
    border-radius: 12px;
    cursor: pointer;
    font-size: 12px;
  }

  .bookmark-overflow-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    min-width: 220px;
    max-height: 320px;
    overflow-y: auto;
    background: #2d2d2d;
    border: 1px solid #555;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 200;
    padding: 4px;
  }

  .bookmark-overflow-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    border: none;
    color: #eee;
    text-align: left;
    cursor: pointer;
    border-radius: 6px;
    font-size: 13px;
  }

  .bookmark-overflow-item:hover {
    background: #404040;
  }
</style>
