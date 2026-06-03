<script lang="ts">
  /**
   * Exodus Browser — new-tab overlay: top sites (Brave/Chrome style) + quick links.
   */
  import type { QuickLink } from '$lib/browserTypes';
  import { DEFAULT_QUICK_LINKS } from '$lib/newTabPage';

  type Props = {
    visible: boolean;
    topSites?: QuickLink[];
    links?: QuickLink[];
    onNavigate: (url: string) => void;
  };

  let {
    visible,
    topSites = [],
    links = DEFAULT_QUICK_LINKS,
    onNavigate,
  }: Props = $props();

  /** First letter or emoji for top-site tile. */
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
</script>

{#if visible}
  <div class="new-tab-overlay exodus-quick-links" aria-label="New tab">
    {#if topSites.length > 0}
      <section class="top-sites" aria-label="Top sites">
        <p class="section-title">Top sites</p>
        <div class="top-sites-grid">
          {#each topSites as site (site.url)}
            <button
              type="button"
              class="top-site-tile"
              title={site.url}
              onclick={() => onNavigate(site.url)}
            >
              <span class="top-site-icon">{tileLabel(site)}</span>
              <span class="top-site-label">{hostLabel(site)}</span>
            </button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="quick-links-section" aria-label="Quick links">
      <p class="section-title">Quick links</p>
      <div class="quick-links-row">
        {#each links as link (link.url)}
          <button type="button" class="quick-link-chip" onclick={() => onNavigate(link.url)}>
            {link.title}
          </button>
        {/each}
      </div>
    </section>
  </div>
{/if}

<style>
  .new-tab-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 28px;
    padding: 24px;
    pointer-events: none;
    z-index: 2;
  }

  .section-title {
    margin: 0 0 12px;
    color: #a1a1aa;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    text-align: center;
    pointer-events: auto;
  }

  .top-sites,
  .quick-links-section {
    pointer-events: auto;
    max-width: 560px;
    width: 100%;
  }

  .top-sites-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
    gap: 12px;
    justify-items: center;
  }

  .top-site-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 96px;
    padding: 12px 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    background: rgba(22, 22, 30, 0.75);
    backdrop-filter: blur(8px);
    color: #e4e4e7;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .top-site-tile:hover {
    background: rgba(99, 102, 241, 0.35);
    border-color: rgba(99, 102, 241, 0.6);
  }

  .top-site-icon {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: linear-gradient(135deg, #6366f1, #22d3ee);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 18px;
  }

  .top-site-label {
    font-size: 11px;
    max-width: 88px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #c4c4cc;
  }

  .quick-links-row {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    justify-content: center;
  }

  .quick-link-chip {
    background: rgba(51, 51, 51, 0.85);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #eee;
    padding: 10px 18px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
  }

  .quick-link-chip:hover {
    background: #6366f1;
    border-color: #6366f1;
  }
</style>
