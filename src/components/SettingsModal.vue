<!--
  Exodus Browser — legacy settings modal wrapper (prefer chrome://settings full page).
-->
<template>
  <Teleport v-if="open && !embedded" to="body">
    <div class="settings-modal-shell">
      <ChromeSettingsPage
        :initial-section="modalSection"
        :content-host="contentHost"
        :p2p-room-id="p2pRoomId"
        :ui-locale="props.uiLocale"
        @close="emit('close')"
        @status="(msg: string) => emit('status', msg)"
        @saved="onSaved"
        @extensions-changed="emit('extensionsChanged')"
        @tracking-changed="emit('trackingChanged')"
        @open-sidebar-customize="emit('openSidebarCustomize')"
        @ntp-layout-reset="emit('ntpLayoutReset')"
        @wallpaper-change="(id: string) => emit('wallpaperChange', id)"
        @vertical-layout-change="(s) => emit('verticalLayoutChange', s)"
        @open-panel="(panel) => emit('openPanel', panel)"
        @navigate="(url: string) => emit('navigate', url)"
        @locale-change="(loc) => emit('localeChange', loc)"
      />
    </div>
  </Teleport>

  <ChromeSettingsPage
    v-else-if="open && embedded"
    :initial-section="modalSection"
    :content-host="contentHost"
    :p2p-room-id="p2pRoomId"
    :ui-locale="props.uiLocale"
    @close="emit('close')"
    @status="(msg: string) => emit('status', msg)"
    @saved="onSaved"
    @extensions-changed="emit('extensionsChanged')"
    @tracking-changed="emit('trackingChanged')"
    @open-sidebar-customize="emit('openSidebarCustomize')"
    @ntp-layout-reset="emit('ntpLayoutReset')"
    @wallpaper-change="(id: string) => emit('wallpaperChange', id)"
    @vertical-layout-change="(s) => emit('verticalLayoutChange', s)"
    @open-panel="(panel) => emit('openPanel', panel)"
    @navigate="(url: string) => emit('navigate', url)"
    @locale-change="(loc) => emit('localeChange', loc)"
  />
</template>

<script setup lang="ts">
/**
 * Exodus Browser — backward-compatible settings entry (wraps ChromeSettingsPage).
 * Menu and shortcuts should navigate to chrome://settings instead of this modal.
 */
import { computed } from 'vue';
import ChromeSettingsPage from '@/components/ChromeSettingsPage.vue';
import { normalizeChromeSettingsSection, type ChromeSettingsSectionId } from '@/lib/chromeSettingsNav';
import type { AppLocale } from '@/lib/appLocale';

const props = defineProps<{
  open: boolean;
  embedded?: boolean;
  contentHost?: HTMLElement;
  p2pRoomId?: string;
  uiLocale?: AppLocale;
  scrollToSection?: 'privacy' | 'history' | 'extensions' | 'tabs' | 'downloads' | null;
}>();

const emit = defineEmits<{
  close: [];
  status: [message: string];
  extensionsChanged: [];
  saved: [];
  wallpaperChange: [id: string];
  ntpLayoutReset: [];
  verticalLayoutChange: [settings: import('$lib/verticalTabs').VerticalTabSettings];
  trackingChanged: [];
  openSidebarCustomize: [];
  openPanel: [panel: 'memory' | 'bookmarks' | 'downloads'];
  navigate: [url: string];
  localeChange: [locale: AppLocale];
}>();

const scrollMap: Record<string, ChromeSettingsSectionId> = {
  privacy: 'privacy',
  history: 'history',
  extensions: 'extensions',
  tabs: 'sidebar',
};

const modalSection = computed(() =>
  normalizeChromeSettingsSection(
    props.scrollToSection ? scrollMap[props.scrollToSection] ?? props.scrollToSection : 'browser',
  ),
);

function onSaved(): void {
  emit('saved');
}
</script>

<style scoped>
.settings-modal-shell {
  position: fixed;
  inset: 0;
  z-index: 2000;
  display: flex;
  background: rgba(0, 0, 0, 0.35);
}

.settings-modal-shell :deep(.chrome-settings) {
  width: 100%;
  height: 100%;
}
</style>
