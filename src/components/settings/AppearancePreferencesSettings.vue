<!--
  Exodus Browser — theme (light/dark/auto) and UI language (settings → Appearance).
-->
<template>
  <section
    class="settings-section appearance-preferences"
    data-testid="appearance-preferences-settings"
  >
    <h3>{{ ui.sectionTitle }}</h3>
    <p class="settings-hint">{{ ui.sectionHint }}</p>

    <div v-if="loading" class="loading-state">{{ ui.loading }}</div>
    <template v-else>
      <label>
        {{ ui.themeLabel }}
        <select v-model="theme" data-testid="appearance-theme-select" @change="onThemeChanged">
          <option v-for="mode in themeModes" :key="mode" :value="mode">
            {{ ui.themeOptionLabel(mode) }}
          </option>
        </select>
      </label>

      <label>
        {{ ui.languageLabel }}
        <select v-model="localeModel" data-testid="appearance-locale-select">
          <option v-for="opt in localeOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </label>
      <p class="settings-hint">{{ ui.languageHint }}</p>
    </template>
  </section>
</template>

<script setup lang="ts">
/**
 * Exodus Browser — appearance preferences (theme + locale), persisted immediately.
 */
import { computed, ref, onMounted } from 'vue';
import { useTheme, type Theme } from '@/composables/useTheme';
import {
  appLocaleOptions,
  LOCALE_DISPLAY_NAMES,
  resolveAppLocale,
  writeAppLocale,
  type AppLocale,
} from '@/lib/appLocale';
import { appearanceSettingsStrings } from '@/lib/appearanceSettingsUi';

const props = defineProps<{
  /** Current UI locale (from shell). */
  uiLocale?: AppLocale;
}>();

const emit = defineEmits<{
  status: [message: string];
  localeChange: [locale: AppLocale];
}>();

const { theme, setTheme } = useTheme();
const themeModes: Theme[] = ['light', 'dark', 'auto'];
const localeOptions = appLocaleOptions();
const loading = ref(true);

const ui = computed(() => appearanceSettingsStrings(resolveAppLocale(props.uiLocale)));

const localeModel = computed({
  get: () => resolveAppLocale(props.uiLocale),
  set: (next: AppLocale) => {
    writeAppLocale(next);
    emit('localeChange', next);
    emit('status', LOCALE_DISPLAY_NAMES[next]);
  },
});

function onThemeChanged(): void {
  setTheme(theme.value);
  emit('status', ui.value.themeOptionLabel(theme.value));
}

onMounted(() => {
  loading.value = false;
});
</script>
