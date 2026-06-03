/**
 * Exodus Browser — Extensions settings UI strings (Chrome://extensions parity).
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('extensionsSettingsUi module loaded');

export interface ExtensionsSettingsStrings {
  pageTitle: string;
  pageHint: string;
  openApps: string;
  preferencesTitle: string;
  confirmHostLabel: string;
  storeUrlLabel: string;
  storeUrlPlaceholder: string;
  saveStoreUrl: string;
  actionsTitle: string;
  refresh: string;
  rescan: string;
  installFolder: string;
  installCrx: string;
  storeTitle: string;
  installedTitle: string;
  searchPlaceholder: string;
  loading: string;
  emptyInstalled: string;
  enabled: string;
  pinToolbar: string;
  siteAccess: string;
  hideSiteAccess: string;
  uninstall: string;
  installedBadge: string;
  install: string;
  permissions: (list: string) => string;
  popup: (path: string) => string;
  loadingSites: string;
  noSites: string;
  revokeAll: string;
  revoke: string;
  statusRescanned: (count: number) => string;
  statusStoreSaved: string;
  statusConfirmOn: string;
  statusConfirmOff: string;
}

const EN: ExtensionsSettingsStrings = {
  pageTitle: 'Extensions',
  pageHint: 'Manage Manifest V3 extensions, site access, and the optional dev catalog.',
  openApps: 'Open apps & shortcuts',
  preferencesTitle: 'Extension preferences',
  confirmHostLabel: 'Ask before granting extension site access on install',
  storeUrlLabel: 'Remote extension catalog URL (JSON)',
  storeUrlPlaceholder: 'https://example.com/extensions/catalog.json',
  saveStoreUrl: 'Save catalog URL',
  actionsTitle: 'Manage extensions',
  refresh: 'Refresh list',
  rescan: 'Rescan folders',
  installFolder: 'Load unpacked…',
  installCrx: 'Packaged extension (.crx / .zip)…',
  storeTitle: 'Available from catalog',
  installedTitle: 'Installed extensions',
  searchPlaceholder: 'Search extensions',
  loading: 'Loading extensions…',
  emptyInstalled: 'No extensions installed yet. Load an unpacked folder or install from the catalog.',
  enabled: 'Enabled',
  pinToolbar: 'Pin to toolbar',
  siteAccess: 'Site access',
  hideSiteAccess: 'Hide site access',
  uninstall: 'Remove',
  installedBadge: 'Installed',
  install: 'Add',
  permissions: (list) => `Permissions: ${list || 'none'}`,
  popup: (path) => `Popup: ${path}`,
  loadingSites: 'Loading granted sites…',
  noSites: 'No granted site patterns yet.',
  revokeAll: 'Revoke all sites',
  revoke: 'Revoke',
  statusRescanned: (n) => `Rescanned ${n} extension(s)`,
  statusStoreSaved: 'Extension catalog URL saved',
  statusConfirmOn: 'Install will ask before granting site access',
  statusConfirmOff: 'Install will auto-grant manifest site access',
};

const ZH: ExtensionsSettingsStrings = {
  pageTitle: '扩展程序',
  pageHint: '管理 Manifest V3 扩展、网站访问权限与可选的开发目录。',
  openApps: '打开应用与快捷方式',
  preferencesTitle: '扩展程序偏好设置',
  confirmHostLabel: '安装扩展时，在授予网站访问权限前先询问',
  storeUrlLabel: '远程扩展目录 URL（JSON）',
  storeUrlPlaceholder: 'https://example.com/extensions/catalog.json',
  saveStoreUrl: '保存目录地址',
  actionsTitle: '管理扩展程序',
  refresh: '刷新列表',
  rescan: '重新扫描文件夹',
  installFolder: '加载已解压的扩展…',
  installCrx: '打包扩展（.crx / .zip）…',
  storeTitle: '目录中的可用扩展',
  installedTitle: '已安装的扩展程序',
  searchPlaceholder: '搜索扩展程序',
  loading: '正在加载扩展程序…',
  emptyInstalled: '尚未安装扩展程序。可加载已解压文件夹或从目录安装。',
  enabled: '已启用',
  pinToolbar: '固定到工具栏',
  siteAccess: '网站访问权限',
  hideSiteAccess: '隐藏网站访问权限',
  uninstall: '移除',
  installedBadge: '已安装',
  install: '添加',
  permissions: (list) => `权限：${list || '无'}`,
  popup: (path) => `弹出窗口：${path}`,
  loadingSites: '正在加载已授权网站…',
  noSites: '尚无已授权的网站模式。',
  revokeAll: '撤销全部网站',
  revoke: '撤销',
  statusRescanned: (n) => `已重新扫描 ${n} 个扩展`,
  statusStoreSaved: '扩展目录 URL 已保存',
  statusConfirmOn: '安装前将询问网站访问权限',
  statusConfirmOff: '安装时将自动授予清单中的网站访问权限',
};

const PACKS: Record<AppLocale, ExtensionsSettingsStrings> = {
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

/** Localized extensions settings copy. */
export function extensionsSettingsStrings(locale?: AppLocale): ExtensionsSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
