<script lang="ts">
  /**
   * Exodus Browser — Brave-style new tab overlay (wallpaper, clock, top sites, quick links).
   */
  import type { QuickLink } from '$lib/browserTypes';
  import { DEFAULT_QUICK_LINKS } from '$lib/newTabPage';
  import {
    getWallpaperById,
    listWallpapers,
    randomWallpaperId,
    resolveWallpaperBackgroundUrl,
    type WallpaperEntry,
  } from '$lib/newTabWallpaper';

  type Props = {
    visible: boolean;
    wallpaperId: string;
    topSites?: QuickLink[];
    links?: QuickLink[];
    aiOnline?: boolean;
    aiModel?: string;
    onNavigate: (url: string) => void;
    onWallpaperChange: (id: string) => void;
  };

  let {
    visible,
    wallpaperId,
    topSites = [],
    links = DEFAULT_QUICK_LINKS,
    aiOnline = false,
    aiModel = 'gemma4-e2b',
    onNavigate,
    onWallpaperChange,
  }: Props = $props();

  let showPicker = $state(false);
  let clockText = $state('');
  let dateText = $state('');

  const wallpapers = listWallpapers();
  const activeWallpaper = $derived(getWallpaperById(wallpaperId));
  const backgroundUrl = $derived(resolveWallpaperBackgroundUrl(wallpaperId));

  function tileLabel(link: QuickLink): string {
    const t = link.title?.trim();
    if (t && t.length > 0) return t.charAt(0).toUpperCase();
    try {
      return new URL(link.url).hostname.charAt(0).toUpperCase();
    } catch {
      return '?';
    }
  }

  function hostLabel(link: QuickLink): string {
    try {
      return new URL(link.url).hostname.replace(/^www\./, '');
    } catch {
      return link.title;
    }
  }

  function updateClock() {
    const now = new Date();
    clockText = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    dateText = now.toLocaleDateString([], {
      weekday: 'long',
      month: 'long',
      day: 'numeric',
    });
  }

  function pickWallpaper(entry: WallpaperEntry) {
    onWallpaperChange(entry.id);
    showPicker = false;
  }

  function shuffleWallpaper() {
    onWallpaperChange(randomWallpaperId());
    showPicker = false;
  }

  $effect(() => {
    if (!visible) return;
    updateClock();
    const timer = window.setInterval(updateClock, 30_000);
    return () => clearInterval(timer);
  });
</script>

{#if visible}
  <div
    class="ntp exodus-new-tab"
    style="--ntp-accent: {activeWallpaper.accent}; --ntp-bg: url('{backgroundUrl}')"
    aria-label="New tab"
  >
    <div class="ntp-vignette" aria-hidden="true"></div>

    <header class="ntp-hero">
      <p class="ntp-brand">⛵ Exodus</p>
      <p class="ntp-clock" aria-live="polite">{clockText}</p>
      <p class="ntp-date">{dateText}</p>
      <p class="ntp-ai" class:online={aiOnline}>
        {#if aiOnline}
          Local AI · <span class="model">{aiModel}</span>
        {:else}
          Start Allama on port 11435 for local chat
        {/if}
      </p>
    </header>

    <main class="ntp-body">
      {#if topSites.length > 0}
        <section class="ntp-section" aria-label="Top sites">
          <h2 class="ntp-section-title">Top sites</h2>
          <div class="ntp-top-grid">
            {#each topSites as site (site.url)}
              <button
                type="button"
                class="ntp-tile"
                title={site.url}
                onclick={() => onNavigate(site.url)}
              >
                <span class="ntp-tile-icon">{tileLabel(site)}</span>
                <span class="ntp-tile-label">{hostLabel(site)}</span>
              </button>
            {/each}
          </div>
        </section>
      {/if}

      <section class="ntp-section" aria-label="Quick links">
        <h2 class="ntp-section-title">Quick links</h2>
        <div class="ntp-chips">
          {#each links as link (link.url)}
            <button type="button" class="ntp-chip" onclick={() => onNavigate(link.url)}>
              {link.title}
            </button>
          {/each}
        </div>
      </section>
    </main>

    <footer class="ntp-footer">
      <button
        type="button"
        class="ntp-wallpaper-btn"
        onclick={() => (showPicker = !showPicker)}
        aria-expanded={showPicker}
      >
        Change background
      </button>
      {#if showPicker}
        <div class="ntp-picker" role="dialog" aria-label="Wallpaper library">
          <div class="ntp-picker-header">
            <span>Wallpaper library</span>
            <button type="button" class="ntp-picker-shuffle" onclick={shuffleWallpaper}>
              Shuffle
            </button>
          </div>
          <div class="ntp-picker-grid">
            {#each wallpapers as wp (wp.id)}
              <button
                type="button"
                class="ntp-picker-item"
                class:selected={wp.id === wallpaperId}
                title={wp.description ?? wp.name}
                style="background-image: url('{resolveWallpaperBackgroundUrl(wp.id)}')"
                onclick={() => pickWallpaper(wp)}
              >
                <span class="ntp-picker-name">{wp.name}</span>
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </footer>
  </div>
{/if}

<style>
  .ntp {
    position: absolute;
    inset: 0;
    z-index: 3;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    padding: 48px 32px 28px;
    background-color: #0a0a0f;
    background-image: var(--ntp-bg);
    background-size: cover;
    background-position: center;
    color: #f4f4f5;
    overflow: auto;
    pointer-events: auto;
  }

  .ntp-vignette {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: linear-gradient(
      180deg,
      rgba(6, 8, 14, 0.35) 0%,
      rgba(8, 10, 18, 0.15) 40%,
      rgba(6, 6, 10, 0.55) 100%
    );
  }

  .ntp-hero,
  .ntp-body,
  .ntp-footer {
    position: relative;
    z-index: 1;
    width: 100%;
    max-width: 720px;
  }

  .ntp-hero {
    text-align: center;
  }

  .ntp-brand {
    margin: 0 0 8px;
    font-size: 1.1rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    opacity: 0.9;
  }

  .ntp-clock {
    margin: 0;
    font-size: clamp(3rem, 10vw, 5.5rem);
    font-weight: 300;
    letter-spacing: -0.03em;
    line-height: 1;
    text-shadow: 0 4px 24px rgba(0, 0, 0, 0.45);
  }

  .ntp-date {
    margin: 10px 0 16px;
    font-size: 1rem;
    color: rgba(255, 255, 255, 0.75);
  }

  .ntp-ai {
    margin: 0 auto;
    max-width: 420px;
    font-size: 0.85rem;
    padding: 8px 14px;
    border-radius: 999px;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #d4d4d8;
  }

  .ntp-ai.online {
    border-color: color-mix(in srgb, var(--ntp-accent) 55%, transparent);
    color: #e9e9ff;
  }

  .ntp-ai .model {
    color: var(--ntp-accent);
    font-weight: 600;
  }

  .ntp-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 28px;
    padding: 24px 0;
  }

  .ntp-section-title {
    margin: 0 0 14px;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    text-align: center;
    color: rgba(255, 255, 255, 0.55);
    font-weight: 600;
  }

  .ntp-top-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 14px;
    justify-items: center;
  }

  .ntp-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    width: 100px;
    padding: 14px 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 14px;
    background: rgba(15, 15, 20, 0.55);
    backdrop-filter: blur(14px);
    color: #f4f4f5;
    cursor: pointer;
    transition:
      transform 0.15s ease,
      border-color 0.15s ease,
      background 0.15s ease;
  }

  .ntp-tile:hover {
    transform: translateY(-2px);
    border-color: var(--ntp-accent);
    background: rgba(20, 20, 28, 0.72);
  }

  .ntp-tile-icon {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 1.1rem;
    background: linear-gradient(135deg, var(--ntp-accent), rgba(255, 255, 255, 0.15));
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  }

  .ntp-tile-label {
    font-size: 0.7rem;
    max-width: 92px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.85;
  }

  .ntp-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: center;
  }

  .ntp-chip {
    padding: 10px 18px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.14);
    background: rgba(15, 15, 20, 0.5);
    backdrop-filter: blur(10px);
    color: #f4f4f5;
    font-size: 0.9rem;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .ntp-chip:hover {
    background: color-mix(in srgb, var(--ntp-accent) 35%, rgba(15, 15, 20, 0.6));
    border-color: var(--ntp-accent);
  }

  .ntp-footer {
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
</style>
