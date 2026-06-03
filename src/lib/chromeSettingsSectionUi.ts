/**
 * Exodus Browser — localized copy for inline Chrome settings section bodies.
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('chromeSettingsSectionUi module loaded');

export interface ChromeSettingsSectionUi {
  browser: {
    generalTitle: string;
    homepageLabel: string;
    homepagePlaceholder: string;
    searchLabel: string;
    searchPlaceholder: string;
    bookmarkBar: string;
  };
  privacy: {
    controlsTitle: string;
    httpsOnly: string;
    privateMode: string;
    blockPopups: string;
  };
  startup: {
    title: string;
    restoreTabs: string;
    hint: string;
  };
  sidebar: {
    title: string;
    hint: string;
    customize: string;
  };
  history: {
    openSidebar: string;
  };
  downloads: {
    hint: string;
    openPanel: string;
  };
  reset: {
    title: string;
    hint: string;
    button: string;
  };
  about: {
    productTitle: string;
    tagline: string;
    versionLabel: string;
    buildLabel: string;
    settingsUrlLabel: string;
  };
}

const EN: ChromeSettingsSectionUi = {
  browser: {
    generalTitle: 'General',
    homepageLabel: 'Homepage URL',
    homepagePlaceholder: 'https://duckduckgo.com',
    searchLabel: "Search engine (use {'{query}'} for terms)",
    searchPlaceholder: 'https://duckduckgo.com/?q={query}',
    bookmarkBar: 'Show bookmark bar',
  },
  privacy: {
    controlsTitle: 'Privacy controls',
    httpsOnly: 'HTTPS-only mode',
    privateMode: 'Private mode',
    blockPopups: 'Block popups',
  },
  startup: {
    title: 'When Exodus starts',
    restoreTabs: 'Restore tabs from last session',
    hint: 'Uses the homepage above when opening a new window without restored tabs.',
  },
  sidebar: {
    title: 'Sidebar layout',
    hint: 'Position, vertical tabs, and which tools appear in the icon rail.',
    customize: 'Customize sidebar…',
  },
  history: {
    openSidebar: 'Open history in sidebar',
  },
  downloads: {
    hint: 'Manage download location and behavior. Completed files appear in the downloads panel.',
    openPanel: 'Open downloads panel',
  },
  reset: {
    title: 'Restore defaults',
    hint: 'Reset new tab page layout and quick links to factory defaults.',
    button: 'Reset new tab layout…',
  },
  about: {
    productTitle: 'Exodus Browser',
    tagline: 'Privacy-focused browser with local AI, extensions, and Chrome-aligned UI.',
    versionLabel: 'Version',
    buildLabel: 'Build',
    settingsUrlLabel: 'Settings URL',
  },
};

const ZH: ChromeSettingsSectionUi = {
  browser: {
    generalTitle: '常规',
    homepageLabel: '主页 URL',
    homepagePlaceholder: 'https://duckduckgo.com',
    searchLabel: '搜索引擎（用 {query} 表示搜索词）',
    searchPlaceholder: 'https://duckduckgo.com/?q={query}',
    bookmarkBar: '显示书签栏',
  },
  privacy: {
    controlsTitle: '隐私控制',
    httpsOnly: '仅 HTTPS 模式',
    privateMode: '隐私模式',
    blockPopups: '阻止弹出窗口',
  },
  startup: {
    title: '启动 Exodus 时',
    restoreTabs: '恢复上次会话的标签页',
    hint: '在未恢复标签页时，新窗口将使用上方设置的主页。',
  },
  sidebar: {
    title: '侧边栏布局',
    hint: '位置、垂直标签页以及图标栏中显示的工具。',
    customize: '自定义侧边栏…',
  },
  history: {
    openSidebar: '在侧边栏中打开历史记录',
  },
  downloads: {
    hint: '管理下载位置与行为。已完成的文件会显示在下载面板中。',
    openPanel: '打开下载面板',
  },
  reset: {
    title: '恢复默认设置',
    hint: '将新标签页布局与快捷链接恢复为出厂默认值。',
    button: '重置新标签页布局…',
  },
  about: {
    productTitle: 'Exodus 浏览器',
    tagline: '注重隐私的浏览器，支持本地 AI、扩展程序与 Chrome 风格界面。',
    versionLabel: '版本',
    buildLabel: '构建',
    settingsUrlLabel: '设置地址',
  },
};

const PACKS: Record<AppLocale, ChromeSettingsSectionUi> = {
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

/** Localized strings for inline settings section bodies. */
export function chromeSettingsSectionUi(locale?: AppLocale): ChromeSettingsSectionUi {
  return PACKS[resolveAppLocale(locale)];
}
