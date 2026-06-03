/**
 * Exodus Browser — Downloads settings panel UI strings.
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('downloadsSettingsUi module loaded');

export interface DownloadsSettingsStrings {
  title: string;
  defaultDirLabel: string;
  defaultDirPlaceholder: string;
  askLocation: string;
  showNotifications: string;
  clearOnExit: string;
  maxConcurrentLabel: string;
  save: string;
  loading: string;
  loadError: string;
  saved: string;
  saveError: string;
}

const EN: DownloadsSettingsStrings = {
  title: 'Download location',
  defaultDirLabel: 'Default download folder',
  defaultDirPlaceholder: '~/Downloads',
  askLocation: 'Ask where to save each file before downloading',
  showNotifications: 'Show download notifications',
  clearOnExit: 'Clear completed downloads when Exodus exits',
  maxConcurrentLabel: 'Maximum concurrent downloads',
  save: 'Save download settings',
  loading: 'Loading download settings…',
  loadError: 'Could not load download settings',
  saved: 'Download settings saved',
  saveError: 'Failed to save download settings',
};

const ZH: DownloadsSettingsStrings = {
  title: '下载位置',
  defaultDirLabel: '默认下载文件夹',
  defaultDirPlaceholder: '~/Downloads',
  askLocation: '下载前询问每个文件的保存位置',
  showNotifications: '显示下载通知',
  clearOnExit: '退出 Exodus 时清除已完成的下载记录',
  maxConcurrentLabel: '最大同时下载数',
  save: '保存下载设置',
  loading: '正在加载下载设置…',
  loadError: '无法加载下载设置',
  saved: '下载设置已保存',
  saveError: '保存下载设置失败',
};

const PACKS: Record<AppLocale, DownloadsSettingsStrings> = {
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

/** Localized downloads settings copy. */
export function downloadsSettingsStrings(locale?: AppLocale): DownloadsSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
