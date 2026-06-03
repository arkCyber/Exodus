<!--
  Exodus Browser — Chrome-style full-page settings (sidebar + content panels).
-->
<template>
  <div
    class="chrome-settings"
    data-testid="chrome-settings-page"
    :data-section="activeSection"
    :data-theme="themeDataAttr"
  >
    <aside class="chrome-settings__sidebar" aria-label="Settings categories">
      <div class="chrome-settings__brand">
        <svg class="chrome-settings__logo" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="1.5" />
          <circle cx="12" cy="12" r="4" fill="currentColor" />
        </svg>
        <span class="chrome-settings__brand-label">{{ ui.pageTitle }}</span>
      </div>

      <label class="chrome-settings__search">
        <span class="sr-only">{{ ui.searchPlaceholder }}</span>
        <input
          v-model="searchQuery"
          type="search"
          class="chrome-settings__search-input"
          data-testid="chrome-settings-search"
          :placeholder="ui.searchPlaceholder"
          autocomplete="off"
        />
      </label>

      <nav class="chrome-settings__nav" role="navigation">
        <template v-for="item in filteredNav" :key="item.id">
          <button
            type="button"
            class="chrome-settings__nav-item"
            :class="{ 'chrome-settings__nav-item--active': activeSection === item.id }"
            :data-testid="`chrome-settings-nav-${item.id}`"
            @click="selectSectionWithExternal(item.id)"
          >
            <ChromeSettingsNavIcon :icon="item.icon" />
            <span class="chrome-settings__nav-label">{{ item.label }}</span>
            <svg
              v-if="item.external"
              class="chrome-settings__nav-external"
              viewBox="0 0 16 16"
              fill="none"
              aria-hidden="true"
            >
              <path d="M6 2.5h7v7M13 3 6.5 9.5M3 6v7.5h7.5" stroke="currentColor" stroke-width="1.2" />
            </svg>
          </button>
        </template>
      </nav>
    </aside>

    <main class="chrome-settings__main">
      <header class="chrome-settings__header">
        <div class="chrome-settings__header-text">
          <h1 class="chrome-settings__title">{{ ui.sectionTitle(activeSection) }}</h1>
          <p v-if="ui.sectionDescription(activeSection)" class="chrome-settings__subtitle">
            {{ ui.sectionDescription(activeSection) }}
          </p>
        </div>
        <button
          type="button"
          class="chrome-settings__close"
          data-testid="chrome-settings-close"
          @click="emit('close')"
        >
          {{ ui.close }}
        </button>
      </header>

      <div ref="mainScrollEl" class="chrome-settings__content">
        <!-- Browser -->
        <section
          v-if="isSectionMounted('browser')"
          v-show="activeSection === 'browser'"
          id="settings-section-browser"
          class="settings-section"
          data-testid="settings-section-browser"
        >
          <div class="settings-card">
            <h3 class="settings-card__title">{{ sectionUi.browser.generalTitle }}</h3>
            <label>
              {{ sectionUi.browser.homepageLabel }}
              <input
                v-model="config.homepageUrl"
                type="text"
                data-testid="settings-homepage-url"
                :placeholder="sectionUi.browser.homepagePlaceholder"
              />
            </label>
            <label>
              {{ sectionUi.browser.searchLabel }}
              <input
                v-model="config.searchEngineUrl"
                type="text"
                data-testid="settings-search-url"
                :placeholder="sectionUi.browser.searchPlaceholder"
              />
            </label>
            <label class="checkbox-row">
              <input v-model="config.showBookmarkBar" type="checkbox" data-testid="settings-bookmark-bar" />
              <span>{{ sectionUi.browser.bookmarkBar }}</span>
            </label>
          </div>
        </section>

        <!-- Autofill -->
        <section
          v-if="isSectionMounted('autofill')"
          v-show="activeSection === 'autofill'"
          id="settings-section-autofill"
          class="settings-section"
          data-testid="settings-section-autofill"
        >
          <PasswordManagerSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Privacy -->
        <section
          v-if="isSectionMounted('privacy')"
          v-show="activeSection === 'privacy'"
          id="settings-section-privacy"
          class="settings-section"
          data-testid="settings-section-privacy"
        >
          <div class="settings-card">
            <PrivacyShieldSettings @status="onStatus" @tracking-changed="emit('trackingChanged')" />
          </div>
          <ClearBrowsingDataSettings
            :ui-locale="resolvedUiLocale"
            @status="onStatus"
            @cleared="emit('trackingChanged')"
          />
          <div class="settings-card">
            <h3 class="settings-card__title">{{ sectionUi.privacy.controlsTitle }}</h3>
            <label class="checkbox-row">
              <input v-model="config.httpsOnly" type="checkbox" data-testid="settings-https-only" />
              <span>{{ sectionUi.privacy.httpsOnly }}</span>
            </label>
            <label class="checkbox-row">
              <input v-model="config.privateMode" type="checkbox" data-testid="settings-private-mode" />
              <span>{{ sectionUi.privacy.privateMode }}</span>
            </label>
            <label class="checkbox-row">
              <input v-model="config.blockPopups" type="checkbox" data-testid="settings-block-popups" />
              <span>{{ sectionUi.privacy.blockPopups }}</span>
            </label>
          </div>
          <CookieManagerSettings @status="onStatus" />
          <BrowserSitePermissionsSettings @status="onStatus" />
        </section>

        <!-- Appearance -->
        <section
          v-if="isSectionMounted('appearance')"
          v-show="activeSection === 'appearance'"
          id="settings-section-appearance"
          class="settings-section"
          data-testid="settings-section-appearance"
        >
          <AppearancePreferencesSettings
            :ui-locale="resolvedUiLocale"
            @status="onStatus"
            @locale-change="(loc) => emit('localeChange', loc)"
          />
          <VerticalTabsSettings @status="onStatus" @layout-change="(s) => emit('verticalLayoutChange', s)" />
          <NewTabWallpaperSettings
            v-if="wallpaperFeatureEnabled"
            @status="onStatus"
            @wallpaper-change="(id: string) => emit('wallpaperChange', id)"
          />
          <NewTabLayoutSettings @status="onStatus" @ntp-layout-reset="emit('ntpLayoutReset')" />
        </section>

        <!-- Startup -->
        <section
          v-if="isSectionMounted('startup')"
          v-show="activeSection === 'startup'"
          id="settings-section-startup"
          class="settings-section"
          data-testid="settings-section-startup"
        >
          <div class="settings-card">
            <h3 class="settings-card__title">{{ sectionUi.startup.title }}</h3>
            <label class="checkbox-row">
              <input v-model="config.sessionRestore" type="checkbox" data-testid="settings-session-restore" />
              <span>{{ sectionUi.startup.restoreTabs }}</span>
            </label>
            <p class="settings-hint">{{ sectionUi.startup.hint }}</p>
          </div>
        </section>

        <!-- AI -->
        <section
          v-if="isSectionMounted('ai')"
          v-show="activeSection === 'ai'"
          id="settings-section-ai"
          class="settings-section"
          data-testid="settings-section-ai"
        >
          <AllamaServiceSettings
            :spawn-allama="config.spawnAllama.value"
            :ai-port="config.aiPort.value"
            @status="onStatus"
            @spawn-allama-change="(v: boolean) => { config.spawnAllama.value = v; }"
            @ai-port-change="(v: number) => { config.aiPort.value = v; }"
          />
          <InferenceEngineSettings
            :ai-model="config.aiModel.value"
            @status="onStatus"
            @model-change="(m: string) => { config.aiModel.value = m; }"
          />
        </section>

        <!-- Search Engine -->
        <section
          v-if="isSectionMounted('search')"
          v-show="activeSection === 'search'"
          id="settings-section-search"
          class="settings-section"
          data-testid="settings-section-search"
        >
          <SearchEngineSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Fonts and Zoom -->
        <section
          v-if="isSectionMounted('fonts')"
          v-show="activeSection === 'fonts'"
          id="settings-section-fonts"
          class="settings-section"
          data-testid="settings-section-fonts"
        >
          <FontAndZoomSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Network -->
        <section
          v-if="isSectionMounted('network')"
          v-show="activeSection === 'network'"
          id="settings-section-network"
          class="settings-section"
          data-testid="settings-section-network"
        >
          <NetworkSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Media -->
        <section
          v-if="isSectionMounted('media')"
          v-show="activeSection === 'media'"
          id="settings-section-media"
          class="settings-section"
          data-testid="settings-section-media"
        >
          <MediaSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Notifications -->
        <section
          v-if="isSectionMounted('notifications')"
          v-show="activeSection === 'notifications'"
          id="settings-section-notifications"
          class="settings-section"
          data-testid="settings-section-notifications"
        >
          <NotificationSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Accessibility -->
        <section
          v-if="isSectionMounted('accessibility')"
          v-show="activeSection === 'accessibility'"
          id="settings-section-accessibility"
          class="settings-section"
          data-testid="settings-section-accessibility"
        >
          <AccessibilitySettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Keyboard Shortcuts -->
        <section
          v-if="isSectionMounted('shortcuts')"
          v-show="activeSection === 'shortcuts'"
          id="settings-section-shortcuts"
          class="settings-section"
          data-testid="settings-section-shortcuts"
        >
          <KeyboardShortcutsSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- System -->
        <section
          v-if="isSectionMounted('system')"
          v-show="activeSection === 'system'"
          id="settings-section-system"
          class="settings-section"
          data-testid="settings-section-system"
        >
          <SystemSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Sync -->
        <section
          v-if="isSectionMounted('sync')"
          v-show="activeSection === 'sync'"
          id="settings-section-sync"
          class="settings-section"
          data-testid="settings-section-sync"
        >
          <SyncSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Profile -->
        <section
          v-if="isSectionMounted('profile')"
          v-show="activeSection === 'profile'"
          id="settings-section-profile"
          class="settings-section"
          data-testid="settings-section-profile"
        >
          <ProfileSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Performance -->
        <section
          v-if="isSectionMounted('performance')"
          v-show="activeSection === 'performance'"
          id="settings-section-performance"
          class="settings-section"
          data-testid="settings-section-performance"
        >
          <PerformanceSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- GPU -->
        <section
          v-if="isSectionMounted('gpu')"
          v-show="activeSection === 'gpu'"
          id="settings-section-gpu"
          class="settings-section"
          data-testid="settings-section-gpu"
        >
          <GpuSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
        </section>

        <!-- Extensions -->
        <section
          v-if="isSectionMounted('extensions')"
          v-show="activeSection === 'extensions'"
          id="settings-section-extensions"
          class="settings-section"
          data-testid="settings-section-extensions"
        >
          <ExtensionsSettings
            :content-host="contentHost"
            :ui-locale="resolvedUiLocale"
            @status="onStatus"
            @extensions-changed="emit('extensionsChanged')"
            @open-apps="emit('navigate', 'chrome://apps')"
          />
        </section>

        <!-- Plugins -->
        <section
          v-if="isSectionMounted('plugins')"
          v-show="activeSection === 'plugins'"
          id="settings-section-plugins"
          class="settings-section"
          data-testid="settings-section-plugins"
        >
          <PluginSandboxSettings @status="onStatus" />
        </section>

        <!-- Sidebar -->
        <section
          v-if="isSectionMounted('sidebar')"
          v-show="activeSection === 'sidebar'"
          id="settings-section-sidebar"
          class="settings-section"
          data-testid="settings-section-sidebar"
        >
          <div class="settings-card">
            <h3 class="settings-card__title">{{ sectionUi.sidebar.title }}</h3>
            <p class="settings-hint">{{ sectionUi.sidebar.hint }}</p>
            <button type="button" class="settings-link-btn" data-testid="settings-sidebar-customize" @click="emit('openSidebarCustomize')">
              {{ sectionUi.sidebar.customize }}
            </button>
          </div>
        </section>

        <!-- History -->
        <section
          v-if="isSectionMounted('history')"
          v-show="activeSection === 'history'"
          id="settings-section-history"
          class="settings-section"
          data-testid="settings-section-history"
        >
          <HistoryManagerSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
          <div class="settings-card">
            <button type="button" class="settings-link-btn" data-testid="settings-open-history-panel" @click="emit('openPanel', 'memory')">
              {{ sectionUi.history.openSidebar }}
            </button>
          </div>
        </section>

        <!-- Downloads -->
        <section
          v-if="isSectionMounted('downloads')"
          v-show="activeSection === 'downloads'"
          id="settings-section-downloads"
          class="settings-section"
          data-testid="settings-section-downloads"
        >
          <DownloadsSettings :ui-locale="resolvedUiLocale" @status="onStatus" />
          <div class="settings-card">
            <p class="settings-hint">{{ sectionUi.downloads.hint }}</p>
            <button type="button" class="settings-link-btn" data-testid="settings-open-downloads-panel" @click="emit('openPanel', 'downloads')">
              {{ sectionUi.downloads.openPanel }}
            </button>
          </div>
        </section>

        <!-- P2P -->
        <section
          v-if="isSectionMounted('p2p')"
          v-show="activeSection === 'p2p'"
          id="settings-section-p2p"
          class="settings-section settings-section--stack"
          data-testid="settings-section-p2p"
        >
          <P2pCdnSettings :room-id="p2pRoomId" @status="onStatus" />
          <GroupChatSettings :group-id="p2pRoomId" @status="onStatus" />
        </section>

        <!-- Reset -->
        <section
          v-if="isSectionMounted('reset')"
          v-show="activeSection === 'reset'"
          id="settings-section-reset"
          class="settings-section"
          data-testid="settings-section-reset"
        >
          <div class="settings-card">
            <h3 class="settings-card__title">{{ sectionUi.reset.title }}</h3>
            <p class="settings-hint">{{ sectionUi.reset.hint }}</p>
            <button type="button" class="settings-link-btn" data-testid="settings-ntp-reset" @click="emit('ntpLayoutReset')">
              {{ sectionUi.reset.button }}
            </button>
          </div>
        </section>

        <!-- About -->
        <section
          v-if="isSectionMounted('about')"
          v-show="activeSection === 'about'"
          id="settings-section-about"
          class="settings-section"
          data-testid="settings-section-about"
        >
          <div class="settings-card" data-testid="settings-about-card">
            <h3 class="settings-card__title">{{ sectionUi.about.productTitle }}</h3>
            <p class="settings-hint">{{ sectionUi.about.tagline }}</p>
            <dl class="settings-about-list">
              <dt>{{ sectionUi.about.versionLabel }}</dt>
              <dd data-testid="settings-about-version">{{ appVersion }}</dd>
              <dt>{{ sectionUi.about.buildLabel }}</dt>
              <dd data-testid="settings-about-build">{{ appBuildStack }}</dd>
              <dt>{{ sectionUi.about.settingsUrlLabel }}</dt>
              <dd><code>chrome://settings</code></dd>
            </dl>
          </div>
        </section>
      </div>

      <footer class="chrome-settings__footer" aria-live="polite">
        <span
          class="chrome-settings__autosave"
          :class="`chrome-settings__autosave--${autoSaveState}`"
          data-testid="chrome-settings-autosave-status"
        >
          {{ autoSaveLabel }}
        </span>
      </footer>
    </main>
  </div>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — full-page Chrome settings shell (sidebar navigation + section panels).
 */
import { computed, ref, watch, onMounted, onUnmounted, defineComponent, h } from 'vue';
import ExtensionsSettings from '@/components/ExtensionsSettings.vue';
import AllamaServiceSettings from '@/components/settings/AllamaServiceSettings.vue';
import InferenceEngineSettings from '@/components/settings/InferenceEngineSettings.vue';
import PasswordManagerSettings from '@/components/settings/PasswordManagerSettings.vue';
import CookieManagerSettings from '@/components/settings/CookieManagerSettings.vue';
import HistoryManagerSettings from '@/components/settings/HistoryManagerSettings.vue';
import BrowserSitePermissionsSettings from '@/components/settings/BrowserSitePermissionsSettings.vue';
import PrivacyShieldSettings from '@/components/settings/PrivacyShieldSettings.vue';
import VerticalTabsSettings from '@/components/settings/VerticalTabsSettings.vue';
import NewTabWallpaperSettings from '@/components/settings/NewTabWallpaperSettings.vue';
import NewTabLayoutSettings from '@/components/settings/NewTabLayoutSettings.vue';
import AppearancePreferencesSettings from '@/components/settings/AppearancePreferencesSettings.vue';
import DownloadsSettings from '@/components/settings/DownloadsSettings.vue';
import ClearBrowsingDataSettings from '@/components/settings/ClearBrowsingDataSettings.vue';
import PluginSandboxSettings from '@/components/settings/PluginSandboxSettings.vue';
import SearchEngineSettings from '@/components/settings/SearchEngineSettings.vue';
import FontAndZoomSettings from '@/components/settings/FontAndZoomSettings.vue';
import NetworkSettings from '@/components/settings/NetworkSettings.vue';
import MediaSettings from '@/components/settings/MediaSettings.vue';
import NotificationSettings from '@/components/settings/NotificationSettings.vue';
import AccessibilitySettings from '@/components/settings/AccessibilitySettings.vue';
import KeyboardShortcutsSettings from '@/components/settings/KeyboardShortcutsSettings.vue';
import SystemSettings from '@/components/settings/SystemSettings.vue';
import SyncSettings from '@/components/settings/SyncSettings.vue';
import ProfileSettings from '@/components/settings/ProfileSettings.vue';
import PerformanceSettings from '@/components/settings/PerformanceSettings.vue';
import GpuSettings from '@/components/settings/GpuSettings.vue';
import { APP_BUILD_STACK, APP_PACKAGE_VERSION } from '@/lib/appVersion';
import { chromeSettingsSectionUi } from '@/lib/chromeSettingsSectionUi';
import { useTheme } from '@/composables/useTheme';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';
import { WALLPAPER_FEATURE_ENABLED } from '@/lib/newTabWallpaper';
import P2pCdnSettings from '@/components/settings/P2pCdnSettings.vue';
import GroupChatSettings from '@/components/settings/GroupChatSettings.vue';
import { useBrowserConfig } from '@/composables/useBrowserConfig';
import { useChromeSettingsAutoSave } from '@/composables/useChromeSettingsAutoSave';
import {
  chromeSettingsNavItems,
  chromeSettingsStrings,
  filterChromeSettingsNav,
  normalizeChromeSettingsSection,
  chromeSettingsUrlForSection,
  type ChromeSettingsNavIcon,
  type ChromeSettingsSectionId,
} from '@/lib/chromeSettingsNav';

const props = defineProps<{
  /** Initial sidebar section (from chrome://settings/… or extensions page). */
  initialSection?: ChromeSettingsSectionId | string | null;
  contentHost?: HTMLElement;
  p2pRoomId?: string;
  uiLocale?: AppLocale;
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
  /** Sync omnibox / tab URL when sidebar section changes (chrome://settings/…). */
  navigate: [url: string];
  /** UI language changed in Appearance → Language. */
  localeChange: [locale: AppLocale];
}>();

const config = useBrowserConfig();
const wallpaperFeatureEnabled = WALLPAPER_FEATURE_ENABLED;
const searchQuery = ref('');
const activeSection = ref<ChromeSettingsSectionId>(
  normalizeChromeSettingsSection(props.initialSection ?? 'browser'),
);
/** Sections that have been opened at least once (lazy-mount heavy panels). */
const mountedSections = ref<Set<ChromeSettingsSectionId>>(
  new Set([normalizeChromeSettingsSection(props.initialSection ?? 'browser')]),
);
const mainScrollEl = ref<HTMLElement>();
const p2pRoomId = ref(props.p2pRoomId ?? 'lobby');

function markSectionMounted(id: ChromeSettingsSectionId): void {
  if (mountedSections.value.has(id)) return;
  mountedSections.value = new Set([...mountedSections.value, id]);
}

function isSectionMounted(id: ChromeSettingsSectionId): boolean {
  return mountedSections.value.has(id);
}

const { isDark } = useTheme();
const resolvedUiLocale = computed(() => resolveAppLocale(props.uiLocale));
const themeDataAttr = computed(() => (isDark.value ? 'dark' : 'light'));
const ui = computed(() => chromeSettingsStrings(resolvedUiLocale.value));
const sectionUi = computed(() => chromeSettingsSectionUi(resolvedUiLocale.value));
const navItems = computed(() => chromeSettingsNavItems(resolvedUiLocale.value));
const appVersion = APP_PACKAGE_VERSION;
const appBuildStack = APP_BUILD_STACK;
const filteredNav = computed(() => filterChromeSettingsNav(navItems.value, searchQuery.value));

const autoSave = useChromeSettingsAutoSave({
  config,
  savedLabel: () => ui.value.autoSaveSaved,
  onSaved: () => emit('saved'),
  onStatus: (msg) => emit('status', msg),
});

const { autoSaveState, beginHydration, endHydration, startAutoSaveWatchers, stopAutoSaveWatchers } =
  autoSave;

const autoSaveLabel = computed(() => {
  switch (autoSaveState.value) {
    case 'saving':
      return ui.value.autoSaveSaving;
    case 'saved':
      return ui.value.autoSaveSaved;
    case 'error':
      return ui.value.autoSaveError;
    default:
      return ui.value.autoSaveIdle;
  }
});

watch(
  () => props.initialSection,
  (section) => {
    const id = normalizeChromeSettingsSection(section ?? 'browser');
    markSectionMounted(id);
    activeSection.value = id;
  },
);

watch(
  () => props.p2pRoomId,
  (id) => {
    if (id) p2pRoomId.value = id;
  },
);

/** Keep active section visible when search filters the sidebar. */
watch(filteredNav, (items) => {
  if (items.length === 0) return;
  const stillVisible = items.some((item) => item.id === activeSection.value);
  if (!stillVisible) {
    selectSection(items[0].id, { syncUrl: false });
  }
});

function onEscapeKey(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return;
  emit('close');
}

onMounted(() => {
  beginHydration();
  startAutoSaveWatchers();
  void Promise.resolve(config.load()).finally(() => endHydration());
  document.addEventListener('keydown', onEscapeKey);
});

onUnmounted(() => {
  stopAutoSaveWatchers();
  document.removeEventListener('keydown', onEscapeKey);
});

function selectSection(id: ChromeSettingsSectionId, opts?: { syncUrl?: boolean }): void {
  markSectionMounted(id);
  activeSection.value = id;
  const el = mainScrollEl.value;
  if (el && typeof el.scrollTo === 'function') {
    el.scrollTo({ top: 0, behavior: 'smooth' });
  } else if (el) {
    el.scrollTop = 0;
  }
  if (opts?.syncUrl !== false) {
    emit('navigate', chromeSettingsUrlForSection(id));
  }
}

function onStatus(msg: string): void {
  emit('status', msg);
}

/** Inline nav icon component (avoids extra file). */
const ChromeSettingsNavIcon = defineComponent({
  name: 'ChromeSettingsNavIcon',
  props: { icon: { type: String, required: true } },
  setup(navIconProps) {
    return () => {
      const stroke = { fill: 'none', stroke: 'currentColor', 'stroke-width': '1.4' };
      const icons: Record<string, () => ReturnType<typeof h>> = {
        account: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 12, cy: 8, r: 4 }),
            h('path', { d: 'M4 20c2-4 6-6 8-6s6 2 8 6' }),
          ]),
        key: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 8, cy: 15, r: 4 }),
            h('path', { d: 'M11 15h9M16 12v6' }),
          ]),
        shield: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M12 3l8 3v6c0 5-3.5 8.5-8 9-4.5-.5-8-4-8-9V6l8-3z' }),
          ]),
        palette: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M12 3a9 9 0 1 0 0 18c-2 0-3-1-3-3 0-1 .5-2 2-2h2a4 4 0 0 0 0-8' }),
            h('circle', { cx: 8, cy: 10, r: 1, fill: 'currentColor' }),
            h('circle', { cx: 12, cy: 7, r: 1, fill: 'currentColor' }),
          ]),
        startup: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M12 3v6M8 7l4-4 4 4' }),
            h('rect', { x: 5, y: 11, width: 14, height: 10, rx: 2 }),
          ]),
        search: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 11, cy: 11, r: 6 }),
            h('path', { d: 'M16 16l5 5' }),
          ]),
        ai: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M8 12h8M12 8v8' }),
            h('rect', { x: 4, y: 4, width: 16, height: 16, rx: 3 }),
          ]),
        extensions: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M8 8a2 2 0 1 1-4 0 2 2 0 0 1 4 0zM20 12a2 2 0 1 1-4 0 2 2 0 0 1 4 0zM8 16a2 2 0 1 1-4 0 2 2 0 0 1 4 0z' }),
            h('path', { d: 'M10 9h6M8 13h2' }),
          ]),
        sidebar: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('rect', { x: 3, y: 4, width: 18, height: 16, rx: 2 }),
            h('path', { d: 'M9 4v16' }),
          ]),
        history: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 12, cy: 12, r: 9 }),
            h('path', { d: 'M12 7v5l3 2' }),
          ]),
        download: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M12 4v10M8 11l4 4 4-4' }),
            h('path', { d: 'M5 18h14' }),
          ]),
        p2p: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 6, cy: 12, r: 2 }),
            h('circle', { cx: 18, cy: 6, r: 2 }),
            h('circle', { cx: 18, cy: 18, r: 2 }),
            h('path', { d: 'M8 11l8-4M8 13l8 4' }),
          ]),
        reset: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('path', { d: 'M12 6V4M6 12a6 6 0 1 0 6-6' }),
          ]),
        info: () =>
          h('svg', { viewBox: '0 0 24 24', class: 'chrome-settings__nav-icon', ...stroke }, [
            h('circle', { cx: 12, cy: 12, r: 9 }),
            h('path', { d: 'M12 10v6M12 8h.01' }),
          ]),
      };
      const key = navIconProps.icon as ChromeSettingsNavIcon;
      const render = icons[key] ?? icons.info;
      return render();
    };
  },
});

/** Select sidebar section (all items use inline panels; panels opened via buttons). */
function selectSectionWithExternal(id: ChromeSettingsSectionId): void {
  selectSection(id);
}
</script>

<style scoped>
@import '@/styles/chrome-settings-fields.css';

.chrome-settings {
  flex: 1;
  min-height: 0;
  display: flex;
  background: #fff;
  color: var(--cs-label, #202124);
  overflow: hidden;
}

.chrome-settings__sidebar {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: #f1f3f4;
  border-right: 1px solid #dadce0;
  padding: 16px 0 12px;
}

.chrome-settings__brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 20px 16px;
}

.chrome-settings__logo {
  width: 24px;
  height: 24px;
  color: #5f6368;
}

.chrome-settings__brand-label {
  font-size: 22px;
  font-weight: 400;
  color: #202124;
}

.chrome-settings__search {
  padding: 0 16px 12px;
}

.chrome-settings__search-input {
  width: 100%;
  box-sizing: border-box;
  padding: 10px 12px;
  border-radius: 24px;
  border: none;
  background: #e8eaed;
  font-size: 13px;
  outline: none;
}

.chrome-settings__search-input:focus {
  background: #fff;
  box-shadow: 0 1px 3px rgba(60, 64, 67, 0.3);
}

.chrome-settings__nav {
  flex: 1;
  overflow-y: auto;
  padding: 4px 8px;
}

.chrome-settings__nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  border-radius: 0 24px 24px 0;
  background: transparent;
  color: #3c4043;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.chrome-settings__nav-item:hover {
  background: rgba(60, 64, 67, 0.08);
}

.chrome-settings__nav-item--active {
  background: #d2e3fc;
  color: #1967d2;
  font-weight: 500;
}

.chrome-settings__nav-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chrome-settings__nav-external {
  width: 14px;
  height: 14px;
  opacity: 0.6;
  flex-shrink: 0;
}

:deep(.chrome-settings__nav-icon) {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.chrome-settings__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: #fff;
}

.chrome-settings__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 32px 12px;
  border-bottom: 1px solid #e8eaed;
  flex-shrink: 0;
}

.chrome-settings__title {
  margin: 0;
  font-size: 22px;
  font-weight: 400;
}

.chrome-settings__subtitle {
  margin: 6px 0 0;
  font-size: 13px;
  color: #5f6368;
}

.chrome-settings__close {
  flex-shrink: 0;
  padding: 8px 20px;
  border-radius: 20px;
  border: 1px solid #dadce0;
  background: transparent;
  color: #1a73e8;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.chrome-settings__close:hover {
  background: rgba(26, 115, 232, 0.08);
}

.chrome-settings__content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 32px 24px;
}

.chrome-settings__footer {
  flex-shrink: 0;
  padding: 12px 32px 20px;
  border-top: 1px solid #e8eaed;
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.chrome-settings__autosave {
  font-size: 13px;
  color: #5f6368;
}

.chrome-settings__autosave--saving {
  color: #1a73e8;
}

.chrome-settings__autosave--saved {
  color: #188038;
}

.chrome-settings__autosave--error {
  color: #d93025;
}

.settings-section {
  max-width: 720px;
}

.settings-card {
  margin-bottom: 20px;
  padding: 16px 20px;
  border: 1px solid var(--cs-border, #e8eaed);
  border-radius: 8px;
  background: var(--cs-surface, #fff);
}

.settings-card__title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 500;
  color: var(--cs-label, #202124);
}

.settings-card label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 13px;
}

.settings-card label.checkbox-row {
  flex-direction: row;
  align-items: center;
  gap: 8px;
}

.settings-card input[type='text'],
.settings-card input[type='url'],
.settings-card input[type='number'],
.settings-card input[type='search'] {
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--cs-field-border, #dadce0);
  background: var(--cs-field-bg, #fff);
  color: var(--cs-field-text, #202124);
  font-size: 13px;
}

.settings-card input::placeholder {
  color: var(--cs-field-placeholder, #80868b);
}

.settings-hint {
  margin: 0 0 10px;
  font-size: 12px;
  color: var(--cs-hint, #5f6368);
  line-height: 1.45;
}

/* Unified child panels (P2P, extensions, privacy sub-panels, etc.) */
.chrome-settings__content :deep(.settings-section) {
  margin-bottom: 20px;
  padding: 16px 20px;
  border: 1px solid var(--cs-border, #e8eaed);
  border-radius: 8px;
  background: var(--cs-surface, #fff);
  max-width: 720px;
  box-sizing: border-box;
}

.chrome-settings__content :deep(.settings-section--stack) {
  padding: 0;
  border: none;
  background: transparent;
}

.chrome-settings__content :deep(.settings-section--stack > .settings-section) {
  margin-bottom: 20px;
}

/* Panels already wrapped in .settings-card — avoid double borders */
.chrome-settings__content :deep(.settings-card .settings-section) {
  margin: 0 0 16px;
  padding: 0;
  border: none;
  background: transparent;
  max-width: none;
}

.chrome-settings__content :deep(.settings-card .settings-section:last-child) {
  margin-bottom: 0;
}

.chrome-settings__content :deep(.settings-section h3),
.chrome-settings__content :deep(.settings-section h4),
.chrome-settings__content :deep(.settings-section .subsection-title) {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 500;
  color: var(--cs-title, #5f6368);
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.chrome-settings__content :deep(.settings-section h4),
.chrome-settings__content :deep(.settings-section .subsection-title) {
  margin-top: 16px;
  text-transform: none;
  letter-spacing: normal;
}

.chrome-settings__content :deep(.settings-section label:not(.checkbox-row)) {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
  font-size: 13px;
  color: var(--cs-label, #202124);
}

.chrome-settings__content :deep(.settings-section .checkbox-row) {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--cs-label, #202124);
  cursor: pointer;
}

.chrome-settings__content :deep(.settings-section input[type='text']),
.chrome-settings__content :deep(.settings-section input[type='url']),
.chrome-settings__content :deep(.settings-section input[type='number']),
.chrome-settings__content :deep(.settings-section input[type='search']),
.chrome-settings__content :deep(.settings-section input[type='password']),
.chrome-settings__content :deep(.settings-section input.field),
.chrome-settings__content :deep(.settings-section select),
.chrome-settings__content :deep(.settings-section textarea),
.chrome-settings__content :deep(.settings-section .field input),
.chrome-settings__content :deep(.settings-section .field select),
.chrome-settings__content :deep(.settings-section .field textarea),
.chrome-settings__content :deep(.settings-section .store-url-row input),
.chrome-settings__content :deep(.settings-section .download-input) {
  box-sizing: border-box;
  width: 100%;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--cs-field-border, #dadce0);
  background: var(--cs-field-bg, #fff);
  color: var(--cs-field-text, #202124);
  font-size: 13px;
}

.chrome-settings__content :deep(.settings-section input::placeholder),
.chrome-settings__content :deep(.settings-section textarea::placeholder) {
  color: var(--cs-field-placeholder, #80868b);
}

.chrome-settings__content :deep(.settings-section .toolbar input[type='text']),
.chrome-settings__content :deep(.settings-section .toolbar input.field) {
  width: auto;
  flex: 1;
  min-width: 120px;
}

.chrome-settings__content :deep(.settings-section .messages),
.chrome-settings__content :deep(.settings-section .announce),
.chrome-settings__content :deep(.settings-section .host-patterns-panel),
.chrome-settings__content :deep(.settings-section .extension-list) {
  border: 1px solid var(--cs-border, #e8eaed);
  border-radius: 8px;
  background: var(--cs-muted-surface, #f1f3f4);
  color: var(--cs-field-text, #202124);
}

.chrome-settings__content :deep(.settings-section .messages) {
  max-height: 160px;
  overflow-y: auto;
  margin: 8px 0;
  padding: 8px 12px;
}

.chrome-settings__content :deep(.settings-section .hint),
.chrome-settings__content :deep(.settings-section .muted),
.chrome-settings__content :deep(.settings-section .settings-hint) {
  font-size: 12px;
  color: var(--cs-hint, #5f6368);
}

.chrome-settings__content :deep(.settings-section .nav-button) {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  background: var(--cs-btn-primary-bg, #1a73e8);
  color: var(--cs-btn-primary-text, #fff);
}

.chrome-settings__content :deep(.settings-section .nav-button.secondary) {
  background: var(--cs-btn-secondary-bg, #f1f3f4);
  color: var(--cs-btn-secondary-text, #202124);
}

.chrome-settings__content :deep(.settings-section .nav-button:disabled) {
  opacity: 0.55;
  cursor: not-allowed;
}

.chrome-settings__content :deep(.settings-section .nav-button.danger) {
  background: #d93025;
  color: #fff;
}

.chrome-settings__content :deep(.settings-section .toolbar) {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
  margin-bottom: 8px;
}

.chrome-settings__content :deep(.settings-section .list) {
  list-style: none;
  padding: 0;
  margin: 0;
}

.settings-link-btn {
  padding: 8px 16px;
  border-radius: 20px;
  border: 1px solid #dadce0;
  background: #f8f9fa;
  font-size: 13px;
  cursor: pointer;
}

.settings-link-btn:hover {
  background: #e8eaed;
}

.settings-about-list {
  margin: 0;
  font-size: 13px;
}

.settings-about-list dt {
  color: #5f6368;
  margin-top: 8px;
}

.settings-about-list dd {
  margin: 2px 0 0;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  border: 0;
}

@media (prefers-color-scheme: dark) {
  .chrome-settings {
    background: #202124;
    color: var(--cs-label, #e8eaed);
  }

  .chrome-settings__sidebar {
    background: #292a2d;
    border-color: #5f6368;
  }

  .chrome-settings__brand-label,
  .chrome-settings__title {
    color: #e8eaed;
  }

  .chrome-settings__search-input {
    background: #3c4043;
    color: #e8eaed;
  }

  .chrome-settings__search-input:focus {
    background: #202124;
  }

  .chrome-settings__nav-item {
    color: #e8eaed;
  }

  .chrome-settings__nav-item--active {
    background: rgba(138, 180, 248, 0.24);
    color: #8ab4f8;
  }

  .chrome-settings__main {
    background: #202124;
  }

  .chrome-settings__header,
  .chrome-settings__footer {
    border-color: #5f6368;
  }

  .chrome-settings__subtitle,
  .settings-hint {
    color: #9aa0a6;
  }

  .chrome-settings__close {
    border-color: #5f6368;
    color: #8ab4f8;
  }

  .settings-card__title {
    color: var(--cs-label, #e8eaed);
  }

  .settings-link-btn {
    background: #3c4043;
    border-color: #5f6368;
    color: #e8eaed;
  }

  .chrome-settings__autosave {
    color: #9aa0a6;
  }

  .chrome-settings__autosave--saved {
    color: #81c995;
  }
}
</style>
