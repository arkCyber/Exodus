/**
 * Exodus Browser — Appearance settings strings (theme + language).
 */

import { logStartup } from '@/lib/startupLog';
import type { AppLocale } from '@/lib/appLocale';
import { resolveAppLocale } from '@/lib/appLocale';
import type { Theme } from '@/composables/useTheme';

logStartup('appearanceSettingsUi module loaded');

export interface AppearanceSettingsStrings {
  sectionTitle: string;
  sectionHint: string;
  themeLabel: string;
  themeLight: string;
  themeDark: string;
  themeAuto: string;
  languageLabel: string;
  languageHint: string;
  themeOptionLabel: (mode: Theme) => string;
  loading: string;
}

type StringPack = AppearanceSettingsStrings;

const EN: StringPack = {
  sectionTitle: 'Theme and language',
  sectionHint: 'Applies to the browser shell and settings. Some extension pages may follow the system.',
  themeLabel: 'Theme',
  themeLight: 'Light',
  themeDark: 'Dark',
  themeAuto: 'Automatic (system)',
  languageLabel: 'Language',
  languageHint: 'UI labels for the bookmark bar, menus, and settings.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return EN.themeLight;
    if (mode === 'dark') return EN.themeDark;
    return EN.themeAuto;
  },
  loading: 'Loading...',
};

const ZH: StringPack = {
  sectionTitle: '主题与语言',
  sectionHint: '应用于浏览器外壳与设置。部分扩展页面可能仍跟随系统。',
  themeLabel: '主题',
  themeLight: '浅色',
  themeDark: '深色',
  themeAuto: '自动（跟随系统）',
  languageLabel: '语言',
  languageHint: '书签栏、菜单与设置界面的显示语言。',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return ZH.themeLight;
    if (mode === 'dark') return ZH.themeDark;
    return ZH.themeAuto;
  },
  loading: '加载中...',
};

const JA: StringPack = {
  sectionTitle: 'テーマと言語',
  sectionHint: 'ブラウザの UI と設定に適用されます。',
  themeLabel: 'テーマ',
  themeLight: 'ライト',
  themeDark: 'ダーク',
  themeAuto: '自動（システム）',
  languageLabel: '言語',
  languageHint: 'ブックマークバー、メニュー、設定の表示言語。',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return JA.themeLight;
    if (mode === 'dark') return JA.themeDark;
    return JA.themeAuto;
  },
  loading: '読み込み中...',
};

const KO: StringPack = {
  sectionTitle: '테마 및 언어',
  sectionHint: '브라우저 셸과 설정에 적용됩니다.',
  themeLabel: '테마',
  themeLight: '라이트',
  themeDark: '다크',
  themeAuto: '자동(시스템)',
  languageLabel: '언어',
  languageHint: '북마크 바, 메뉴, 설정 UI 표시 언어.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return KO.themeLight;
    if (mode === 'dark') return KO.themeDark;
    return KO.themeAuto;
  },
  loading: '로딩 중...',
};

const ES: StringPack = {
  sectionTitle: 'Tema e idioma',
  sectionHint: 'Se aplica al shell del navegador y a la configuración.',
  themeLabel: 'Tema',
  themeLight: 'Claro',
  themeDark: 'Oscuro',
  themeAuto: 'Automático (sistema)',
  languageLabel: 'Idioma',
  languageHint: 'Etiquetas de la barra de marcadores, menús y ajustes.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return ES.themeLight;
    if (mode === 'dark') return ES.themeDark;
    return ES.themeAuto;
  },
  loading: 'Cargando...',
};

const FR: StringPack = {
  sectionTitle: 'Thème et langue',
  sectionHint: 'S’applique au shell du navigateur et aux paramètres.',
  themeLabel: 'Thème',
  themeLight: 'Clair',
  themeDark: 'Sombre',
  themeAuto: 'Automatique (système)',
  languageLabel: 'Langue',
  languageHint: 'Libellés de la barre de favoris, menus et paramètres.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return FR.themeLight;
    if (mode === 'dark') return FR.themeDark;
    return FR.themeAuto;
  },
  loading: 'Chargement...',
};

const DE: StringPack = {
  sectionTitle: 'Design und Sprache',
  sectionHint: 'Gilt für Browser-Oberfläche und Einstellungen.',
  themeLabel: 'Design',
  themeLight: 'Hell',
  themeDark: 'Dunkel',
  themeAuto: 'Automatisch (System)',
  languageLabel: 'Sprache',
  languageHint: 'UI-Sprache für Lesezeichenleiste, Menüs und Einstellungen.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return DE.themeLight;
    if (mode === 'dark') return DE.themeDark;
    return DE.themeAuto;
  },
  loading: 'Wird geladen...',
};

const PT: StringPack = {
  sectionTitle: 'Tema e idioma',
  sectionHint: 'Aplica-se ao shell do navegador e às configurações.',
  themeLabel: 'Tema',
  themeLight: 'Claro',
  themeDark: 'Escuro',
  themeAuto: 'Automático (sistema)',
  languageLabel: 'Idioma',
  languageHint: 'Idioma da barra de favoritos, menus e configurações.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return PT.themeLight;
    if (mode === 'dark') return PT.themeDark;
    return PT.themeAuto;
  },
  loading: 'Carregando...',
};

const RU: StringPack = {
  sectionTitle: 'Тема и язык',
  sectionHint: 'Применяется к оболочке браузера и настройкам.',
  themeLabel: 'Тема',
  themeLight: 'Светлая',
  themeDark: 'Тёмная',
  themeAuto: 'Авто (система)',
  languageLabel: 'Язык',
  languageHint: 'Язык панели закладок, меню и настроек.',
  themeOptionLabel: (mode) => {
    if (mode === 'light') return RU.themeLight;
    if (mode === 'dark') return RU.themeDark;
    return RU.themeAuto;
  },
  loading: 'Загрузка...',
};

const PACKS: Record<AppLocale, StringPack> = {
  en: EN,
  zh: ZH,
  ja: JA,
  ko: KO,
  es: ES,
  fr: FR,
  de: DE,
  pt: PT,
  ru: RU,
};

/** Localized appearance settings copy. */
export function appearanceSettingsStrings(locale?: AppLocale): AppearanceSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
