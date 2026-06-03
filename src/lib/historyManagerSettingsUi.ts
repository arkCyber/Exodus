/**
 * Exodus Browser — History manager settings UI strings.
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('historyManagerSettingsUi module loaded');

export interface HistoryManagerSettingsStrings {
  title: string;
  hint: string;
  enable: string;
  remember: string;
  retention: string;
  searchPlaceholder: string;
  search: string;
  refresh: string;
  remove: string;
  empty: string;
  clearAll: string;
  confirmClear: string;
}

const EN: HistoryManagerSettingsStrings = {
  title: 'Browsing history',
  hint: 'Managed history store (separate from sidebar visit list).',
  enable: 'Enable history manager',
  remember: 'Remember browsing',
  retention: 'Retention (days, 0 = forever)',
  searchPlaceholder: 'Search URL or title…',
  search: 'Search',
  refresh: 'Refresh',
  remove: 'Remove',
  empty: 'No history entries.',
  clearAll: 'Clear full history',
  confirmClear: 'Click again to confirm',
};

const ZH: HistoryManagerSettingsStrings = {
  title: '浏览历史记录',
  hint: '托管的历史记录存储（与侧边栏访问列表分开）。',
  enable: '启用历史记录管理器',
  remember: '记住浏览记录',
  retention: '保留天数（0 = 永久）',
  searchPlaceholder: '搜索 URL 或标题…',
  search: '搜索',
  refresh: '刷新',
  remove: '移除',
  empty: '没有历史记录条目。',
  clearAll: '清除全部历史记录',
  confirmClear: '再次点击以确认',
};

const PACKS: Record<AppLocale, HistoryManagerSettingsStrings> = {
  en: EN,
  zh: ZH,
  ja: EN,
  ko: EN,
  es: EN,
  fr: EN,
  de: EN,
  pt: EN,
  ru: EN,
};

/** Localized history manager settings copy. */
export function historyManagerSettingsStrings(locale?: AppLocale): HistoryManagerSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
