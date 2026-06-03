# Svelte routes retired (Vue 3 entry)

The browser shell **no longer loads** SvelteKit routes. Production entry:

- [`/index.html`](../../index.html) → [`/src/main.ts`](../main.ts) → [`/src/views/BrowserPage.vue`](../views/BrowserPage.vue)

## Archived sources

| Former path | Archive |
|-------------|---------|
| `+page.svelte` (main shell, ~3500 lines) | [`_archive/+page.svelte.legacy`](./_archive/+page.svelte.legacy) |
| `loading/+page.svelte` | [`_archive/loading-page.svelte.legacy`](./_archive/loading-page.svelte.legacy) |

`+layout.ts` remains only as a historical SvelteKit SPA marker; it is not imported by the Vite + Vue build.

## Migration reference

See [`/VUE3_MIGRATION_GUIDE.md`](../../VUE3_MIGRATION_GUIDE.md) for the feature parity checklist. Do not add new features to the archived Svelte files.

## Svelte IM components

Legacy IM Svelte sources live under [`src/lib/components/_archive/im/`](../lib/components/_archive/im/IM_RETIRED.md) (replaced by Vue 3 WebChat).
