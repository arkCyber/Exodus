# New Tab Wallpapers (Brave-style)

Exodus ships a **wallpaper library** for the new tab page, similar to Brave’s background gallery.

## Library location

| Path | Purpose |
|------|---------|
| `static/newtab/wallpapers/manifest.json` | Bundled catalog (dev UI + preload) |
| `static/newtab/wallpapers/*.svg` | Bundled SVG assets |
| `{app_data}/new_tab_page/wallpapers/` | Seeded + custom wallpapers on disk (Rust) |

Bundled themes: **Aurora**, **Deep Ocean**, **Sunset**, **Forest**, **Nebula**, **Midnight**.

**Default wallpaper:** **Nebula** (`defaultId` in both manifest files below).

Drop custom `.svg`, `.png`, or `.jpg` files into the app-data wallpaper folder (shown in **Settings → Appearance → New tab background**). Use **Refresh library** to pick them up.

### Change the default wallpaper (for all new users)

Edit `defaultId` in **both** files (keep them in sync):

- `static/newtab/wallpapers/manifest.json`
- `src-tauri/assets/ntp-wallpaper-manifest.json`

Valid ids: `aurora`, `ocean`, `sunset`, `forest`, `nebula`, `midnight`.

## How it works

1. **Svelte overlay** (`NewTabPage.svelte`) — clock, top sites, quick links, wallpaper picker (primary UI).
2. **WebView underlay** (`newTabPage.ts` data URL) — same wallpaper for native tabs behind the overlay.
3. **Persistence** — selected wallpaper id in `localStorage` key `exodus-ntp-wallpaper-id`.

## User actions

- New tab → **Change background** → pick from grid or **Shuffle**
- Settings → **Appearance** → **New tab background** gallery

## Adding a wallpaper

1. Add `my-theme.svg` under `static/newtab/wallpapers/`.
2. Register in `manifest.json`:

```json
{
  "id": "my-theme",
  "name": "My Theme",
  "file": "my-theme.svg",
  "accent": "#f472b6",
  "description": "Optional short description"
}
```

3. Restart dev server; the new entry appears in Settings and the new-tab picker.

## Code

- `src/lib/newTabWallpaper.ts` — catalog, preload as `data:` URLs, custom merge
- `src-tauri/src/ntp_wallpapers.rs` — seed library, list catalog, read file as data URL
- `src/lib/components/NewTabPage.svelte` — Brave-style UI
- `src/lib/components/NewTabWallpaperSettings.svelte` — Settings panel
