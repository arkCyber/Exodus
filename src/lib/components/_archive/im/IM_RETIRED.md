# Svelte IM components retired (Vue 3 WebChat)

The browser shell **no longer loads** these Svelte IM components. Production WebChat stack:

- [`BrowserPage.vue`](../../../views/BrowserPage.vue) → [`ImMessenger.vue`](../../../components/ImMessenger.vue)
- [`P2pSidebarPanel.vue`](../../../components/P2pSidebarPanel.vue)
- [`ContactDirectoryPanel.vue`](../../../components/ContactDirectoryPanel.vue)
- [`RtcCallHost.vue`](../../../components/RtcCallHost.vue)
- [`MentionMessageBody.vue`](../../../components/MentionMessageBody.vue)

## Archived files

| Former path | Archive |
|-------------|---------|
| `ImMessenger.svelte` | [`ImMessenger.svelte`](./ImMessenger.svelte) |
| `P2pSidebarPanel.svelte` | [`P2pSidebarPanel.svelte`](./P2pSidebarPanel.svelte) |
| `ContactDirectory.svelte` | [`ContactDirectory.svelte`](./ContactDirectory.svelte) |
| `RtcCallHost.svelte` | [`RtcCallHost.svelte`](./RtcCallHost.svelte) |
| `RtcCallOverlay.svelte` | [`RtcCallOverlay.svelte`](./RtcCallOverlay.svelte) |
| `MentionMessageBody.svelte` | [`MentionMessageBody.svelte`](./MentionMessageBody.svelte) |

Vue replacement: [`MentionMessageBody.vue`](../../../components/MentionMessageBody.vue). Legacy `GroupChatPanel.svelte` imports the archived Svelte copy.

Do not add new features here. See [`/VUE3_MIGRATION_GUIDE.md`](../../../../VUE3_MIGRATION_GUIDE.md).
