<template>
  <div
    class="ntp exodus-new-tab"
    :style="ntpStyle"
    :data-ntp-wallpaper-id="currentWallpaperId"
    aria-label="New tab"
  >
    <img
      v-if="wallpaperUrl"
      class="ntp-bg-image"
      :src="wallpaperUrl"
      alt=""
      @error="onWallpaperImgError"
    />
    <div class="ntp-vignette" aria-hidden="true"></div>

    <!-- Settings button -->
    <button
      class="ntp-settings-btn"
      @click="handleSettingsClick"
      title="Customize new tab page"
      aria-label="Customize new tab page"
    >
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M10 3.5a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3zM10 8.5a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3zM10 13.5a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3z"/>
      </svg>
    </button>

    <main class="ntp-body">
      <div class="ntp-logo">⛵ Exodus</div>

      <div class="ntp-search">
        <div class="ntp-search-wrapper">
          <svg class="ntp-search-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#5f6368" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="M21 21l-4.35-4.35"/>
          </svg>
          <input
            ref="searchInput"
            type="text"
            placeholder="在 Exodus 中搜索或输入网址"
            class="ntp-search-input"
            @keydown="handleSearchKeydown"
          />
          <div class="ntp-search-actions">
            <svg class="ntp-search-action-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#5f6368" stroke-width="2">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
            <svg class="ntp-search-action-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#5f6368" stroke-width="2">
              <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/>
              <circle cx="12" cy="13" r="4"/>
            </svg>
            <button class="ntp-search-ai-btn">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#5f6368" stroke-width="2">
                <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                <path d="M2 17l10 5 10-5"/>
                <path d="M2 12l10 5 10-5"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <section class="ntp-section" aria-label="Top sites">
        <div v-if="displayTopSites.length === 0" class="ntp-empty-hint">
          Right-click a site to add, or use <strong>+</strong> below.
        </div>
        <div class="ntp-top-grid">
          <button
            v-for="site in displayTopSites"
            :key="site.url"
            type="button"
            class="ntp-tile"
            :data-ntp-site-url="site.url"
            :title="site.url"
            @click="() => onNavigate(site.url)"
            @contextmenu.prevent.stop="handleTileContextMenu(site, $event)"
          >
            <span v-if="getFaviconUrl(site)" class="ntp-tile-icon ntp-tile-icon--img">
              <img :src="getFaviconUrl(site)" :alt="hostLabel(site)" @error="(e) => (e.target as HTMLImageElement).style.display = 'none'" />
            </span>
            <span v-else class="ntp-tile-icon">{{ tileLabel(site) }}</span>
            <span class="ntp-tile-label">{{ hostLabel(site) }}</span>
          </button>
          <button
            type="button"
            class="ntp-tile ntp-tile-add"
            title="Add top site"
            aria-label="Add top site"
            @click="handleAddTopSiteClick"
          >
            <span class="ntp-tile-icon">+</span>
            <span class="ntp-tile-label">Add</span>
          </button>
        </div>
      </section>

      <section class="ntp-section" aria-label="Quick links">
        <div class="ntp-chips">
          <button
            v-for="link in displayQuickLinks"
            :key="link.url"
            type="button"
            class="ntp-chip"
            :data-ntp-chip-url="link.url"
            :title="link.url"
            @click="() => onNavigate(link.url)"
            @contextmenu.prevent.stop="handleChipContextMenu(link, $event)"
          >
            {{ link.title }}
          </button>
          <button
            type="button"
            class="ntp-chip ntp-chip-add"
            title="Add quick link"
            aria-label="Add quick link"
            @click="handleAddQuickLinkClick"
          >
            +
          </button>
        </div>
      </section>
    </main>

    <!-- Context menu for tiles -->
    <div
      v-if="contextMenuVisible"
      class="ntp-context-menu"
      :style="{ left: contextMenuX + 'px', top: contextMenuY + 'px' }"
      @click="closeContextMenu"
    >
      <button
        class="ntp-context-item"
        :disabled="isInQuickLinks(selectedSite)"
        @click="handleAddToQuickLinks"
      >
        Add to quick links
      </button>
      <button
        class="ntp-context-item"
        :disabled="isSitePinned(selectedSite)"
        @click="handlePinSite"
      >
        Pin to front
      </button>
      <button
        class="ntp-context-item"
        :disabled="!isSitePinned(selectedSite)"
        @click="handleUnpinSite"
      >
        Unpin
      </button>
      <button class="ntp-context-item" @click="handleRemoveSite">Remove from top sites</button>
    </div>

    <!-- Context menu for quick-link chips -->
    <div
      v-if="chipContextMenuVisible"
      class="ntp-context-menu"
      :style="{ left: chipContextMenuX + 'px', top: chipContextMenuY + 'px' }"
      @click="closeChipContextMenu"
    >
      <button class="ntp-context-item" @click="handleRemoveChip">Remove quick link</button>
    </div>

    <!-- Add URL dialog (replaces window.prompt — blocked in Tauri WKWebView) -->
    <template v-if="urlAddDialogMode">
      <button
        type="button"
        class="ntp-url-dialog-backdrop"
        aria-label="Cancel"
        @click="closeUrlAddDialog"
      />
      <div
        class="ntp-url-dialog"
        role="dialog"
        aria-labelledby="ntp-url-dialog-title"
        @click.stop
      >
        <h3 id="ntp-url-dialog-title">{{ urlAddDialogTitle }}</h3>
        <label class="ntp-url-dialog-field">
          <span>URL</span>
          <input
            ref="urlAddInputEl"
            v-model="urlAddInput"
            type="text"
            class="ntp-url-dialog-input"
            placeholder="https://example.com"
            @keydown.enter.prevent="submitUrlAddDialog"
          />
        </label>
        <p v-if="urlAddError" class="ntp-url-dialog-error">{{ urlAddError }}</p>
        <div class="ntp-url-dialog-actions">
          <button type="button" class="ntp-url-dialog-btn secondary" @click="closeUrlAddDialog">
            Cancel
          </button>
          <button
            type="button"
            class="ntp-url-dialog-btn primary"
            :disabled="!urlAddInput.trim()"
            @click="submitUrlAddDialog"
          >
            Add
          </button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Brave-style new tab overlay (wallpaper, top sites, quick links).
 * Uses Tauri blob URLs for raster backgrounds in the desktop shell.
 */
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import type { QuickLink } from '@/lib/browserTypes';
import {
  DEFAULT_QUICK_LINKS,
  isValidNtpSiteUrl,
  ntpHostLabel,
  ntpTileLabel,
  quickLinkFromUserUrl,
} from '@/lib/newTabPage';
import { normalizeNtpSiteUrl } from '@/lib/ntpTopSitesStore';
import { faviconUrlFor } from '@/lib/favicon';
import { logStartup } from '@/lib/startupLog';
import { ntpLog } from '@/lib/diagnosticLog';
import {
  WALLPAPER_FEATURE_ENABLED,
  loadWallpaperId,
  getWindowSessionWallpaperId,
  getWallpaperById,
  wallpaperAssetUrl,
  wallpaperAbsoluteAssetUrl,
  peekCachedWallpaperDisplayUrl,
  resolveWallpaperDisplayUrl,
} from '@/lib/newTabWallpaper';

interface Props {
  visible?: boolean;
  topSites?: QuickLink[];
  links?: QuickLink[];
  wallpaperId?: string;
  wallpaperDisplayUrl?: string;
  pinnedTopSiteUrls?: string[];
  aiOnline?: boolean;
  aiModel?: string;
  onNavigate: (url: string) => void;
  onOpenSettings?: () => void;
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  topSites: () => [],
  links: () => DEFAULT_QUICK_LINKS,
  wallpaperId: '',
  wallpaperDisplayUrl: '',
  pinnedTopSiteUrls: () => [],
  aiOnline: false,
  aiModel: 'gemma4-e2b',
});

const emit = defineEmits<{
  pinSite: [site: QuickLink];
  unpinSite: [site: QuickLink];
  removeSite: [site: QuickLink];
  addQuickLink: [link: QuickLink];
  removeQuickLink: [link: QuickLink];
  addTopSite: [site: QuickLink];
}>();

const searchInput = ref<HTMLInputElement>();
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const selectedSite = ref<QuickLink | null>(null);
const chipContextMenuVisible = ref(false);
const chipContextMenuX = ref(0);
const chipContextMenuY = ref(0);
const selectedChip = ref<QuickLink | null>(null);
const urlAddDialogMode = ref<'top-site' | 'quick-link' | null>(null);
const urlAddInput = ref('');
const urlAddError = ref('');
const urlAddInputEl = ref<HTMLInputElement>();
const currentWallpaperId = ref<string>('');
const wallpaperUrl = ref<string>('');
const wallpaperLoadAttempt = ref(0);

/** Top sites from shell (user list — no auto-refill). */
const displayTopSites = computed(() =>
  (props.topSites ?? []).filter((site) => isValidNtpSiteUrl(site.url)),
);

/** Quick-link chips from shell (user list — no auto-refill). */
const displayQuickLinks = computed(() =>
  (props.links ?? []).filter((link) => isValidNtpSiteUrl(link.url)),
);

const pinnedUrlSet = computed(() => new Set(props.pinnedTopSiteUrls ?? []));

const quickLinkUrlSet = computed(
  () => new Set(displayQuickLinks.value.map((link) => normalizeNtpSiteUrl(link.url))),
);

const urlAddDialogTitle = computed(() =>
  urlAddDialogMode.value === 'quick-link' ? 'Add quick link' : 'Add top site',
);

function isSitePinned(site: QuickLink | null): boolean {
  if (!site) return false;
  return pinnedUrlSet.value.has(normalizeNtpSiteUrl(site.url));
}

function isInQuickLinks(site: QuickLink | null): boolean {
  if (!site) return false;
  return quickLinkUrlSet.value.has(normalizeNtpSiteUrl(site.url));
}

/** Dynamic accent based on wallpaper. */
const ntpStyle = computed(() => {
  if (!WALLPAPER_FEATURE_ENABLED || !currentWallpaperId.value) {
    return { '--ntp-accent': '#6366f1' };
  }
  const wallpaper = getWallpaperById(currentWallpaperId.value);
  return { '--ntp-accent': wallpaper?.accent || '#6366f1' };
});

/** Resolve wallpaper id: explicit prop → per-window session → global preference. */
function resolveWallpaperId(): string {
  if (props.wallpaperId) return props.wallpaperId;
  return getWindowSessionWallpaperId() ?? loadWallpaperId();
}

/**
 * Load wallpaper: instant paint URL, then async upgrade (blob/asset) even when parent preloads.
 */
async function loadWallpaper(forceBlob = false): Promise<void> {
  if (!WALLPAPER_FEATURE_ENABLED) return;
  const id = resolveWallpaperId();
  currentWallpaperId.value = id;
  const entry = getWallpaperById(id);

  ntpLog.timeStart('NewTabPage.loadWallpaper');
  try {
    if (!forceBlob) {
      wallpaperUrl.value =
        props.wallpaperDisplayUrl ||
        peekCachedWallpaperDisplayUrl(id) ||
        wallpaperAbsoluteAssetUrl(entry.file) ||
        wallpaperAssetUrl(entry.file);
      ntpLog.info('wallpaper fast paint', {
        id,
        fromParent: !!props.wallpaperDisplayUrl,
        url: wallpaperUrl.value,
      });
    }

    const url = await resolveWallpaperDisplayUrl(id, forceBlob ? { forceBlob: true } : undefined);
    if (url) wallpaperUrl.value = url;
    ntpLog.info('wallpaper ready', { id, hasUrl: !!url });
  } catch (error) {
    ntpLog.error('wallpaper load failed', error);
    if (!wallpaperUrl.value) {
      wallpaperUrl.value = wallpaperAssetUrl(entry.file);
    }
  } finally {
    ntpLog.timeEnd('NewTabPage.loadWallpaper', { id });
  }
}

/** Retry with backend blob when img/CSS URL fails in Tauri dev shell. */
async function onWallpaperImgError(): Promise<void> {
  if (wallpaperLoadAttempt.value >= 1) return;
  wallpaperLoadAttempt.value += 1;
  try {
    await loadWallpaper(true);
  } catch (error) {
    console.error('[NewTabPage] Wallpaper retry failed:', error);
  }
}

watch(
  () => props.wallpaperId,
  (id) => {
    if (!id || id === currentWallpaperId.value) return;
    wallpaperLoadAttempt.value = 0;
    void loadWallpaper();
  },
);

watch(
  () => props.wallpaperDisplayUrl,
  (url) => {
    if (!url || url === wallpaperUrl.value) return;
    wallpaperUrl.value = url;
  },
);

function onDocumentClick(e: MouseEvent): void {
  const target = e.target as HTMLElement;
  if (contextMenuVisible.value && !target.closest('.ntp-context-menu')) {
    closeContextMenu();
  }
  if (chipContextMenuVisible.value && !target.closest('.ntp-context-menu')) {
    closeChipContextMenu();
  }
}

onMounted(() => {
  logStartup('NewTabPage mounted');
  document.addEventListener('click', onDocumentClick);
  void loadWallpaper();
});

onUnmounted(() => {
  document.removeEventListener('click', onDocumentClick);
});

function getFaviconUrl(link: QuickLink): string | undefined {
  return faviconUrlFor(link.url) || undefined;
}

function tileLabel(link: QuickLink): string {
  return ntpTileLabel(link);
}

function hostLabel(link: QuickLink): string {
  return ntpHostLabel(link);
}

function handleSearchKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && searchInput.value?.value) {
    const value = searchInput.value.value;
    props.onNavigate(
      value.startsWith('http') ? value : `https://www.google.com/search?q=${encodeURIComponent(value)}`
    );
  }
}

function handleSettingsClick() {
  props.onOpenSettings?.();
}

function handleTileContextMenu(site: QuickLink, event: MouseEvent) {
  closeChipContextMenu();
  selectedSite.value = site;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuVisible.value = true;
}

function handleChipContextMenu(link: QuickLink, event: MouseEvent) {
  closeContextMenu();
  selectedChip.value = link;
  chipContextMenuX.value = event.clientX;
  chipContextMenuY.value = event.clientY;
  chipContextMenuVisible.value = true;
}

function closeContextMenu() {
  contextMenuVisible.value = false;
  selectedSite.value = null;
}

function closeChipContextMenu() {
  chipContextMenuVisible.value = false;
  selectedChip.value = null;
}

function handlePinSite() {
  if (selectedSite.value && !isSitePinned(selectedSite.value)) {
    emit('pinSite', selectedSite.value);
  }
  closeContextMenu();
}

function handleUnpinSite() {
  if (selectedSite.value && isSitePinned(selectedSite.value)) {
    emit('unpinSite', selectedSite.value);
  }
  closeContextMenu();
}

function handleRemoveSite() {
  if (selectedSite.value) {
    emit('removeSite', selectedSite.value);
  }
  closeContextMenu();
}

function handleAddToQuickLinks() {
  if (selectedSite.value && !isInQuickLinks(selectedSite.value)) {
    emit('addQuickLink', selectedSite.value);
  }
  closeContextMenu();
}

function handleRemoveChip() {
  if (selectedChip.value) {
    emit('removeQuickLink', selectedChip.value);
  }
  closeChipContextMenu();
}

function handleAddQuickLinkClick() {
  openUrlAddDialog('quick-link');
}

function handleAddTopSiteClick() {
  openUrlAddDialog('top-site');
}

/** Open the in-page URL dialog (Tauri-safe alternative to window.prompt). */
function openUrlAddDialog(mode: 'top-site' | 'quick-link'): void {
  closeContextMenu();
  closeChipContextMenu();
  urlAddDialogMode.value = mode;
  urlAddInput.value = '';
  urlAddError.value = '';
  void nextTick(() => urlAddInputEl.value?.focus());
}

/** Close the add-URL dialog without submitting. */
function closeUrlAddDialog(): void {
  urlAddDialogMode.value = null;
  urlAddInput.value = '';
  urlAddError.value = '';
}

/** Validate URL input and emit addTopSite or addQuickLink. */
function submitUrlAddDialog(): void {
  const link = quickLinkFromUserUrl(urlAddInput.value);
  if (!link) {
    urlAddError.value = 'Enter a valid https:// URL';
    return;
  }
  if (urlAddDialogMode.value === 'quick-link') {
    emit('addQuickLink', link);
  } else {
    emit('addTopSite', link);
  }
  closeUrlAddDialog();
}
</script>

<style scoped>
.ntp {
  position: absolute;
  inset: 0;
  z-index: 3;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--chrome-ntp-padding-y, 24px) var(--chrome-ntp-padding-x, 20px) 16px;
  background-color: #0a0a0f;
  color: #f4f4f5;
  min-height: 100%;
  min-width: 100%;
}

.ntp--gradient {
  background: linear-gradient(165deg, #1a1f3a 0%, #0f111a 45%, #0a0a0f 100%);
}

.ntp-bg-image {
  position: absolute;
  inset: 0;
  z-index: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  object-position: center;
  pointer-events: none;
}

.ntp-vignette {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  background: linear-gradient(
    180deg,
    rgba(6, 8, 14, 0.35) 0%,
    rgba(8, 10, 18, 0.15) 40%,
    rgba(6, 6, 10, 0.55) 100%
  );
}

.ntp-settings-btn {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 10;
  padding: 8px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.2s ease;
}

.ntp-settings-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.ntp-wallpaper-btn {
  position: absolute;
  top: 16px;
  right: 56px;
  z-index: 10;
  padding: 8px;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 4px;
}

.ntp-wallpaper-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
  transform: scale(1.05);
}

.ntp-wallpaper-hint {
  font-size: 14px;
  opacity: 0.8;
}

.ntp-body {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 584px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--chrome-ntp-gap, 20px);
}

.ntp-logo {
  font-size: 1.5rem;
  font-weight: 700;
  margin-bottom: 4px;
  opacity: 0.92;
  letter-spacing: -0.02em;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.8);
}

.ntp-search {
  width: 100%;
  max-width: 584px;
}

.ntp-search-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
  max-width: 584px;
  height: 46px;
  padding: 0 14px 0 44px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.3);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.2);
  transition: all 0.2s ease;
}

.ntp-search-wrapper:hover {
  background: rgba(255, 255, 255, 0.9);
  border-color: rgba(255, 255, 255, 0.5);
  box-shadow: 0 1px 6px rgba(0, 0, 0, 0.3);
}

.ntp-search-icon {
  position: absolute;
  left: 16px;
  color: #000;
  pointer-events: none;
  z-index: 20;
  -webkit-font-smoothing: antialiased;
}

.ntp-search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: #000;
  font-size: 16px;
  font-weight: 500;
  outline: none;
  z-index: 1;
  position: relative;
}

.ntp-search-input::placeholder {
  color: rgba(0, 0, 0, 0.7);
}

.ntp-search-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 8px;
  z-index: 20;
  position: relative;
}

.ntp-search-action-icon {
  flex-shrink: 0;
  color: #000;
  cursor: pointer;
  transition: color 0.2s ease;
  -webkit-font-smoothing: antialiased;
}

.ntp-search-action-icon:hover {
  color: #000;
}

.ntp-search-ai-btn {
  flex-shrink: 0;
  padding: 6px;
  border: none;
  border-radius: 50%;
  background: transparent;
  cursor: pointer;
  transition: background 0.2s ease;
}

.ntp-search-ai-btn:hover {
  background: rgba(0, 0, 0, 0.1);
}

.ntp-search-ai-btn svg {
  color: #000;
  -webkit-font-smoothing: antialiased;
}

.ntp-section {
  width: 100%;
}

.ntp-top-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  justify-items: center;
}

.ntp-empty-hint {
  width: 100%;
  margin-bottom: 8px;
  text-align: center;
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.55);
}

.ntp-tile-add {
  border-style: dashed;
  opacity: 0.85;
}

.ntp-tile-add:hover {
  opacity: 1;
}

.ntp-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 10px 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  color: #f4f4f5;
  cursor: pointer;
  transition: all 0.2s ease;
}

.ntp-tile:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-2px);
  border-color: var(--ntp-accent);
}

.ntp-tile-icon {
  width: var(--chrome-ntp-tile-icon, 32px);
  height: var(--chrome-ntp-tile-icon, 32px);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 0.875rem;
  background: linear-gradient(135deg, var(--ntp-accent), rgba(255, 255, 255, 0.2));
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
}

.ntp-tile-icon--img {
  background: transparent;
  box-shadow: none;
  padding: 0;
  overflow: hidden;
}

.ntp-tile-icon--img img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 50%;
}

.ntp-tile-label {
  font-size: 0.75rem;
  font-weight: 600;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  opacity: 0.85;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
}

.ntp-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
}

.ntp-chip {
  padding: 6px 12px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: #f4f4f5;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
}

.ntp-chip:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: var(--ntp-accent);
}

.ntp-chip-add {
  min-width: 36px;
  font-size: 1.1rem;
  font-weight: 600;
  line-height: 1;
  opacity: 0.85;
}

.ntp-chip-add:hover {
  opacity: 1;
}

.ntp-context-menu {
  position: fixed;
  z-index: 10001;
  padding: 8px 0;
  border-radius: 8px;
  background: rgba(30, 30, 35, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  min-width: 140px;
}

.ntp-context-item {
  width: 100%;
  padding: 8px 16px;
  border: none;
  background: transparent;
  color: #f4f4f5;
  font-size: 0.85rem;
  text-align: left;
  cursor: pointer;
  transition: background 0.15s ease;
}

.ntp-context-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.ntp-context-item:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.ntp-context-item:disabled:hover {
  background: transparent;
}

.ntp-footer {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.ntp-wallpaper-btn {
  padding: 8px 16px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  color: #e4e4e7;
  font-size: 0.82rem;
  cursor: pointer;
}

.ntp-wallpaper-btn:hover {
  border-color: var(--ntp-accent);
  color: #fff;
}

.ntp-picker {
  width: min(520px, 92vw);
  padding: 14px;
  border-radius: 14px;
  background: rgba(12, 12, 18, 0.88);
  border: 1px solid rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.45);
}

.ntp-picker-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  font-size: 0.85rem;
  color: #d4d4d8;
}

.ntp-picker-shuffle {
  border: none;
  background: transparent;
  color: var(--ntp-accent);
  cursor: pointer;
  font-size: 0.82rem;
}

.ntp-picker-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}

.ntp-picker-item {
  position: relative;
  aspect-ratio: 16 / 10;
  border-radius: 10px;
  border: 2px solid transparent;
  background-size: cover;
  background-position: center;
  cursor: pointer;
  overflow: hidden;
}

.ntp-picker-item.selected {
  border-color: var(--ntp-accent);
  box-shadow: 0 0 0 1px var(--ntp-accent);
}

.ntp-picker-name {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 6px 8px;
  font-size: 0.7rem;
  text-align: left;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.75));
}

.ntp-url-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10001;
  border: none;
  background: rgba(0, 0, 0, 0.55);
  cursor: default;
}

.ntp-url-dialog {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 10002;
  width: min(400px, 92vw);
  padding: 20px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(20, 20, 28, 0.96);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
  color: #f4f4f5;
}

.ntp-url-dialog h3 {
  margin: 0 0 16px;
  font-size: 1rem;
  font-weight: 600;
}

.ntp-url-dialog-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 0.85rem;
}

.ntp-url-dialog-input {
  width: 100%;
  box-sizing: border-box;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.08);
  color: #f4f4f5;
  font-size: 14px;
  outline: none;
}

.ntp-url-dialog-input:focus {
  border-color: var(--ntp-accent, #6366f1);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.35);
}

.ntp-url-dialog-error {
  margin: 0 0 12px;
  font-size: 0.8rem;
  color: #f87171;
}

.ntp-url-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.ntp-url-dialog-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  font-size: 0.85rem;
  cursor: pointer;
}

.ntp-url-dialog-btn.secondary {
  background: rgba(255, 255, 255, 0.1);
  color: #f4f4f5;
}

.ntp-url-dialog-btn.primary {
  background: var(--ntp-accent, #6366f1);
  color: #fff;
}

.ntp-url-dialog-btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
