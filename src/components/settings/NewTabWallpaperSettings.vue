<!--
  Exodus Browser — new tab wallpaper picker (Settings → Appearance).
-->
<template>
  <section class="settings-section" data-testid="new-tab-wallpaper-settings">
    <h3>New tab background</h3>
    <div v-if="loading" class="loading-state">Loading wallpapers…</div>
    <div v-else class="grid">
      <button
        v-for="w in wallpapers"
        :key="w.id"
        type="button"
        class="tile"
        :class="{ active: w.id === selectedId }"
        @click="() => void select(w.id)"
        data-testid="wallpaper-tile"
      >
        <div class="tile-preview" :style="{ background: w.accent }">
          <img v-if="w.file" :src="wallpaperAssetUrl(w.file)" :alt="w.name" @error="handleImageError" />
        </div>
        <span class="tile-name">{{ w.name }}</span>
      </button>
    </div>
    <div class="actions">
      <button type="button" class="nav-button secondary" @click="() => void resetDefault()" data-testid="wallpaper-reset-default">Reset default</button>
      <button type="button" class="nav-button secondary" @click="showDownloadDialog = true" data-testid="wallpaper-download">Download wallpaper</button>
    </div>

    <!-- Download dialog -->
    <div v-if="showDownloadDialog" class="download-dialog-backdrop" @click="showDownloadDialog = false">
      <div class="download-dialog" @click.stop>
        <h4>Download Wallpaper</h4>
        <input
          v-model="downloadUrl"
          type="text"
          placeholder="https://example.com/wallpaper.jpg"
          class="download-input"
          data-testid="wallpaper-download-url"
        />
        <input
          v-model="downloadName"
          type="text"
          placeholder="Wallpaper name"
          class="download-input"
          data-testid="wallpaper-download-name"
        />
        <div class="download-actions">
          <button type="button" class="nav-button secondary" @click="showDownloadDialog = false" data-testid="wallpaper-download-cancel">Cancel</button>
          <button type="button" class="nav-button primary" @click="downloadWallpaper" :disabled="downloading" data-testid="wallpaper-download-submit">
            {{ downloading ? 'Downloading...' : 'Download' }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { canInvokeTauri } from '@/lib/tauri';
import {
  getWallpaperById,
  listWallpapers,
  loadWallpaperId,
  resetWallpaperToDefault,
  saveWallpaperIdAndSync,
  wallpaperAssetUrl,
  type WallpaperEntry,
} from '$lib/newTabWallpaper';

const emit = defineEmits<{
  status: [message: string];
  wallpaperChange: [id: string];
}>();

const selectedId = ref(loadWallpaperId());
const wallpapers = ref<WallpaperEntry[]>(listWallpapers());
const loading = ref(false);
const showDownloadDialog = ref(false);
const downloadUrl = ref('');
const downloadName = ref('');
const downloading = ref(false);

function refresh(): void {
  // Use bundled catalog directly (no backend calls, no blocking, synchronous)
  wallpapers.value = listWallpapers();
  console.log('[NewTabWallpaperSettings] Loaded wallpapers from bundled catalog:', wallpapers.value.length);
  console.log('[NewTabWallpaperSettings] Wallpaper IDs:', wallpapers.value.map(w => w.id).join(', '));
  
  // Set preview URLs: all use asset URLs (bundled wallpapers are in static/)
  wallpapers.value = wallpapers.value.map(w => {
    const url = wallpaperAssetUrl(w.file);
    console.log(`[NewTabWallpaperSettings] ${w.id} (${w.file}) -> asset URL: ${url}`);
    return {
      ...w,
      previewUrl: url
    };
  });
  
  console.log('[NewTabWallpaperSettings] Preview URLs set (all static asset URLs)');
  console.log('[NewTabWallpaperSettings] Total wallpapers to display:', wallpapers.value.length);
  
  if (!wallpapers.value.some((w) => w.id === selectedId.value)) {
    selectedId.value = loadWallpaperId();
  }
}

async function select(id: string): Promise<void> {
  selectedId.value = id;
  // Save wallpaper ID asynchronously without blocking UI
  void saveWallpaperIdAndSync(id).then(() => {
    emit('wallpaperChange', id);
    emit('status', `Background: ${getWallpaperById(id).name}`);
  }).catch((error) => {
    console.error('[NewTabWallpaperSettings] Failed to save wallpaper:', error);
    emit('status', 'Failed to save wallpaper selection');
  });
}

async function resetDefault(): Promise<void> {
  // Reset wallpaper asynchronously without blocking UI
  void resetWallpaperToDefault().then((id) => {
    selectedId.value = id;
    emit('wallpaperChange', id);
    emit('status', `Background reset to ${getWallpaperById(id).name}`);
  }).catch((error) => {
    console.error('[NewTabWallpaperSettings] Failed to reset wallpaper:', error);
    emit('status', 'Failed to reset wallpaper');
  });
}

function handleImageError(event: Event): void {
  const img = event.target as HTMLImageElement;
  img.style.display = 'none';
}

async function downloadWallpaper(): Promise<void> {
  if (!downloadUrl.value || !downloadName.value) {
    emit('status', 'Please enter URL and name');
    return;
  }
  if (!canInvokeTauri()) {
    emit('status', 'Download wallpaper requires the Tauri desktop app');
    return;
  }

  downloading.value = true;
  try {
    const id = `custom-${Date.now()}`;
    await invoke<string>('ntp_download_wallpaper', {
      url: downloadUrl.value,
      filename: '',
      name: downloadName.value,
      id,
    });

    const savedName = downloadName.value;
    refresh();
    select(id);
    showDownloadDialog.value = false;
    downloadUrl.value = '';
    downloadName.value = '';
    emit('status', `Wallpaper "${savedName}" downloaded successfully`);
  } catch (error) {
    console.error('[NewTabWallpaperSettings] downloadWallpaper failed:', error);
    emit('status', `Failed to download wallpaper: ${error}`);
  } finally {
    downloading.value = false;
  }
}

onMounted(() => void refresh());
</script>

<style scoped>
.grid { display: flex; flex-wrap: wrap; gap: 12px; margin-bottom: 16px; }
.loading-state { padding: 20px; text-align: center; color: var(--color-text-secondary, #9ca3af); }
.tile { 
  display: flex; 
  flex-direction: column; 
  align-items: center; 
  padding: 8px; 
  border-radius: 8px; 
  border: 2px solid var(--color-border, #404040); 
  cursor: pointer; 
  background: var(--color-bg-secondary, #2a2a2a); 
  color: inherit; 
  width: 100px;
  transition: all 0.2s ease;
}
.tile:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
.tile.active { border-color: var(--color-accent, #2563eb); box-shadow: 0 0 0 2px var(--color-accent, #2563eb); }
.tile-preview { 
  width: 80px; 
  height: 60px; 
  border-radius: 4px; 
  overflow: hidden; 
  display: flex; 
  align-items: center; 
  justify-content: center;
  margin-bottom: 8px;
}
.tile-preview img { 
  width: 100%; 
  height: 100%; 
  object-fit: cover; 
}
.tile-name { 
  font-size: 11px; 
  text-align: center; 
  overflow: hidden; 
  text-overflow: ellipsis; 
  white-space: nowrap; 
  width: 100%;
}
.hint { font-size: 12px; color: var(--color-text-secondary, #888); }
.actions { display: flex; gap: 8px; }
.nav-button.secondary { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-bg-tertiary, #404040); color: #fff; }
.nav-button.primary { padding: 8px 16px; border: none; border-radius: 6px; cursor: pointer; background: var(--color-accent, #2563eb); color: #fff; }
.nav-button:disabled { opacity: 0.6; cursor: not-allowed; }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; text-transform: uppercase; color: var(--color-text-secondary, #9ca3af); }

.download-dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.download-dialog {
  background: var(--color-bg-primary, #1a1a1a);
  border: 1px solid var(--color-border, #404040);
  border-radius: 8px;
  padding: 20px;
  min-width: 320px;
  max-width: 480px;
}

.download-dialog h4 {
  margin: 0 0 16px;
  font-size: 16px;
  color: var(--color-text-primary, #fff);
}

.download-input {
  width: 100%;
  padding: 8px 12px;
  margin-bottom: 12px;
  border: 1px solid var(--color-border, #404040);
  border-radius: 4px;
  background: var(--color-bg-secondary, #2a2a2a);
  color: var(--color-text-primary, #fff);
  box-sizing: border-box;
}

.download-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
