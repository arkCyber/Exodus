<!--
  Exodus Browser — chrome:// internal pages (settings, extensions, hub pages).
-->
<template>
  <div class="chrome-internal-page" :data-page="page">
    <ChromeSettingsPage
      v-if="page === 'settings' || page === 'extensions'"
      :initial-section="settingsSection"
      :content-host="contentHost"
      :p2p-room-id="p2pRoomId"
      :ui-locale="props.uiLocale"
      @close="onCloseSettings"
      @status="(msg: string) => emit('status', msg)"
      @saved="emit('saved')"
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

    <template v-else>
      <header class="chrome-internal-header">
        <h1>{{ title }}</h1>
        <p class="chrome-internal-url">{{ displayUrl }}</p>
      </header>

      <ChromeAppsPage
        v-if="page === 'apps'"
        @navigate="(url: string) => emit('navigate', url)"
        @open-extensions="() => emit('navigate', 'chrome://extensions')"
        @open-bookmarks-panel="() => emit('openPanel', 'bookmarks')"
        @status="(msg: string) => emit('status', msg)"
      />

      <div v-else-if="page === 'unknown'" class="chrome-internal-body">
        <p>This internal page is not available in Exodus yet.</p>
        <button type="button" class="nav-button" @click="emit('navigate', 'chrome://settings')">
          Open Settings
        </button>
      </div>

      <div v-else class="chrome-internal-body">
        <p>{{ hubMessage }}</p>
        <button type="button" class="nav-button" @click="onOpenPanel">
          {{ hubButtonLabel }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — renders chrome:// pages inside the tab content area.
 */
import { computed } from 'vue';
import ChromeSettingsPage from '@/components/ChromeSettingsPage.vue';
import ChromeAppsPage from '@/components/ChromeAppsPage.vue';
import {
  parseChromeInternalUrl,
  chromeInternalTitle,
  normalizeChromeInternalUrl,
  type ChromeInternalPage,
} from '@/lib/chromeInternal';
import { parseChromeSettingsSection, type ChromeSettingsSectionId } from '@/lib/chromeSettingsNav';
import { NEWTAB_INTERNAL_URL } from '@/lib/newTabPage';

const props = defineProps<{
  url: string;
  contentHost?: HTMLElement;
  p2pRoomId?: string;
  uiLocale?: import('@/lib/appLocale').AppLocale;
}>();

const emit = defineEmits<{
  navigate: [url: string];
  status: [message: string];
  saved: [];
  extensionsChanged: [];
  trackingChanged: [];
  openSidebarCustomize: [];
  ntpLayoutReset: [];
  wallpaperChange: [id: string];
  verticalLayoutChange: [settings: import('$lib/verticalTabs').VerticalTabSettings];
  openPanel: [panel: 'memory' | 'bookmarks' | 'downloads'];
  closeTab: [];
  localeChange: [locale: import('@/lib/appLocale').AppLocale];
  /** User closed settings (restore prior page in BrowserPage). */
  close: [];
}>();

const displayUrl = computed(() => normalizeChromeInternalUrl(props.url));

const page = computed((): ChromeInternalPage => parseChromeInternalUrl(props.url) ?? 'unknown');

const title = computed(() => chromeInternalTitle(page.value));

const settingsSection = computed((): ChromeSettingsSectionId => {
  if (page.value === 'extensions') return 'extensions';
  if (page.value === 'settings') return parseChromeSettingsSection(props.url);
  return 'browser';
});

const hubMessage = computed(() => {
  switch (page.value) {
    case 'history':
      return 'Browsing history is available in the sidebar Memory panel.';
    case 'bookmarks':
      return 'Bookmarks are available in the sidebar.';
    case 'downloads':
      return 'Downloads are managed in the downloads panel.';
    default:
      return '';
  }
});

const hubButtonLabel = computed(() => {
  switch (page.value) {
    case 'history':
      return 'Open History';
    case 'bookmarks':
      return 'Open Bookmarks';
    case 'downloads':
      return 'Open Downloads';
    default:
      return 'Open';
  }
});

function onOpenPanel(): void {
  if (page.value === 'history') {
    emit('openPanel', 'memory');
    return;
  }
  if (page.value === 'bookmarks') {
    emit('openPanel', 'bookmarks');
    return;
  }
  if (page.value === 'downloads') {
    emit('openPanel', 'downloads');
  }
}

function onCloseSettings(): void {
  emit('close');
}
</script>

<style scoped>
.chrome-internal-page {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary, #fff);
  color: var(--color-text-primary, #202124);
  overflow: hidden;
}

.chrome-internal-header {
  padding: 20px 24px 12px;
  border-bottom: 1px solid var(--chrome-divider, #dadce0);
  flex-shrink: 0;
}

.chrome-internal-header h1 {
  margin: 0 0 6px;
  font-size: 22px;
  font-weight: 500;
}

.chrome-internal-url {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary, #5f6368);
  font-family: ui-monospace, monospace;
}

.chrome-internal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: flex-start;
}

.nav-button {
  padding: 8px 16px;
  border-radius: 4px;
  border: 1px solid var(--chrome-divider, #dadce0);
  background: var(--color-bg-secondary, #f1f3f4);
  cursor: pointer;
  font-size: 14px;
}

.nav-button:hover {
  background: var(--color-bg-tertiary, #e8eaed);
}
</style>
