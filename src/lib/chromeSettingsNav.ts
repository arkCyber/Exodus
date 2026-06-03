/**
 * Exodus Browser — Chrome-style settings navigation (full-page settings UI).
 */

import { logStartup } from '@/lib/startupLog';
import { resolveAppLocale, type AppLocale } from '@/lib/appLocale';

logStartup('chromeSettingsNav module loaded');

/** @deprecated Use AppLocale — settings UI locale id. */
export type ChromeSettingsLocale = AppLocale;

/** Settings sidebar section identifiers. */
export type ChromeSettingsSectionId =
  | 'browser'
  | 'autofill'
  | 'privacy'
  | 'appearance'
  | 'startup'
  | 'ai'
  | 'extensions'
  | 'sidebar'
  | 'history'
  | 'downloads'
  | 'p2p'
  | 'plugins'
  | 'reset'
  | 'about'
  | 'search'
  | 'fonts'
  | 'network'
  | 'profile'
  | 'sync'
  | 'media'
  | 'notifications'
  | 'accessibility'
  | 'shortcuts'
  | 'system'
  | 'performance'
  | 'gpu';

const VALID_SECTIONS = new Set<ChromeSettingsSectionId>([
  'browser',
  'autofill',
  'privacy',
  'appearance',
  'startup',
  'ai',
  'extensions',
  'sidebar',
  'history',
  'downloads',
  'p2p',
  'plugins',
  'reset',
  'about',
  'search',
  'fonts',
  'network',
  'media',
  'notifications',
  'accessibility',
  'shortcuts',
  'system',
  'sync',
  'profile',
  'performance',
  'gpu',
]);

/** Alias map for chrome://settings URLs and legacy scroll targets. */
const SECTION_ALIASES: Record<string, ChromeSettingsSectionId> = {
  browser: 'browser',
  general: 'browser',
  autofill: 'autofill',
  passwords: 'autofill',
  privacy: 'privacy',
  security: 'privacy',
  appearance: 'appearance',
  themes: 'appearance',
  startup: 'startup',
  'on-startup': 'startup',
  ai: 'ai',
  inference: 'ai',
  allama: 'ai',
  extensions: 'extensions',
  sidebar: 'sidebar',
  tabs: 'sidebar',
  history: 'history',
  downloads: 'downloads',
  p2p: 'p2p',
  reset: 'reset',
  about: 'about',
  sync: 'sync',
  profile: 'profile',
  profiles: 'profile',
  performance: 'performance',
  gpu: 'gpu',
};

export type ChromeSettingsNavItem = {
  id: ChromeSettingsSectionId;
  label: string;
  icon: ChromeSettingsNavIcon;
  group: 'primary' | 'advanced' | 'footer';
  /** Opens external panel instead of inline panel only. */
  external?: boolean;
};

export type ChromeSettingsNavIcon =
  | 'account'
  | 'key'
  | 'shield'
  | 'palette'
  | 'startup'
  | 'search'
  | 'ai'
  | 'extensions'
  | 'sidebar'
  | 'history'
  | 'download'
  | 'p2p'
  | 'plugins'
  | 'reset'
  | 'info'
  | 'font'
  | 'network'
  | 'media'
  | 'bell'
  | 'accessibility'
  | 'keyboard'
  | 'system'
  | 'sync'
  | 'speed'
  | 'gpu';

export interface ChromeSettingsStrings {
  pageTitle: string;
  close: string;
  autoSaveIdle: string;
  autoSaveSaving: string;
  autoSaveSaved: string;
  autoSaveError: string;
  searchPlaceholder: string;
  primaryGroup: string;
  advancedGroup: string;
  footerGroup: string;
  sectionTitle: (id: ChromeSettingsSectionId) => string;
  sectionDescription: (id: ChromeSettingsSectionId) => string;
  nav: (id: ChromeSettingsSectionId) => string;
}

const EN_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Browser',
  autofill: 'Autofill and passwords',
  privacy: 'Privacy and security',
  appearance: 'Appearance',
  startup: 'On startup',
  ai: 'AI and inference',
  extensions: 'Extensions',
  sidebar: 'Sidebar',
  history: 'History',
  downloads: 'Downloads',
  p2p: 'P2P and chat',
  plugins: 'Plugins',
  reset: 'Reset settings',
  about: 'About Exodus',
  search: 'Search engine',
  fonts: 'Fonts and zoom',
  network: 'Network',
  media: 'Media',
  notifications: 'Notifications',
  accessibility: 'Accessibility',
  shortcuts: 'Keyboard shortcuts',
  system: 'System',
  sync: 'Sync',
  profile: 'Profile',
  performance: 'Performance',
  gpu: 'GPU & Hardware',
};

const ZH_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: '浏览器',
  autofill: '自动填充和密码',
  privacy: '隐私与安全',
  appearance: '外观',
  startup: '起始页面',
  ai: 'AI 与推理',
  extensions: '扩展程序',
  sidebar: '侧边栏',
  history: '历史记录',
  downloads: '下载内容',
  p2p: 'P2P 与聊天',
  plugins: '插件',
  reset: '重置设置',
  about: '关于 Exodus',
  search: '搜索引擎',
  fonts: '字体和缩放',
  network: '网络',
  media: '媒体',
  notifications: '通知',
  accessibility: '无障碍',
  shortcuts: '键盘快捷键',
  system: '系统',
  sync: '同步',
  profile: '配置文件',
  performance: '性能',
  gpu: 'GPU 与硬件',
};

const EN_DESC: Partial<Record<ChromeSettingsSectionId, string>> = {
  browser: 'Homepage, search, and bookmark bar.',
  autofill: 'Saved passwords and autofill data.',
  privacy: 'Tracking protection, HTTPS, and site permissions.',
  appearance: 'New tab page, wallpapers, and layout.',
  startup: 'Session restore and startup behavior.',
  ai: 'Local AI service and model selection.',
  extensions: 'Enable, install, and manage extensions; site access and toolbar pins.',
  sidebar: 'Firefox-style sidebar and vertical tabs.',
  history: 'Browsing history storage and clearing.',
  downloads: 'Download location and history.',
  p2p: 'Peer-to-peer CDN and group chat rooms.',
  plugins: 'Native plugin management, sandbox configuration, and resource monitoring.',
  reset: 'Restore defaults for selected features.',
  about: 'Version and build information.',
  search: 'Manage default and custom search engines.',
  fonts: 'Customize font size, family, and page zoom.',
  network: 'Configure proxy, DNS, and connection settings.',
  media: 'Autoplay, picture-in-picture, and hardware acceleration.',
  notifications: 'Website notification permissions and display.',
  accessibility: 'Screen reader, high contrast, and other accessibility features.',
  shortcuts: 'View and customize keyboard shortcuts.',
  system: 'Default browser, background apps, and system updates.',
  sync: 'Synchronize bookmarks and settings across devices.',
  gpu: 'GPU acceleration, WebGL, WebGPU, and hardware rendering.',
};

const ZH_DESC: Partial<Record<ChromeSettingsSectionId, string>> = {
  browser: '主页、搜索引擎与书签栏。',
  autofill: '已保存的密码与自动填充。',
  privacy: '跟踪保护、HTTPS 与网站权限。',
  appearance: '新标签页、壁纸与布局。',
  startup: '启动时恢复标签页等行为。',
  ai: '本地 AI 服务与模型选择。',
  extensions: '启用、安装与管理扩展程序；网站访问与工具栏固定。',
  sidebar: 'Firefox 风格侧边栏与垂直标签。',
  history: '浏览历史记录与清理。',
  downloads: '下载位置与记录。',
  p2p: '点对点 CDN 与群聊房间。',
  plugins: '原生插件管理、沙箱配置与资源监控。',
  reset: '将部分功能恢复为默认设置。',
  about: '版本与构建信息。',
  search: '管理默认和自定义搜索引擎。',
  fonts: '自定义字体大小、字体族和页面缩放。',
  network: '配置代理、DNS 和连接设置。',
  media: '自动播放、画中画和硬件加速。',
  notifications: '网站通知权限和显示设置。',
  accessibility: '屏幕阅读器、高对比度和其他辅助功能。',
  shortcuts: '查看和自定义键盘快捷键。',
  system: '默认浏览器、后台应用和系统更新。',
  sync: '在设备间同步书签和设置。',
  gpu: 'GPU 加速、WebGL、WebGPU 与硬件渲染。',
};

const JA_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'ブラウザ',
  autofill: '自動入力とパスワード',
  privacy: 'プライバシーとセキュリティ',
  accessibility: 'アクセシビリティ',
  shortcuts: 'キーボードショートカット',
  appearance: '外観',
  startup: '起動時',
  ai: 'AI と推論',
  extensions: '拡張機能',
  sidebar: 'サイドバー',
  history: '履歴',
  downloads: 'ダウンロード',
  p2p: 'P2P とチャット',
  plugins: 'プラグイン',
  reset: '設定をリセット',
  about: 'Exodus について',
  search: '検索エンジン',
  fonts: 'フォントとズーム',
  network: 'ネットワーク',
  media: 'メディア',
  notifications: '通知',
  system: 'システム',
  sync: '同期',
  profile: 'プロファイル',
  performance: 'パフォーマンス',
  gpu: 'GPUとハードウェア',
};

const KO_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: '브라우저',
  autofill: '자동 완성 및 비밀번호',
  privacy: '개인정보 및 보안',
  appearance: '모양',
  startup: '시작 시',
  ai: 'AI 및 추론',
  extensions: '확장 프로그램',
  sidebar: '사이드바',
  history: '기록',
  downloads: '다운로드',
  p2p: 'P2P 및 채팅',
  plugins: '플러그인',
  reset: '설정 초기화',
  about: 'Exodus 정보',
  search: '검색 엔진',
  fonts: '글꼴 및 확대/축소',
  network: '네트워크',
  media: '미디어',
  notifications: '알림',
  accessibility: '접근성',
  system: '시스템',
  shortcuts: '키보드 단축키',
  sync: '동기화',
  profile: '프로필',
  performance: '성능',
  gpu: 'GPU 및 하드웨어',
};

const ES_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Navegador',
  autofill: 'Autocompletar y contraseñas',
  privacy: 'Privacidad y seguridad',
  appearance: 'Apariencia',
  startup: 'Al iniciar',
  ai: 'IA e inferencia',
  extensions: 'Extensiones',
  sidebar: 'Barra lateral',
  history: 'Historial',
  downloads: 'Descargas',
  p2p: 'P2P y chat',
  plugins: 'Complementos',
  reset: 'Restablecer',
  about: 'Acerca de Exodus',
  search: 'Motor de búsqueda',
  fonts: 'Fuentes y zoom',
  network: 'Red',
  media: 'Medios',
  notifications: 'Notificaciones',
  system: 'Sistema',
  accessibility: 'Accesibilidad',
  shortcuts: 'Atajos de teclado',
  sync: 'Sincronización',
  profile: 'Perfil',
  performance: 'Rendimiento',
  gpu: 'GPU y hardware',
};

const FR_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Navigateur',
  autofill: 'Saisie auto et mots de passe',
  privacy: 'Confidentialité et sécurité',
  appearance: 'Apparence',
  startup: 'Au démarrage',
  ai: 'IA et inférence',
  extensions: 'Extensions',
  sidebar: 'Barre latérale',
  history: 'Historique',
  downloads: 'Téléchargements',
  p2p: 'P2P et chat',
  plugins: 'Plugins',
  reset: 'Réinitialiser',
  about: 'À propos d’Exodus',
  search: 'Moteur de recherche',
  fonts: 'Polices et zoom',
  network: 'Réseau',
  media: 'Médias',
  system: 'Système',
  notifications: 'Notifications',
  accessibility: 'Accessibilité',
  shortcuts: 'Raccourcis clavier',
  sync: 'Synchronisation',
  profile: 'Profil',
  performance: 'Performances',
  gpu: 'GPU et matériel',
};

const DE_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Browser',
  autofill: 'Autofill und Passwörter',
  privacy: 'Datenschutz und Sicherheit',
  appearance: 'Darstellung',
  startup: 'Beim Start',
  ai: 'KI und Inferenz',
  extensions: 'Erweiterungen',
  sidebar: 'Seitenleiste',
  history: 'Verlauf',
  downloads: 'Downloads',
  p2p: 'P2P und Chat',
  plugins: 'Plugins',
  reset: 'Zurücksetzen',
  about: 'Über Exodus',
  search: 'Suchmaschine',
  fonts: 'Schriftarten und Zoom',
  network: 'Netzwerk',
  system: 'System',
  media: 'Medien',
  notifications: 'Benachrichtigungen',
  accessibility: 'Barrierefreiheit',
  shortcuts: 'Tastenkombinationen',
  sync: 'Synchronisation',
  profile: 'Profil',
  performance: 'Leistung',
  gpu: 'GPU und Hardware',
};

const PT_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Navegador',
  autofill: 'Preenchimento e senhas',
  privacy: 'Privacidade e segurança',
  appearance: 'Aparência',
  startup: 'Na inicialização',
  ai: 'IA e inferência',
  extensions: 'Extensões',
  sidebar: 'Barra lateral',
  history: 'Histórico',
  downloads: 'Downloads',
  p2p: 'P2P e chat',
  plugins: 'Plugins',
  reset: 'Redefinir',
  about: 'Sobre o Exodus',
  search: 'Motor de busca',
  fonts: 'Fontes e zoom',
  system: 'Sistema',
  network: 'Rede',
  media: 'Mídia',
  notifications: 'Notificações',
  accessibility: 'Acessibilidade',
  shortcuts: 'Atalhos de teclado',
  sync: 'Sincronização',
  profile: 'Perfil',
  performance: 'Desempenho',
  gpu: 'GPU e hardware',
};

const RU_NAV: Record<ChromeSettingsSectionId, string> = {
  browser: 'Браузер',
  autofill: 'Автозаполнение и пароли',
  privacy: 'Конфиденциальность',
  appearance: 'Внешний вид',
  startup: 'При запуске',
  ai: 'ИИ и инференс',
  extensions: 'Расширения',
  sidebar: 'Боковая панель',
  history: 'История',
  downloads: 'Загрузки',
  p2p: 'P2P и чат',
  plugins: 'Плагины',
  reset: 'Сброс',
  about: 'О Exodus',
  search: 'Поисковая система',
  system: 'Система',
  fonts: 'Шрифты и масштаб',
  network: 'Сеть',
  media: 'Медиа',
  notifications: 'Уведомления',
  accessibility: 'Специальные возможности',
  shortcuts: 'Горячие клавиши',
  sync: 'Синхронизация',
  profile: 'Профиль',
  performance: 'Производительность',
  gpu: 'GPU и оборудование',
};

type SettingsCoreStrings = Pick<
  ChromeSettingsStrings,
  | 'pageTitle'
  | 'close'
  | 'autoSaveIdle'
  | 'autoSaveSaving'
  | 'autoSaveSaved'
  | 'autoSaveError'
  | 'searchPlaceholder'
>;

function buildChromeSettingsStrings(
  nav: Record<ChromeSettingsSectionId, string>,
  desc: Partial<Record<ChromeSettingsSectionId, string>>,
  core: SettingsCoreStrings,
): ChromeSettingsStrings {
  return {
    ...core,
    primaryGroup: '',
    advancedGroup: '',
    footerGroup: '',
    sectionTitle: (id) => nav[id],
    sectionDescription: (id) => desc[id] ?? '',
    nav: (id) => nav[id],
  };
}

const EN_CORE: SettingsCoreStrings = {
  pageTitle: 'Settings',
  close: 'Close',
  autoSaveIdle: 'Changes save automatically',
  autoSaveSaving: 'Saving…',
  autoSaveSaved: 'Saved',
  autoSaveError: 'Could not save',
  searchPlaceholder: 'Search settings',
};

const ZH_CORE: SettingsCoreStrings = {
  pageTitle: '设置',
  close: '关闭',
  autoSaveIdle: '更改会自动保存',
  autoSaveSaving: '正在保存…',
  autoSaveSaved: '已保存',
  autoSaveError: '保存失败',
  searchPlaceholder: '搜索设置',
};

const JA_CORE: SettingsCoreStrings = {
  pageTitle: '設定',
  close: '閉じる',
  autoSaveIdle: '変更は自動的に保存されます',
  autoSaveSaving: '保存中…',
  autoSaveSaved: '保存しました',
  autoSaveError: '保存できませんでした',
  searchPlaceholder: '設定を検索',
};

const KO_CORE: SettingsCoreStrings = {
  pageTitle: '설정',
  close: '닫기',
  autoSaveIdle: '변경 사항이 자동 저장됩니다',
  autoSaveSaving: '저장 중…',
  autoSaveSaved: '저장됨',
  autoSaveError: '저장 실패',
  searchPlaceholder: '설정 검색',
};

const ES_CORE: SettingsCoreStrings = {
  pageTitle: 'Configuración',
  close: 'Cerrar',
  autoSaveIdle: 'Los cambios se guardan automáticamente',
  autoSaveSaving: 'Guardando…',
  autoSaveSaved: 'Guardado',
  autoSaveError: 'No se pudo guardar',
  searchPlaceholder: 'Buscar en ajustes',
};

const FR_CORE: SettingsCoreStrings = {
  pageTitle: 'Paramètres',
  close: 'Fermer',
  autoSaveIdle: 'Les modifications sont enregistrées automatiquement',
  autoSaveSaving: 'Enregistrement…',
  autoSaveSaved: 'Enregistré',
  autoSaveError: 'Échec de l’enregistrement',
  searchPlaceholder: 'Rechercher dans les paramètres',
};

const DE_CORE: SettingsCoreStrings = {
  pageTitle: 'Einstellungen',
  close: 'Schließen',
  autoSaveIdle: 'Änderungen werden automatisch gespeichert',
  autoSaveSaving: 'Speichern…',
  autoSaveSaved: 'Gespeichert',
  autoSaveError: 'Speichern fehlgeschlagen',
  searchPlaceholder: 'Einstellungen durchsuchen',
};

const PT_CORE: SettingsCoreStrings = {
  pageTitle: 'Configurações',
  close: 'Fechar',
  autoSaveIdle: 'As alterações são salvas automaticamente',
  autoSaveSaving: 'Salvando…',
  autoSaveSaved: 'Salvo',
  autoSaveError: 'Não foi possível salvar',
  searchPlaceholder: 'Pesquisar configurações',
};

const RU_CORE: SettingsCoreStrings = {
  pageTitle: 'Настройки',
  close: 'Закрыть',
  autoSaveIdle: 'Изменения сохраняются автоматически',
  autoSaveSaving: 'Сохранение…',
  autoSaveSaved: 'Сохранено',
  autoSaveError: 'Не удалось сохранить',
  searchPlaceholder: 'Поиск в настройках',
};

const SETTINGS_STRING_PACKS: Record<AppLocale, ChromeSettingsStrings> = {
  en: buildChromeSettingsStrings(EN_NAV, EN_DESC, EN_CORE),
  zh: buildChromeSettingsStrings(ZH_NAV, ZH_DESC, ZH_CORE),
  ja: buildChromeSettingsStrings(JA_NAV, EN_DESC, JA_CORE),
  ko: buildChromeSettingsStrings(KO_NAV, EN_DESC, KO_CORE),
  es: buildChromeSettingsStrings(ES_NAV, EN_DESC, ES_CORE),
  fr: buildChromeSettingsStrings(FR_NAV, EN_DESC, FR_CORE),
  de: buildChromeSettingsStrings(DE_NAV, EN_DESC, DE_CORE),
  pt: buildChromeSettingsStrings(PT_NAV, EN_DESC, PT_CORE),
  ru: buildChromeSettingsStrings(RU_NAV, EN_DESC, RU_CORE),
};

/** Resolve settings UI locale (saved preference or browser language). */
export function resolveChromeSettingsLocale(explicit?: ChromeSettingsLocale): ChromeSettingsLocale {
  return resolveAppLocale(explicit);
}

/** Localized settings chrome strings. */
export function chromeSettingsStrings(locale?: ChromeSettingsLocale): ChromeSettingsStrings {
  return SETTINGS_STRING_PACKS[resolveAppLocale(locale)];
}

/** Ordered sidebar navigation (Chrome-like grouping). */
export function chromeSettingsNavItems(locale?: ChromeSettingsLocale): ChromeSettingsNavItem[] {
  const nav = chromeSettingsStrings(locale).nav;
  return [
    { id: 'browser', label: nav('browser'), icon: 'account', group: 'primary' },
    { id: 'autofill', label: nav('autofill'), icon: 'key', group: 'primary' },
    { id: 'privacy', label: nav('privacy'), icon: 'shield', group: 'primary' },
    { id: 'appearance', label: nav('appearance'), icon: 'palette', group: 'primary' },
    { id: 'startup', label: nav('startup'), icon: 'startup', group: 'primary' },
    { id: 'ai', label: nav('ai'), icon: 'ai', group: 'primary' },
    { id: 'search', label: nav('search'), icon: 'search', group: 'primary' },
    { id: 'fonts', label: nav('fonts'), icon: 'font', group: 'primary' },
    { id: 'network', label: nav('network'), icon: 'network', group: 'primary' },
    { id: 'media', label: nav('media'), icon: 'media', group: 'primary' },
    { id: 'notifications', label: nav('notifications'), icon: 'bell', group: 'primary' },
    { id: 'accessibility', label: nav('accessibility'), icon: 'accessibility', group: 'primary' },
    { id: 'shortcuts', label: nav('shortcuts'), icon: 'keyboard', group: 'primary' },
    { id: 'system', label: nav('system'), icon: 'system', group: 'primary' },
    { id: 'sync', label: nav('sync'), icon: 'sync', group: 'primary' },
    { id: 'profile', label: nav('profile'), icon: 'account', group: 'primary' },
    { id: 'performance', label: nav('performance'), icon: 'speed', group: 'primary' },
    { id: 'gpu', label: nav('gpu'), icon: 'gpu', group: 'primary' },
    { id: 'extensions', label: nav('extensions'), icon: 'extensions', group: 'advanced' },
    { id: 'plugins', label: nav('plugins'), icon: 'plugins', group: 'advanced' },
    { id: 'sidebar', label: nav('sidebar'), icon: 'sidebar', group: 'advanced' },
    { id: 'history', label: nav('history'), icon: 'history', group: 'advanced' },
    { id: 'downloads', label: nav('downloads'), icon: 'download', group: 'advanced' },
    { id: 'p2p', label: nav('p2p'), icon: 'p2p', group: 'advanced' },
    { id: 'reset', label: nav('reset'), icon: 'reset', group: 'advanced' },
    { id: 'about', label: nav('about'), icon: 'info', group: 'footer' },
  ];
}

/** Normalize route/hash/alias to a settings section id. */
export function normalizeChromeSettingsSection(input: string | null | undefined): ChromeSettingsSectionId {
  if (!input || typeof input !== 'string') return 'browser';
  const key = input.trim().toLowerCase().replace(/_/g, '-');
  const mapped = SECTION_ALIASES[key];
  if (mapped && VALID_SECTIONS.has(mapped)) return mapped;
  if (VALID_SECTIONS.has(key as ChromeSettingsSectionId)) return key as ChromeSettingsSectionId;
  return 'browser';
}

/**
 * Parse section from chrome://settings URL (supports /privacy, #privacy, ?section=privacy).
 */
export function parseChromeSettingsSection(url: string): ChromeSettingsSectionId {
  const raw = url.trim().toLowerCase();
  if (!raw.startsWith('chrome://settings')) return 'browser';
  const tail = raw.slice('chrome://settings'.length);
  const pathPart = tail.split(/[?#]/)[0].replace(/^\/+/, '');
  if (pathPart) return normalizeChromeSettingsSection(pathPart);
  const hash = tail.includes('#') ? tail.split('#')[1]?.split(/[/?]/)[0] : '';
  if (hash) return normalizeChromeSettingsSection(hash);
  const queryMatch = tail.match(/[?&]section=([^&#]+)/);
  if (queryMatch?.[1]) return normalizeChromeSettingsSection(queryMatch[1]);
  return 'browser';
}

/** Build chrome://settings URL for a section (deep links). */
export function chromeSettingsUrlForSection(section: ChromeSettingsSectionId): string {
  if (section === 'browser') return 'chrome://settings';
  return `chrome://settings/${section}`;
}

/** True when `url` is chrome://settings or a settings subsection path. */
export function isChromeSettingsUrl(url: string): boolean {
  const raw = url.trim().toLowerCase();
  return raw === 'chrome://settings' || raw.startsWith('chrome://settings/');
}

/**
 * True when navigation only changes the settings subsection (not leaving settings).
 * Used to avoid heavy shell work (extension sync) on sidebar clicks inside settings.
 */
export function isChromeSettingsSectionHop(previousUrl: string, nextUrl: string): boolean {
  return isChromeSettingsUrl(previousUrl) && isChromeSettingsUrl(nextUrl);
}

/** Filter nav items by search query (label match). */
export function filterChromeSettingsNav(
  items: ChromeSettingsNavItem[],
  query: string,
): ChromeSettingsNavItem[] {
  const q = query.trim().toLowerCase();
  if (!q) return items;
  return items.filter((item) => item.label.toLowerCase().includes(q));
}
