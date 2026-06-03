<!--
  Exodus Browser — chrome://apps grid (Chrome-style, separate from Extensions settings).
-->
<template>
  <div class="chrome-apps-page">
    <header class="chrome-apps-header">
      <h2>{{ ui.pageTitle }}</h2>
      <button type="button" class="chrome-apps-manage" @click="emit('openExtensions')">
        {{ ui.manageExtensions }}
      </button>
    </header>

    <p v-if="loading" class="chrome-apps-hint">{{ ui.pageTitle }}…</p>
    <p v-else-if="tiles.length === 0" class="chrome-apps-hint">{{ ui.empty }}</p>

    <div v-else class="chrome-apps-grid" role="list">
      <button
        v-for="tile in tiles"
        :key="tile.id"
        type="button"
        class="chrome-apps-tile"
        role="listitem"
        :title="tile.name"
        @click="onTileClick(tile)"
      >
        <span class="chrome-apps-tile-icon" :class="{ 'chrome-apps-tile-icon--letter': !iconUrls[tile.id] }">
          <img
            v-if="iconUrls[tile.id]"
            class="chrome-apps-tile-img"
            :src="iconUrls[tile.id]"
            alt=""
            width="32"
            height="32"
          />
          <span v-else aria-hidden="true">{{ tile.iconLetter }}</span>
        </span>
        <span class="chrome-apps-tile-label">{{ tile.name }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — Chrome-style apps launcher grid for chrome://apps.
 */
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listExtensions } from '@/lib/extensions/api';
import type { ExtensionInfo } from '@/lib/extensions/types';
import {
  buildChromeAppTiles,
  chromeAppsPageStrings,
  type ChromeAppTile,
} from '@/lib/chromeAppsPage';
import type { BookmarkBarLocale } from '@/lib/bookmarkBarUi';
import { resolveExtensionIconUrl } from '@/lib/extensionToolbarIcon';
import { canUseNativeWebview } from '@/lib/exodusBrowser';

const props = defineProps<{
  uiLocale?: BookmarkBarLocale;
}>();

const emit = defineEmits<{
  navigate: [url: string];
  openExtensions: [];
  openBookmarksPanel: [];
  status: [message: string];
}>();

const loading = ref(true);
const extensions = ref<ExtensionInfo[]>([]);
const iconUrls = ref<Record<string, string>>({});

const ui = computed(() => chromeAppsPageStrings(props.uiLocale));
const tiles = computed(() => buildChromeAppTiles(extensions.value, props.uiLocale));

/** Resolve extension tile icons after loading the registry. */
async function resolveTileIcons(list: ExtensionInfo[]): Promise<void> {
  const next: Record<string, string> = {};
  await Promise.all(
    list
      .filter((ext) => ext.enabled)
      .map(async (ext) => {
        const url = await resolveExtensionIconUrl(ext);
        if (url) next[`ext-${ext.id}`] = url;
      }),
  );
  iconUrls.value = next;
}

/** Load installed extensions for the apps grid. */
async function loadApps(): Promise<void> {
  loading.value = true;
  try {
    extensions.value = await listExtensions();
    await resolveTileIcons(extensions.value);
  } catch (error) {
    console.error('chrome apps load failed:', error);
    extensions.value = [];
    iconUrls.value = {};
    emit('status', 'Failed to load apps');
  } finally {
    loading.value = false;
  }
}

/** Open built-in shortcut, extension popup, or extensions manager. */
async function onTileClick(tile: ChromeAppTile): Promise<void> {
  if (tile.kind === 'builtin') {
    if (tile.url === 'chrome://bookmarks') {
      emit('openBookmarksPanel');
      return;
    }
    if (tile.url) {
      try {
        emit('navigate', tile.url);
      } catch (error) {
        console.error('Failed to navigate to app URL:', error);
      }
    }
    return;
  }

  if (tile.extensionId && tile.hasPopup && canUseNativeWebview()) {
    try {
      await invoke('extension_open_popup_window', { extensionId: tile.extensionId });
      return;
    } catch (error) {
      console.error('extension_open_popup_window failed:', error);
      emit('status', `Could not open ${tile.name}`);
    }
  }

  emit('openExtensions');
}

onMounted(() => {
  void loadApps();
});
</script>

<style scoped>
.chrome-apps-page {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 24px 32px 32px;
  background: var(--color-bg-primary, #fff);
  color: var(--color-text-primary, #202124);
}

.chrome-apps-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 24px;
}

.chrome-apps-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
}

.chrome-apps-manage {
  border: 1px solid var(--chrome-divider, #dadce0);
  border-radius: 4px;
  background: var(--color-bg-secondary, #f1f3f4);
  color: var(--color-text-primary, #202124);
  font-size: 13px;
  padding: 8px 14px;
  cursor: pointer;
}

.chrome-apps-manage:hover {
  background: var(--color-bg-tertiary, #e8eaed);
}

.chrome-apps-hint {
  margin: 0;
  color: var(--color-text-secondary, #5f6368);
  font-size: 14px;
}

.chrome-apps-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  gap: 20px 16px;
  max-width: 720px;
}

.chrome-apps-tile {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 8px;
  border: none;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  color: inherit;
  min-width: 0;
}

.chrome-apps-tile:hover {
  background: rgba(0, 0, 0, 0.04);
}

.chrome-apps-tile-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
}

.chrome-apps-tile-icon--letter {
  font-size: 22px;
  font-weight: 600;
  background: linear-gradient(135deg, #e8f0fe 0%, #d2e3fc 100%);
  color: #1a73e8;
}

.chrome-apps-tile-img {
  width: 32px;
  height: 32px;
  object-fit: contain;
}

.chrome-apps-tile-label {
  font-size: 12px;
  line-height: 1.3;
  text-align: center;
  max-width: 88px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (prefers-color-scheme: dark) {
  .chrome-apps-page {
    background: var(--chrome-tab-bg-active, #292a2d);
    color: #e8eaed;
  }

  .chrome-apps-tile:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .chrome-apps-tile-icon--letter {
    background: linear-gradient(135deg, #3c4043 0%, #5f6368 100%);
    color: #8ab4f8;
  }

  .chrome-apps-manage {
    background: #3c4043;
    border-color: #5f6368;
    color: #e8eaed;
  }
}
</style>
