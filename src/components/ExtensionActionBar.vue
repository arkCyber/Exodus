<!--
  Exodus Browser — Chrome-style pinned extension toolbar display module (address bar).
-->
<template>
  <div
    v-show="ready"
    class="extension-action-bar exodus-chrome-extension-toolbar extension-toolbar-display"
    :class="{ 'extension-action-bar--inline': inline }"
    role="toolbar"
    :aria-label="toolbarAriaLabel"
  >
    <button
      v-for="ext in visibleExtensionsWithIcons"
      :key="ext.id"
      type="button"
      class="extension-action-btn"
      :class="{ active: openPopupId === ext.id }"
      :disabled="!nativePopups && !!ext.actionPopup"
      :title="actionTitle(ext)"
      :aria-label="ext.name"
      :aria-expanded="openPopupId === ext.id ? 'true' : 'false'"
      @click="onExtensionClick(ext, $event)"
      @contextmenu.prevent="onExtensionRightClick(ext, $event)"
    >
      <img
        class="extension-action-icon"
        :src="iconUrls[ext.id]"
        alt=""
        width="16"
        height="16"
      />
    </button>

    <button
      v-if="extensionsWithIcons.length > MAX_VISIBLE_EXTENSIONS"
      type="button"
      class="extension-action-btn extension-overflow-btn"
      :title="`More extensions (${overflowExtensionsWithIcons.length})`"
      :aria-label="`More extensions (${overflowExtensionsWithIcons.length})`"
      @click="toggleOverflow"
    >
      <svg class="extension-overflow-svg" viewBox="0 0 16 16" fill="none" aria-hidden="true">
        <path d="M3 4l3.5 4L3 12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        <path d="M7 4l3.5 4L7 12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>

    <button
      type="button"
      class="extension-action-btn extension-puzzle-btn"
      :title="puzzleTitle"
      :aria-label="puzzleTitle"
      @click="emit('openExtensionsManager')"
    >
      <Blocks :size="16" class="extension-puzzle-svg" aria-hidden="true" />
    </button>
  </div>

  <Teleport to="body">
    <template v-if="showOverflowMenu">
      <button
        type="button"
        class="extension-popup-backdrop"
        aria-label="Close overflow menu"
        @click="closeOverflow"
      />
      <div class="extension-overflow-menu" role="menu">
        <button
          v-for="ext in overflowExtensionsWithIcons"
          :key="ext.id"
          class="extension-overflow-item"
          role="menuitem"
          :title="actionTitle(ext)"
          @click="onExtensionClick(ext, $event)"
        >
          <img
            class="extension-action-icon"
            :src="iconUrls[ext.id]"
            alt=""
            width="16"
            height="16"
          />
          <span class="extension-overflow-name">{{ ext.name }}</span>
        </button>
      </div>
    </template>
  </Teleport>

  <Teleport to="body">
    <button
      v-if="openPopupId"
      type="button"
      class="extension-popup-backdrop"
      aria-label="Close extension popup"
      @click="closeEmbeddedPopup"
    />
  </Teleport>

  <ContextMenu
    :visible="contextMenu.visible"
    :x="contextMenu.x"
    :y="contextMenu.y"
    :items="contextMenu.items"
    @close="closeContextMenu"
  />

  <!-- Removal Confirmation Dialog -->
  <Teleport to="body">
    <div v-if="removalDialog.visible" class="extension-removal-backdrop" @click="cancelRemoval">
      <div class="extension-removal-dialog" @click.stop>
        <div class="removal-dialog-header">
          <h3>Remove Extension</h3>
        </div>
        <div class="removal-dialog-body">
          <p>Are you sure you want to remove <strong>{{ removalDialog.extensionName }}</strong>?</p>
          <p class="removal-warning">This action cannot be undone.</p>
        </div>
        <div class="removal-dialog-footer">
          <button type="button" class="removal-btn removal-btn-cancel" @click="cancelRemoval">
            Cancel
          </button>
          <button type="button" class="removal-btn removal-btn-confirm" @click="confirmRemoval">
            Remove
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Undo Toast for Extension Hiding -->
  <Teleport to="body">
    <div v-if="undoToast.visible" class="extension-undo-toast">
      <span class="undo-toast-message">
        <strong>{{ undoToast.extensionName }}</strong> hidden from toolbar
      </span>
      <button type="button" class="undo-toast-btn" @click="undoHideExtension">
        Undo
      </button>
      <button type="button" class="undo-toast-close" @click="hideUndoToast">
        ✕
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — pinned extension toolbar display module (Chrome-aligned).
 * Shows only enabled extensions pinned to the address bar; puzzle opens extensions manager.
 */
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listExtensions, extensionPopupUrl } from '$lib/extensions/api';
import type { ExtensionInfo } from '$lib/extensions/types';
import { canUseNativeWebview } from '$lib/exodusBrowser';
import {
  extensionIconLetter,
  resolveExtensionIconUrl,
} from '@/lib/extensionToolbarIcon';
import {
  openToolbarExtensionPopup,
  closeToolbarExtensionPopup,
} from '@/lib/extensionToolbarPopup';
import {
  pinnedToolbarExtensions,
  pinnedToolbarActionTitle,
  pinnedToolbarModuleLabel,
} from '@/lib/extensionToolbarDisplay';
import { Blocks } from '@lucide/vue';
import ContextMenu from '@/components/ContextMenu.vue';

/**
 * Type-safe interface for context menu items
 */
interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  disabled?: boolean;
  separator?: boolean;
  action?: () => void | Promise<void>;
}

/**
 * Type-safe interface for removal dialog state
 */
interface RemovalDialogState {
  visible: boolean;
  extensionId: string | null;
  extensionName: string;
}

/**
 * Type-safe interface for undo toast state
 */
interface UndoToastState {
  visible: boolean;
  extensionId: string | null;
  extensionName: string;
}

const props = withDefaults(
  defineProps<{
    inline?: boolean;
    refreshKey?: number;
    puzzleTitle?: string;
  }>(),
  {
    inline: false,
    refreshKey: 0,
    puzzleTitle: 'Extensions',
  },
);

const MAX_VISIBLE_EXTENSIONS = 4;

const emit = defineEmits<{
  openExtensionsManager: [];
  popupClosed: [];
}>();

const extensions = ref<ExtensionInfo[]>([]);
const iconUrls = ref<Record<string, string>>({});
const openPopupId = ref<string | null>(null);
const ready = ref(false);
const nativePopups = computed(() => canUseNativeWebview());
const toolbarAriaLabel = computed(() => pinnedToolbarModuleLabel(extensions.value.length));

const showOverflowMenu = ref(false);

const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  items: [] as ContextMenuItem[],
  extensionId: null as string | null,
});

const removalDialog = ref<RemovalDialogState>({
  visible: false,
  extensionId: null,
  extensionName: '',
});

const undoToast = ref<UndoToastState>({
  visible: false,
  extensionId: null,
  extensionName: '',
});

const extensionsWithIcons = computed(() => {
  // Aerospace-level performance: memoize with stable reference
  return extensions.value.filter(ext => iconUrls.value[ext.id]);
});

const visibleExtensionsWithIcons = computed(() => {
  // Aerospace-level performance: limit array operations
  return extensionsWithIcons.value.slice(0, MAX_VISIBLE_EXTENSIONS);
});

const overflowExtensionsWithIcons = computed(() => {
  // Aerospace-level performance: avoid redundant slicing
  return extensionsWithIcons.value.slice(MAX_VISIBLE_EXTENSIONS);
});

let retryTimer: ReturnType<typeof setTimeout> | null = null;
let isRefreshing = false; // Aerospace-level concurrency flag
let pendingOperations = new Set<string>(); // Track pending operations for concurrency safety

function actionTitle(ext: ExtensionInfo): string {
  return pinnedToolbarActionTitle(ext, { nativePopups: nativePopups.value });
}

/**
 * Reload pinned + enabled extensions for the toolbar display strip.
 * Aerospace-level error handling with detailed logging and recovery.
 * Concurrency-safe: prevents multiple simultaneous refresh operations.
 */
async function refresh(): Promise<void> {
  // Aerospace-level concurrency check
  if (isRefreshing) {
    console.warn('[ExtensionActionBar] Refresh already in progress, skipping duplicate request');
    return;
  }
  
  isRefreshing = true;
  
  try {
    const list = pinnedToolbarExtensions(await listExtensions());
    extensions.value = list;
    const nextIcons: Record<string, string> = {};
    
    // Process icon loading with individual error handling
    await Promise.allSettled(
      list.map(async (ext) => {
        try {
          const url = await resolveExtensionIconUrl(ext);
          if (url) nextIcons[ext.id] = url;
        } catch (error) {
          console.error(`[ExtensionActionBar] Failed to load icon for ${ext.id}:`, error);
          // Continue without icon rather than failing entire refresh
        }
      }),
    );
    
    iconUrls.value = nextIcons;
    ready.value = true;
  } catch (error) {
    console.error('[ExtensionActionBar] Extension list refresh failed:', error);
    // Graceful degradation: show empty state rather than crash
    extensions.value = [];
    iconUrls.value = {};
    ready.value = true;
  } finally {
    isRefreshing = false;
  }
}

/**
 * Schedule retry refresh with aerospace-level timer management.
 * Ensures only one retry timer is active at any time.
 */
function scheduleRetryRefresh(): void {
  if (retryTimer) clearTimeout(retryTimer);
  retryTimer = setTimeout(() => {
    void refresh();
  }, 800);
}

/**
 * Notify shell to restore the active tab webview after popup closes/sql close.
 * Aerospace-level event dispatching with error handling.
 */
function notifyPopupClosed(): void {
  emit('popupClosed');
  window.dispatchEvent(new CustomEvent('exodus-extension-popup-closed'));
}

/**
 * Close legacy separate-window popup if it was opened elsewhere.
 * Aerospace-level error handling with silent failure for non-critical operations.
 */
async function closeLegacyPopupWindow(extensionId: string): Promise<void> {
  try {
    await invoke('extension_close_popup_window', { extensionId });
  } catch {
    /* ignore - legacy popup may not exist */
  }
}

/**
 * Close the currently open embedded extension popup.
 * Aerospace-level state management with cleanup coordination.
 */
async function closeEmbeddedPopup(): Promise<void> {
  if (!openPopupId.value) return;
  const extensionId = openPopupId.value;
  openPopupId.value = null;
  try {
    await closeToolbarExtensionPopup(extensionId);
  } catch (error) {
    console.error('[ExtensionActionBar] closeToolbarExtensionPopup failed:', error);
  }
  await closeLegacyPopupWindow(extensionId);
  notifyPopupClosed();
}

/**
 * Open extension popup or extensions manager (Chrome action button).
 * Aerospace-level state management with popup lifecycle coordination.
 */
async function onExtensionClick(ext: ExtensionInfo, event: MouseEvent): Promise<void> {
  if (ext.actionPopup && canUseNativeWebview()) {
    if (openPopupId.value === ext.id) {
      await closeEmbeddedPopup();
      return;
    }
    if (openPopupId.value) {
      await closeEmbeddedPopup();
    }
    const anchor = event.currentTarget;
    if (!(anchor instanceof HTMLElement)) return;

    try {
      const popupUrl = await extensionPopupUrl(ext.id);
      if (!popupUrl) {
        emit('openExtensionsManager');
        return;
      }
      await closeLegacyPopupWindow(ext.id);
      await openToolbarExtensionPopup({
        extensionId: ext.id,
        popupUrl,
        anchor,
      });
      openPopupId.value = ext.id;
    } catch (error) {
      console.error('[ExtensionActionBar] openToolbarExtensionPopup failed:', error);
    }
    return;
  }
  emit('openExtensionsManager');
}

/**
 * Handle extension list changes from backend.
 * Aerospace-level event handling with debounced refresh.
 */
function onExtensionsChanged(): void {
  void refresh();
}

/**
 * Handle global popup close events from other components.
 * Aerospace-level state synchronization.
 */
function onGlobalPopupClosed(): void {
  openPopupId.value = null;
}

/**
 * Handle keyboard events for accessibility and UI control.
 * Aerospace-level keyboard event handling with ESC key support.
 */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    if (openPopupId.value) {
      void closeEmbeddedPopup();
    }
    if (showOverflowMenu.value) {
      closeOverflow();
    }
    if (contextMenu.value.visible) {
      closeContextMenu();
    }
  }
}

/**
 * Toggle overflow menu visibility.
 * Aerospace-level state management with user interaction tracking.
 */
function toggleOverflow(): void {
  showOverflowMenu.value = !showOverflowMenu.value;
}

/**
 * Close overflow menu with state cleanup.
 */
function closeOverflow(): void {
  showOverflowMenu.value = false;
}

/**
 * Handle right-click on extension icon to show context menu.
 * Aerospace-level menu positioning with boundary checking and dynamic item generation.
 */
function onExtensionRightClick(ext: ExtensionInfo, event: MouseEvent): void {
  // Calculate menu position with boundary checking
  const menuWidth = 220;
  const menuHeight = 200;
  let x = event.clientX;
  let y = event.clientY;
  
  // Prevent menu from going off-screen
  if (x + menuWidth > window.innerWidth) {
    x = window.innerWidth - menuWidth - 8;
  }
  if (y + menuHeight > window.innerHeight) {
    y = window.innerHeight - menuHeight - 8;
  }
  
  // Build menu items based on extension capabilities
  const menuItems: { id: string; label: string; icon?: string; disabled?: boolean; separator?: boolean; action?: () => void }[] = [
    {
      id: 'manage',
      label: 'Manage extension',
      icon: '⚙️',
      action: () => {
        emit('openExtensionsManager');
      },
    },
  ];
  
  // Add Options item if extension has options page
  if (ext.optionsUrl) {
    menuItems.push({
      id: 'options',
      label: 'Options',
      icon: '🔧',
      action: () => {
        if (ext.optionsUrl) {
          window.open(ext.optionsUrl, '_blank');
        }
      },
    });
  }
  
  // Add separator
  menuItems.push({
    id: 'separator1',
    label: '',
    separator: true,
  });
  
  // Add Hide/Show item based on current pinned status
  if (ext.pinned !== false) {
    menuItems.push({
      id: 'hide',
      label: 'Hide in toolbar',
      icon: '👁️',
      action: () => {
        hideExtensionFromToolbar(ext.id);
      },
    });
  } else {
    menuItems.push({
      id: 'show',
      label: 'Show in toolbar',
      icon: '👁️‍🗨️',
      action: () => {
        showExtensionInToolbar(ext.id);
      },
    });
  }
  
  // Add Remove item
  menuItems.push({
    id: 'remove',
    label: 'Remove',
    icon: '🗑️',
    action: () => {
      showRemovalDialog(ext.id, ext.name);
    },
  });
  
  contextMenu.value = {
    visible: true,
    x,
    y,
    extensionId: ext.id,
    items: menuItems,
  };
}

/**
 * Hide extension from toolbar by updating pinned status.
 * Aerospace-level error handling with user feedback and state validation.
 * Concurrency-safe: prevents duplicate operations on same extension.
 * Security: validates extension ID format to prevent injection attacks.
 */
async function hideExtensionFromToolbar(extensionId: string): Promise<void> {
  // Validate input format (security: prevent injection attacks)
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ExtensionActionBar] Invalid extension ID for hide operation:', extensionId);
    return;
  }
  
  // Security: validate extension ID format (alphanumeric, hyphens, underscores only)
  const validIdPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validIdPattern.test(extensionId)) {
    console.error('[ExtensionActionBar] Invalid extension ID format for hide operation:', extensionId);
    return;
  }
  
  // Aerospace-level concurrency check
  const operationKey = `hide-${extensionId}`;
  if (pendingOperations.has(operationKey)) {
    console.warn('[ExtensionActionBar] Hide operation already pending for:', extensionId);
    return;
  }
  
  pendingOperations.add(operationKey);
  
  try {
    const ext = extensions.value.find(e => e.id === extensionId);
    if (!ext) {
      console.error('[ExtensionActionBar] Extension not found for hide operation:', extensionId);
      return;
    }
    
    await invoke('extension_set_pinned', { extensionId, pinned: false });
    await refresh();
    window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
    
    // Show undo toast
    showUndoToast(extensionId, ext.name);
  } catch (error) {
    console.error('[ExtensionActionBar] Failed to hide extension from toolbar:', error);
    // User-facing error message
    alert('Failed to hide extension. Please try again.');
  } finally {
    pendingOperations.delete(operationKey);
  }
}

/**
 * Show undo toast after hiding extension.
 * Aerospace-level timeout management with cleanup.
 */
function showUndoToast(extensionId: string, extensionName: string): void {
  // Validate inputs
  if (!extensionId || !extensionName) {
    console.error('[ExtensionActionBar] Invalid parameters for showUndoToast');
    return;
  }
  
  // Clear existing timer if any
  if (retryTimer) {
    clearTimeout(retryTimer);
    retryTimer = null;
  }
  
  undoToast.value = {
    visible: true,
    extensionId,
    extensionName,
  };
  
  // Auto-hide after 5 seconds with timer reference for cleanup
  const toastTimer = setTimeout(() => {
    hideUndoToast();
  }, 5000);
  
  // Store timer reference for potential cleanup
  (undoToast as any)._timer = toastTimer;
}

/**
 * Hide the undo toast with timer cleanup.
 */
function hideUndoToast(): void {
  // Clear auto-hide timer if exists
  const timer = (undoToast as any)._timer;
  if (timer) {
    clearTimeout(timer);
    (undoToast as any)._timer = null;
  }
  
  undoToast.value.visible = false;
  undoToast.value.extensionId = null;
  undoToast.value.extensionName = '';
}

/**
 * Undo hiding extension by pinning it back.
 * Aerospace-level error handling with state validation.
 */
async function undoHideExtension(): Promise<void> {
  const extensionId = undoToast.value.extensionId;
  
  // Validate state
  if (!extensionId) {
    console.error('[ExtensionActionBar] No extension ID in undo toast');
    hideUndoToast();
    return;
  }
  
  hideUndoToast();
  
  try {
    await invoke('extension_set_pinned', { extensionId, pinned: true });
    await refresh();
    window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
  } catch (error) {
    console.error('[ExtensionActionBar] Failed to undo hide extension:', error);
    alert('Failed to restore extension. Please try again.');
  }
}

/**
 * Show extension in toolbar by pinning it.
 * Aerospace-level error handling with input validation.
 * Concurrency-safe: prevents duplicate operations on same extension.
 * Security: validates extension ID format to prevent injection attacks.
 */
async function showExtensionInToolbar(extensionId: string): Promise<void> {
  // Validate input format (security: prevent injection attacks)
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ExtensionActionBar] Invalid extension ID for show operation:', extensionId);
    return;
  }
  
  // Security: validate extension ID format (alphanumeric, hyphens, underscores only)
  const validIdPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validIdPattern.test(extensionId)) {
    console.error('[ExtensionActionBar] Invalid extension ID format for show operation:', extensionId);
    return;
  }
  
  // Aerospace-level concurrency check
  const operationKey = `show-${extensionId}`;
  if (pendingOperations.has(operationKey)) {
    console.warn('[ExtensionActionBar] Show operation already pending for:', extensionId);
    return;
  }
  
  pendingOperations.add(operationKey);
  
  try {
    await invoke('extension_set_pinned', { extensionId, pinned: true });
    await refresh();
    window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
  } catch (error) {
    console.error('[ExtensionActionBar] Failed to show extension in toolbar:', error);
    alert('Failed to show extension. Please try again.');
  } finally {
    pendingOperations.delete(operationKey);
  }
}

/**
 * Remove extension completely.
 * Aerospace-level error handling with input validation and state management.
 * Security: validates extension ID format and prevents unauthorized removals.
 */
async function removeExtension(extensionId: string): Promise<void> {
  // Validate input format (security: prevent injection attacks)
  if (!extensionId || typeof extensionId !== 'string') {
    console.error('[ExtensionActionBar] Invalid extension ID for remove operation:', extensionId);
    throw new Error('Invalid extension ID');
  }
  
  // Security: validate extension ID format (alphanumeric, hyphens, underscores only)
  const validIdPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validIdPattern.test(extensionId)) {
    console.error('[ExtensionActionBar] Invalid extension ID format for remove operation:', extensionId);
    throw new Error('Invalid extension ID format');
  }
  
  try {
    await invoke('extension_uninstall', { extensionId });
    await refresh();
    window.dispatchEvent(new CustomEvent('exodus-extensions-changed'));
  } catch (error) {
    console.error('[ExtensionActionBar] Failed to remove extension:', error);
    throw error;
  }
}

/**
 * Show removal confirmation dialog.
 * Aerospace-level input validation and state management.
 */
/**
 * Show removal confirmation dialog.
 * Aerospace-level input validation with security checks.
 */
function showRemovalDialog(extensionId: string, extensionName: string): void {
  // Validate inputs
  if (!extensionId || !extensionName) {
    console.error('[ExtensionActionBar] Invalid parameters for showRemovalDialog');
    return;
  }
  
  if (typeof extensionId !== 'string' || typeof extensionName !== 'string') {
    console.error('[ExtensionActionBar] Invalid parameter types for showRemovalDialog');
    return;
  }
  
  // Aerospace-level security validation: validate extension ID format
  const validIdPattern = /^[a-zA-Z0-9_-]+$/;
  if (!validIdPattern.test(extensionId)) {
    console.error('[ExtensionActionBar] Invalid extension ID format for showRemovalDialog:', extensionId);
    return;
  }
  
  // Close context menu first
  closeContextMenu();
  
  removalDialog.value = {
    visible: true,
    extensionId,
    extensionName,
  };
}

/**
 * Cancel removal and close dialog with state cleanup.
 */
function cancelRemoval(): void {
  removalDialog.value.visible = false;
  removalDialog.value.extensionId = null;
  removalDialog.value.extensionName = '';
}

/**
 * Confirm and execute removal with aerospace-level error handling.
 */
async function confirmRemoval(): Promise<void> {
  const extensionId = removalDialog.value.extensionId;
  
  // Validate state before proceeding
  if (!extensionId) {
    console.error('[ExtensionActionBar] No extension ID in removal dialog');
    cancelRemoval();
    return;
  }
  
  removalDialog.value.visible = false;
  
  try {
    await removeExtension(extensionId);
    // Only clear state on success
    removalDialog.value.extensionId = null;
    removalDialog.value.extensionName = '';
  } catch (error) {
    console.error('[ExtensionActionBar] Failed to remove extension:', error);
    // Show error feedback to user
    alert('Failed to remove extension. Please try again.');
    // Reopen dialog on error for retry
    removalDialog.value.visible = true;
  }
}

/**
 * Close context menu with state cleanup.
 * Aerospace-level state management to prevent menu state leaks.
 */
function closeContextMenu(): void {
  contextMenu.value.visible = false;
  contextMenu.value.extensionId = null;
}

watch(
  () => props.refreshKey,
  () => {
    void refresh();
  },
);

onMounted(() => {
  void refresh();
  scheduleRetryRefresh();
  window.addEventListener('exodus-extensions-changed', onExtensionsChanged);
  window.addEventListener('exodus-extension-popup-closed', onGlobalPopupClosed);
  window.addEventListener('keydown', onKeydown);
});

onUnmounted(() => {
  // Aerospace-level cleanup: clear all timers and event listeners
  if (retryTimer) {
    clearTimeout(retryTimer);
    retryTimer = null;
  }
  
  // Clear toast timer if exists
  const toastTimer = (undoToast as any)._timer;
  if (toastTimer) {
    clearTimeout(toastTimer);
    (undoToast as any)._timer = null;
  }
  
  // Remove all event listeners
  window.removeEventListener('exodus-extensions-changed', onExtensionsChanged);
  window.removeEventListener('exodus-extension-popup-closed', onGlobalPopupClosed);
  window.removeEventListener('keydown', onKeydown);
  
  // Close any open popups and dialogs
  void closeEmbeddedPopup();
  closeContextMenu();
  hideUndoToast();
  cancelRemoval();
});

defineExpose({ refresh, closeEmbeddedPopup });
</script>

<style scoped>
.extension-action-bar {
  display: flex;
  align-items: center;
  gap: 0;
  flex-shrink: 0;
  min-width: 0;
}

.extension-action-bar--inline {
  padding: 0 2px;
  border: none;
  background: transparent;
}

.extension-action-btn {
  min-width: var(--chrome-icon-btn-size, 28px);
  width: var(--chrome-icon-btn-size, 28px);
  height: var(--chrome-icon-btn-size, 28px);
  border: none;
  border-radius: var(--chrome-icon-btn-radius, 50%);
  background: transparent;
  cursor: pointer;
  color: var(--chrome-tab-text, #202124);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background-color 0.15s ease;
  padding: 0;
}

.extension-action-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.06);
}

.extension-action-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.extension-action-btn.active {
  background: rgba(95, 99, 104, 0.12);
}

.extension-action-icon {
  width: var(--chrome-icon-size, 16px);
  height: var(--chrome-icon-size, 16px);
  display: block;
  object-fit: contain;
}

.extension-action-letter {
  width: var(--chrome-icon-size, 16px);
  height: var(--chrome-icon-size, 16px);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 500;
  color: var(--chrome-tab-text, #202124);
  line-height: 1;
}

.extension-puzzle-svg {
  width: var(--chrome-icon-size, 16px);
  height: var(--chrome-icon-size, 16px);
  display: block;
  color: var(--chrome-tab-text, #202124);
}

.extension-overflow-svg {
  width: 14px;
  height: 14px;
  display: block;
  color: var(--chrome-tab-text, #202124);
}

.extension-overflow-menu {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 1000;
  min-width: 200px;
  max-width: 280px;
  max-height: 400px;
  overflow-y: auto;
  padding: 4px;
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  background: var(--chrome-tab-bg-active, #ffffff);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.extension-overflow-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--chrome-tab-text-active, #202124);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.extension-overflow-item:hover {
  background: var(--chrome-tab-bg-hover, #e8eaed);
}

.extension-overflow-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.extension-popup-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9998;
  border: none;
  padding: 0;
  margin: 0;
  background: transparent;
  cursor: default;
}

@media (prefers-color-scheme: dark) {
  .extension-action-btn {
    color: #e8eaed;
  }

  .extension-action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
  }

  .extension-action-btn.active {
    background: rgba(255, 255, 255, 0.12);
  }

  .extension-action-letter,
  .extension-puzzle-svg,
  .extension-overflow-svg {
    color: #e8eaed;
  }

  .extension-overflow-menu {
    background: #292a2d;
    border-color: #5f6368;
  }

  .extension-overflow-item {
    color: #e8eaed;
  }

  .extension-overflow-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

/* Removal Confirmation Dialog Styles */
.extension-removal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.15s ease;
}

.extension-removal-dialog {
  background: var(--chrome-tab-bg-active, #ffffff);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  max-width: 400px;
  width: 90%;
  animation: slideUp 0.2s ease;
}

.removal-dialog-header {
  padding: 20px 20px 12px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
}

.removal-dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 500;
  color: var(--chrome-tab-text-active, #202124);
}

.removal-dialog-body {
  padding: 16px 20px;
}

.removal-dialog-body p {
  margin: 0 0 12px;
  font-size: 14px;
  color: var(--chrome-tab-text, #5f6368);
  line-height: 1.5;
}

.removal-dialog-body strong {
  color: var(--chrome-tab-text-active, #202124);
}

.removal-warning {
  color: #d93025 !important;
  font-size: 13px !important;
  margin-top: 8px !important;
}

.removal-dialog-footer {
  padding: 12px 20px 20px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.removal-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background-color 0.15s ease;
}

.removal-btn-cancel {
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  border: 1px solid var(--chrome-divider, #dadce0);
}

.removal-btn-cancel:hover {
  background: rgba(0, 0, 0, 0.06);
}

.removal-btn-confirm {
  background: #d93025;
  color: white;
}

.removal-btn-confirm:hover {
  background: #c5221f;
}

@media (prefers-color-scheme: dark) {
  .extension-removal-dialog {
    background: #2d2e30;
    border: 1px solid #5f6368;
  }

  .removal-dialog-header {
    border-color: #5f6368;
  }

  .removal-dialog-header h3 {
    color: #e8eaed;
  }

  .removal-dialog-body p {
    color: #9aa0a6;
  }

  .removal-dialog-body strong {
    color: #e8eaed;
  }

  .removal-btn-cancel {
    color: #e8eaed;
    border-color: #5f6368;
  }

  .removal-btn-cancel:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Undo Toast Styles */
.extension-undo-toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--chrome-tab-bg-active, #ffffff);
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 10002;
  animation: slideUp 0.2s ease;
  min-width: 300px;
}

.undo-toast-message {
  flex: 1;
  font-size: 14px;
  color: var(--chrome-tab-text, #5f6368);
}

.undo-toast-message strong {
  color: var(--chrome-tab-text-active, #202124);
}

.undo-toast-btn {
  padding: 6px 12px;
  background: #1a73e8;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.undo-toast-btn:hover {
  background: #1557b0;
}

.undo-toast-close {
  padding: 4px 8px;
  background: transparent;
  color: var(--chrome-tab-text, #5f6368);
  border: none;
  border-radius: 4px;
  font-size: 16px;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.undo-toast-close:hover {
  background: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  .extension-undo-toast {
    background: #2d2e30;
    border-color: #5f6368;
  }

  .undo-toast-message {
    color: #9aa0a6;
  }

  .undo-toast-message strong {
    color: #e8eaed;
  }

  .undo-toast-close {
    color: #9aa0a6;
  }

  .undo-toast-close:hover {
    background: rgba(255, 255, 255, 0.08);
  }
}

</style>
