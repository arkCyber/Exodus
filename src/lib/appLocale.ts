/**
 * Exodus Browser — application UI locale (worldwide language support).
 */

import { logStartup } from '@/lib/startupLog';

logStartup('appLocale module loaded');

/** Supported UI locales for shell, settings, and bookmark bar. */
export const APP_LOCALES = ['en', 'zh', 'ja', 'ko', 'es', 'fr', 'de', 'pt', 'ru'] as const;

export type AppLocale = (typeof APP_LOCALES)[number];

export const APP_LOCALE_STORAGE_KEY = 'exodus-ui-locale';

/** Human-readable locale names (shown in the language picker). */
export const LOCALE_DISPLAY_NAMES: Record<AppLocale, string> = {
  en: 'English',
  zh: '中文',
  ja: '日本語',
  ko: '한국어',
  es: 'Español',
  fr: 'Français',
  de: 'Deutsch',
  pt: 'Português',
  ru: 'Русский',
};

/** Returns true when `value` is a supported AppLocale. */
export function isAppLocale(value: string): value is AppLocale {
  return (APP_LOCALES as readonly string[]).includes(value);
}

/**
 * Resolve UI locale: explicit override, saved preference, then browser language, else English.
 */
export function resolveAppLocale(explicit?: AppLocale | string | null): AppLocale {
  if (explicit && isAppLocale(explicit)) return explicit;
  if (typeof localStorage !== 'undefined') {
    try {
      const saved = localStorage.getItem(APP_LOCALE_STORAGE_KEY);
      if (saved && isAppLocale(saved)) return saved;
    } catch (error) {
      console.error('resolveAppLocale read failed:', error);
    }
  }
  if (typeof navigator === 'undefined') return 'en';
  const lang = navigator.language.toLowerCase();
  if (lang.startsWith('zh')) return 'zh';
  if (lang.startsWith('ja')) return 'ja';
  if (lang.startsWith('ko')) return 'ko';
  if (lang.startsWith('es')) return 'es';
  if (lang.startsWith('fr')) return 'fr';
  if (lang.startsWith('de')) return 'de';
  if (lang.startsWith('pt')) return 'pt';
  if (lang.startsWith('ru')) return 'ru';
  return 'en';
}

/** Persist UI locale to localStorage. */
export function writeAppLocale(locale: AppLocale): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(APP_LOCALE_STORAGE_KEY, locale);
  } catch (error) {
    console.error('writeAppLocale failed:', error);
  }
}

/** Read persisted locale without browser fallback. */
export function readAppLocale(): AppLocale | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const saved = localStorage.getItem(APP_LOCALE_STORAGE_KEY);
    return saved && isAppLocale(saved) ? saved : null;
  } catch (error) {
    console.error('readAppLocale failed:', error);
    return null;
  }
}

/** Options for locale &lt;select&gt; in settings. */
export function appLocaleOptions(): { value: AppLocale; label: string }[] {
  return APP_LOCALES.map((value) => ({ value, label: LOCALE_DISPLAY_NAMES[value] }));
}
