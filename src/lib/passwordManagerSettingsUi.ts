/**
 * Exodus Browser — Password manager settings UI strings.
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('passwordManagerSettingsUi module loaded');

export interface PasswordManagerSettingsStrings {
  title: string;
  searchPlaceholder: string;
  add: string;
  generate: string;
  empty: string;
  copy: string;
  delete: string;
  addTitle: string;
  site: string;
  url: string;
  username: string;
  password: string;
  cancel: string;
  save: string;
  genTitle: string;
  genLength: string;
  genSymbols: string;
  genButton: string;
}

const EN: PasswordManagerSettingsStrings = {
  title: 'Password manager',
  searchPlaceholder: 'Search passwords…',
  add: 'Add',
  generate: 'Generate',
  empty: 'No saved passwords.',
  copy: 'Copy',
  delete: 'Delete',
  addTitle: 'Add password',
  site: 'Site',
  url: 'URL',
  username: 'Username',
  password: 'Password',
  cancel: 'Cancel',
  save: 'Save',
  genTitle: 'Generate password',
  genLength: 'Length',
  genSymbols: 'Include symbols',
  genButton: 'Generate',
};

const ZH: PasswordManagerSettingsStrings = {
  title: '密码管理器',
  searchPlaceholder: '搜索密码…',
  add: '添加',
  generate: '生成',
  empty: '没有已保存的密码。',
  copy: '复制',
  delete: '删除',
  addTitle: '添加密码',
  site: '网站',
  url: 'URL',
  username: '用户名',
  password: '密码',
  cancel: '取消',
  save: '保存',
  genTitle: '生成密码',
  genLength: '长度',
  genSymbols: '包含符号',
  genButton: '生成',
};

const PACKS: Record<AppLocale, PasswordManagerSettingsStrings> = {
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

/** Localized password manager settings copy. */
export function passwordManagerSettingsStrings(locale?: AppLocale): PasswordManagerSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
