/**
 * Exodus Browser — performance settings UI strings.
 */
import { resolveAppLocale, type AppLocale } from './appLocale';

export interface PerformanceSettingsStrings {
  title: string;
  hint: string;
  memorySection: string;
  memorySaver: string;
  memorySaverHint: string;
  tabMemoryLimit: string;
  unlimited: string;
  tabSection: string;
  suspendBackgroundTabs: string;
  suspendTimeout: string;
  preloadTabs: string;
  cacheSection: string;
  diskCache: string;
  cacheSize: string;
  currentCacheUsage: string;
  clearCache: string;
  renderingSection: string;
  gpuAcceleration: string;
  gpuAccelerationHint: string;
  animatedScroll: string;
  reset: string;
  saved: string;
  saveError: string;
  cacheCleared: string;
  loading: string;
}

const EN: PerformanceSettingsStrings = {
  title: 'Performance',
  hint: 'Optimize browser performance, memory usage, and rendering.',
  memorySection: 'Memory',
  memorySaver: 'Memory saver',
  memorySaverHint: 'Reduce memory usage by suspending inactive tabs and processes.',
  tabMemoryLimit: 'Tab memory limit',
  unlimited: 'Unlimited',
  tabSection: 'Tabs',
  suspendBackgroundTabs: 'Suspend background tabs',
  suspendTimeout: 'Suspend after',
  preloadTabs: 'Preload pages for faster navigation',
  cacheSection: 'Cache',
  diskCache: 'Use disk cache',
  cacheSize: 'Cache size',
  currentCacheUsage: 'Current cache usage',
  clearCache: 'Clear cache',
  renderingSection: 'Rendering',
  gpuAcceleration: 'Use GPU acceleration',
  gpuAccelerationHint: 'Use hardware acceleration for smoother graphics and video.',
  animatedScroll: 'Animated scrolling',
  reset: 'Reset to defaults',
  saved: 'Performance settings saved',
  saveError: 'Failed to save performance settings',
  cacheCleared: 'Cache cleared',
  loading: 'Loading...',
};

const ZH: PerformanceSettingsStrings = {
  title: '性能',
  hint: '优化浏览器性能、内存使用和渲染。',
  memorySection: '内存',
  memorySaver: '内存节省',
  memorySaverHint: '通过挂起非活动标签页和进程来减少内存使用。',
  tabMemoryLimit: '标签页内存限制',
  unlimited: '无限制',
  tabSection: '标签页',
  suspendBackgroundTabs: '挂起后台标签页',
  suspendTimeout: '挂起时间',
  preloadTabs: '预加载页面以加快导航',
  cacheSection: '缓存',
  diskCache: '使用磁盘缓存',
  cacheSize: '缓存大小',
  currentCacheUsage: '当前缓存使用量',
  clearCache: '清除缓存',
  renderingSection: '渲染',
  gpuAcceleration: '使用 GPU 加速',
  gpuAccelerationHint: '使用硬件加速以获得更流畅的图形和视频。',
  animatedScroll: '动画滚动',
  reset: '重置为默认值',
  saved: '性能设置已保存',
  saveError: '保存性能设置失败',
  cacheCleared: '缓存已清除',
  loading: '加载中...',
};

const JA: PerformanceSettingsStrings = {
  title: 'パフォーマンス',
  hint: 'ブラウザのパフォーマンス、メモリ使用量、レンダリングを最適化します。',
  memorySection: 'メモリ',
  memorySaver: 'メモリセーバー',
  memorySaverHint: '非アクティブなタブとプロセスを一時停止してメモリ使用量を削減します。',
  tabMemoryLimit: 'タブのメモリ制限',
  unlimited: '無制限',
  tabSection: 'タブ',
  suspendBackgroundTabs: 'バックグラウンドタブを一時停止',
  suspendTimeout: '一時停止までの時間',
  preloadTabs: 'ページをプリロードしてナビゲーションを高速化',
  cacheSection: 'キャッシュ',
  diskCache: 'ディスクキャッシュを使用',
  cacheSize: 'キャッシュサイズ',
  currentCacheUsage: '現在のキャッシュ使用量',
  clearCache: 'キャッシュをクリア',
  renderingSection: 'レンダリング',
  gpuAcceleration: 'GPUアクセラレーションを使用',
  gpuAccelerationHint: 'ハードウェアアクセラレーションを使用してグラフィックとビデオを滑らかにします。',
  animatedScroll: 'アニメーションスクロール',
  reset: 'デフォルトにリセット',
  saved: 'パフォーマンス設定が保存されました',
  saveError: 'パフォーマンス設定の保存に失敗しました',
  cacheCleared: 'キャッシュがクリアされました',
  loading: '読み込み中...',
};

const KO: PerformanceSettingsStrings = {
  title: '성능',
  hint: '브라우저 성능, 메모리 사용량, 렌더링을 최적화합니다.',
  memorySection: '메모리',
  memorySaver: '메모리 절약',
  memorySaverHint: '비활성 탭과 프로세스를 일시 중단하여 메모리 사용량을 줄입니다.',
  tabMemoryLimit: '탭 메모리 제한',
  unlimited: '무제한',
  tabSection: '탭',
  suspendBackgroundTabs: '백그라운드 탭 일시 중단',
  suspendTimeout: '일시 중단 시간',
  preloadTabs: '페이지를 미리 로드하여 탐색 속도 향상',
  cacheSection: '캐시',
  diskCache: '디스크 캐시 사용',
  cacheSize: '캐시 크기',
  currentCacheUsage: '현재 캐시 사용량',
  clearCache: '캐시 지우기',
  renderingSection: '렌더링',
  gpuAcceleration: 'GPU 가속 사용',
  gpuAccelerationHint: '하드웨어 가속을 사용하여 그래픽과 비디오를 부드럽게 합니다.',
  animatedScroll: '애니메이션 스크롤',
  reset: '기본값으로 재설정',
  saved: '성능 설정이 저장되었습니다',
  saveError: '성능 설정 저장 실패',
  cacheCleared: '캐시가 지워졌습니다',
  loading: '로딩 중...',
};

const ES: PerformanceSettingsStrings = {
  title: 'Rendimiento',
  hint: 'Optimiza el rendimiento del navegador, uso de memoria y renderizado.',
  memorySection: 'Memoria',
  memorySaver: 'Ahorro de memoria',
  memorySaverHint: 'Reduce el uso de memoria suspendiendo pestañas y procesos inactivos.',
  tabMemoryLimit: 'Límite de memoria de pestaña',
  unlimited: 'Ilimitado',
  tabSection: 'Pestañas',
  suspendBackgroundTabs: 'Suspender pestañas en segundo plano',
  suspendTimeout: 'Suspender después de',
  preloadTabs: 'Precargar páginas para una navegación más rápida',
  cacheSection: 'Caché',
  diskCache: 'Usar caché de disco',
  cacheSize: 'Tamaño de caché',
  currentCacheUsage: 'Uso actual de caché',
  clearCache: 'Limpiar caché',
  renderingSection: 'Renderizado',
  gpuAcceleration: 'Usar aceleración de GPU',
  gpuAccelerationHint: 'Usa aceleración de hardware para gráficos y video más fluidos.',
  animatedScroll: 'Desplazamiento animado',
  reset: 'Restablecer valores predeterminados',
  saved: 'Configuración de rendimiento guardada',
  saveError: 'Error al guardar la configuración de rendimiento',
  cacheCleared: 'Caché limpiada',
  loading: 'Cargando...',
};

const FR: PerformanceSettingsStrings = {
  title: 'Performances',
  hint: 'Optimisez les performances du navigateur, l\'utilisation de la mémoire et le rendu.',
  memorySection: 'Mémoire',
  memorySaver: 'Économiseur de mémoire',
  memorySaverHint: 'Réduisez l\'utilisation de la mémoire en suspendant les onglets et processus inactifs.',
  tabMemoryLimit: 'Limite de mémoire par onglet',
  unlimited: 'Illimité',
  tabSection: 'Onglets',
  suspendBackgroundTabs: 'Suspendre les onglets en arrière-plan',
  suspendTimeout: 'Suspendre après',
  preloadTabs: 'Précharger les pages pour une navigation plus rapide',
  cacheSection: 'Cache',
  diskCache: 'Utiliser le cache disque',
  cacheSize: 'Taille du cache',
  currentCacheUsage: 'Utilisation actuelle du cache',
  clearCache: 'Vider le cache',
  renderingSection: 'Rendu',
  gpuAcceleration: 'Utiliser l\'accélération GPU',
  gpuAccelerationHint: 'Utilise l\'accélération matérielle pour des graphiques et vidéos plus fluides.',
  animatedScroll: 'Défilement animé',
  reset: 'Rétablir les valeurs par défaut',
  saved: 'Paramètres de performance enregistrés',
  saveError: 'Échec de l\'enregistrement des paramètres de performance',
  cacheCleared: 'Cache vidé',
  loading: 'Chargement...',
};

const DE: PerformanceSettingsStrings = {
  title: 'Leistung',
  hint: 'Browser-Leistung, Speichernutzung und Rendering optimieren.',
  memorySection: 'Speicher',
  memorySaver: 'Speichersparmodus',
  memorySaverHint: 'Speichernutzung reduzieren durch Aussetzen inaktiver Tabs und Prozesse.',
  tabMemoryLimit: 'Tab-Speicherlimit',
  unlimited: 'Unbegrenzt',
  tabSection: 'Tabs',
  suspendBackgroundTabs: 'Hintergrund-Tabs aussetzen',
  suspendTimeout: 'Aussetzen nach',
  preloadTabs: 'Seiten vorab laden für schnellere Navigation',
  cacheSection: 'Cache',
  diskCache: 'Festplatten-Cache verwenden',
  cacheSize: 'Cache-Größe',
  currentCacheUsage: 'Aktuelle Cache-Nutzung',
  clearCache: 'Cache leeren',
  renderingSection: 'Rendering',
  gpuAcceleration: 'GPU-Beschleunigung verwenden',
  gpuAccelerationHint: 'Hardware-Beschleunigung für flüssigere Grafik und Videos verwenden.',
  animatedScroll: 'Animiertes Scrollen',
  reset: 'Auf Standardwerte zurücksetzen',
  saved: 'Leistungseinstellungen gespeichert',
  saveError: 'Fehler beim Speichern der Leistungseinstellungen',
  cacheCleared: 'Cache geleert',
  loading: 'Wird geladen...',
};

const PT: PerformanceSettingsStrings = {
  title: 'Desempenho',
  hint: 'Otimize o desempenho do navegador, uso de memória e renderização.',
  memorySection: 'Memória',
  memorySaver: 'Economizador de memória',
  memorySaverHint: 'Reduza o uso de memória suspendendo abas e processos inativos.',
  tabMemoryLimit: 'Limite de memória da aba',
  unlimited: 'Ilimitado',
  tabSection: 'Abas',
  suspendBackgroundTabs: 'Suspender abas em segundo plano',
  suspendTimeout: 'Suspender após',
  preloadTabs: 'Pré-carregar páginas para navegação mais rápida',
  cacheSection: 'Cache',
  diskCache: 'Usar cache de disco',
  cacheSize: 'Tamanho do cache',
  currentCacheUsage: 'Uso atual do cache',
  clearCache: 'Limpar cache',
  renderingSection: 'Renderização',
  gpuAcceleration: 'Usar aceleração de GPU',
  gpuAccelerationHint: 'Usa aceleração de hardware para gráficos e vídeos mais suaves.',
  animatedScroll: 'Rolagem animada',
  reset: 'Restaurar padrões',
  saved: 'Configurações de desempenho salvas',
  saveError: 'Falha ao salvar configurações de desempenho',
  cacheCleared: 'Cache limpo',
  loading: 'Carregando...',
};

const RU: PerformanceSettingsStrings = {
  title: 'Производительность',
  hint: 'Оптимизация производительности браузера, использования памяти и рендеринга.',
  memorySection: 'Память',
  memorySaver: 'Экономия памяти',
  memorySaverHint: 'Сократите использование памяти, приостанавливая неактивные вкладки и процессы.',
  tabMemoryLimit: 'Лимит памяти вкладки',
  unlimited: 'Без ограничений',
  tabSection: 'Вкладки',
  suspendBackgroundTabs: 'Приостанавливать фоновые вкладки',
  suspendTimeout: 'Приостановить через',
  preloadTabs: 'Предварительная загрузка страниц для быстрой навигации',
  cacheSection: 'Кэш',
  diskCache: 'Использовать дисковый кэш',
  cacheSize: 'Размер кэша',
  currentCacheUsage: 'Текущее использование кэша',
  clearCache: 'Очистить кэш',
  renderingSection: 'Рендеринг',
  gpuAcceleration: 'Использовать ускорение GPU',
  gpuAccelerationHint: 'Используйте аппаратное ускорение для более плавной графики и видео.',
  animatedScroll: 'Анимированная прокрутка',
  reset: 'Сбросить настройки по умолчанию',
  saved: 'Настройки производительности сохранены',
  saveError: 'Не удалось сохранить настройки производительности',
  cacheCleared: 'Кэш очищен',
  loading: 'Загрузка...',
};

const PACKS: Record<AppLocale, PerformanceSettingsStrings> = {
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

/** Localized performance settings copy. */
export function performanceSettingsStrings(locale?: AppLocale): PerformanceSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
