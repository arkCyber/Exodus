/**
 * Exodus Browser — Clear browsing data settings UI strings (Chrome privacy parity).
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('clearBrowsingDataSettingsUi module loaded');

export interface ClearBrowsingDataSettingsStrings {
  title: string;
  hint: string;
  cookies: string;
  history: string;
  localStorage: string;
  cache: string;
  clearButton: string;
  clearing: string;
  cleared: (items: string) => string;
  clearError: string;
  nothingSelected: string;
  loading: string;
}

const EN: ClearBrowsingDataSettingsStrings = {
  title: 'Clear browsing data',
  hint: 'Remove stored data from this device. This cannot be undone.',
  cookies: 'Cookies and other site data',
  history: 'Browsing history',
  localStorage: 'Cached site data (local storage index)',
  cache: 'Cached images and files (noted only in Tauri builds)',
  clearButton: 'Clear data',
  clearing: 'Clearing…',
  cleared: (items) => `Cleared: ${items}`,
  clearError: 'Failed to clear browsing data',
  nothingSelected: 'Select at least one data type to clear',
  loading: 'Loading...',
};

const ZH: ClearBrowsingDataSettingsStrings = {
  title: '清除浏览数据',
  hint: '从本设备删除所选数据，此操作无法撤销。',
  cookies: 'Cookie 及其他网站数据',
  history: '浏览历史记录',
  localStorage: '缓存的网站数据（本地存储索引）',
  cache: '缓存的图片和文件（Tauri 构建中仅作记录）',
  clearButton: '清除数据',
  clearing: '正在清除…',
  cleared: (items) => `已清除：${items}`,
  clearError: '清除浏览数据失败',
  nothingSelected: '请至少选择一种要清除的数据类型',
  loading: '加载中...',
};

const PACKS: Record<AppLocale, ClearBrowsingDataSettingsStrings> = {
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

/** Localized clear-browsing-data panel copy. */
export function clearBrowsingDataSettingsStrings(locale?: AppLocale): ClearBrowsingDataSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
