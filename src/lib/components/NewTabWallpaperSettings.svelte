<script lang="ts">
  /**
   * Exodus Browser — new tab wallpaper picker (Settings → Appearance).
   */
  import {
    getWallpaperById,
    getWallpaperLibraryPath,
    listWallpapers,
    loadWallpaperCatalog,
    defaultWallpaperId,
    loadWallpaperId,
    resetWallpaperToDefault,
    resolveWallpaperBackgroundUrl,
    saveWallpaperIdAndSync,
  } from '$lib/newTabWallpaper';

  type Props = {
    onStatus: (message: string) => void;
    onWallpaperChange?: (id: string) => void;
  };

  let { onStatus, onWallpaperChange }: Props = $props();

  let selectedId = $state(loadWallpaperId());
  let libraryPath = $state('');
  let wallpapers = $state(listWallpapers());
  let loading = $state(true);

  async function refresh() {
    loading = true;
    try {
      await loadWallpaperCatalog();
      wallpapers = listWallpapers();
      libraryPath = await getWallpaperLibraryPath();
      if (!wallpapers.some((w) => w.id === selectedId)) {
        selectedId = loadWallpaperId();
      }
    } catch (error) {
      console.error('NewTabWallpaperSettings refresh failed:', error);
    } finally {
      loading = false;
    }
  }

  async function select(id: string) {
    selectedId = id;
    await saveWallpaperIdAndSync(id);
    onWallpaperChange?.(id);
    onStatus(`Background: ${getWallpaperById(id).name}`);
  }

  async function resetDefault() {
    const id = await resetWallpaperToDefault();
    selectedId = id;
    onWallpaperChange?.(id);
    onStatus(`Background reset to ${getWallpaperById(id).name}`);
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    void refresh();
  });
</script>

<div class="ntp-settings">
  <h4>New tab background</h4>
  <p class="hint">
    Built-in library plus custom images in your wallpaper folder (Brave-style). Default:
    <strong>{getWallpaperById(defaultWallpaperId()).name}</strong>. Add
    <code>.svg</code>, <code>.png</code>, or <code>.jpg</code> files, then click Refresh.
  </p>
  {#if libraryPath}
    <p class="path" title={libraryPath}>Library: {libraryPath}</p>
  {/if}
  <div class="toolbar">
    <button type="button" class="btn" onclick={() => void refresh()} disabled={loading}>
      {loading ? 'Loading…' : 'Refresh library'}
    </button>
    <button type="button" class="btn secondary" onclick={() => void resetDefault()}>
      Reset to default
    </button>
  </div>
  <div class="grid">
    {#each wallpapers as wp (wp.id)}
      <button
        type="button"
        class="tile"
        class:selected={wp.id === selectedId}
        class:custom={wp.custom}
        style="background-image: url('{resolveWallpaperBackgroundUrl(wp.id)}')"
        title={wp.description ?? wp.name}
        onclick={() => select(wp.id)}
      >
        <span class="name">{wp.name}{wp.custom ? ' · custom' : ''}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .ntp-settings h4 {
    margin: 12px 0 8px;
    font-size: 14px;
    color: #e0e0e0;
  }

  .hint {
    font-size: 12px;
    color: #999;
    margin: 0 0 8px;
    line-height: 1.45;
  }

  .hint code {
    font-size: 11px;
    color: #c4b5fd;
  }

  .path {
    font-size: 11px;
    color: #7a7a85;
    margin: 0 0 10px;
    word-break: break-all;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 10px;
  }

  .btn {
    padding: 6px 12px;
    border-radius: 6px;
    border: 1px solid #555;
    background: #333;
    color: #eee;
    font-size: 12px;
    cursor: pointer;
  }

  .btn.secondary {
    background: #2a2a32;
    border-color: #4b5563;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }

  .tile {
    position: relative;
    aspect-ratio: 16 / 10;
    border-radius: 8px;
    border: 2px solid transparent;
    background-size: cover;
    background-position: center;
    cursor: pointer;
    overflow: hidden;
  }

  .tile.selected {
    border-color: #818cf8;
  }

  .tile.custom {
    box-shadow: inset 0 0 0 1px rgba(34, 211, 238, 0.35);
  }

  .name {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 6px;
    font-size: 11px;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
    color: #eee;
    text-align: left;
  }
</style>
