/**
 * Exodus Browser — GPU settings UI strings.
 */
import { resolveAppLocale, type AppLocale } from './appLocale';

export interface GpuSettingsStrings {
  title: string;
  hint: string;
  gpuInfo: string;
  vendor: string;
  renderer: string;
  driverVersion: string;
  apiType: string;
  accelerationSection: string;
  gpuAcceleration: string;
  gpuAccelerationHint: string;
  webglSection: string;
  webglEnabled: string;
  webgl1Available: string;
  webgl2Available: string;
  webglVersion: string;
  webgpuSection: string;
  webgpuEnabled: string;
  webgpuAvailable: string;
  adapterInfo: string;
  features: string;
  angleSection: string;
  angleBackend: string;
  angleBackendHint: string;
  angleDefault: string;
  angleGl: string;
  angleD3d11: string;
  angleD3d9: string;
  angleMetal: string;
  angleVulkan: string;
  advancedSection: string;
  gpuRasterization: string;
  zeroCopyVideo: string;
  ignoreGpuBlocklist: string;
  ignoreGpuBlocklistWarning: string;
  performanceSection: string;
  memoryUsed: string;
  memoryTotal: string;
  gpuUtilization: string;
  temperature: string;
  refreshMetrics: string;
  reset: string;
  saved: string;
  saveError: string;
  loading: string;
}

const EN: GpuSettingsStrings = {
  title: 'GPU & Hardware Acceleration',
  hint: 'Configure GPU acceleration, WebGL, WebGPU, and hardware rendering settings.',
  gpuInfo: 'GPU Information',
  vendor: 'Vendor',
  renderer: 'Renderer',
  driverVersion: 'Driver Version',
  apiType: 'API Type',
  accelerationSection: 'Hardware Acceleration',
  gpuAcceleration: 'Use GPU acceleration',
  gpuAccelerationHint: 'Enable hardware acceleration for smoother graphics and video playback.',
  webglSection: 'WebGL',
  webglEnabled: 'Enable WebGL',
  webgl1Available: 'WebGL 1.0',
  webgl2Available: 'WebGL 2.0',
  webglVersion: 'WebGL Version',
  webgpuSection: 'WebGPU',
  webgpuEnabled: 'Enable WebGPU',
  webgpuAvailable: 'WebGPU Available',
  adapterInfo: 'Adapter Info',
  features: 'Features',
  angleSection: 'ANGLE Backend',
  angleBackend: 'ANGLE Backend',
  angleBackendHint: 'Choose the graphics backend for WebGL (default recommended).',
  angleDefault: 'Default',
  angleGl: 'OpenGL',
  angleD3d11: 'Direct3D 11',
  angleD3d9: 'Direct3D 9',
  angleMetal: 'Metal',
  angleVulkan: 'Vulkan',
  advancedSection: 'Advanced',
  gpuRasterization: 'GPU rasterization',
  zeroCopyVideo: 'Zero-copy video',
  ignoreGpuBlocklist: 'Ignore GPU blocklist',
  ignoreGpuBlocklistWarning: 'Warning: Ignoring the GPU blocklist may cause stability issues with certain GPUs.',
  performanceSection: 'Performance Metrics',
  memoryUsed: 'Memory Used',
  memoryTotal: 'Memory Total',
  gpuUtilization: 'GPU Utilization',
  temperature: 'Temperature',
  refreshMetrics: 'Refresh Metrics',
  reset: 'Reset to defaults',
  saved: 'GPU settings saved',
  saveError: 'Failed to save GPU settings',
  loading: 'Loading GPU information...',
};

const ZH: GpuSettingsStrings = {
  title: 'GPU 和硬件加速',
  hint: '配置 GPU 加速、WebGL、WebGPU 和硬件渲染设置。',
  gpuInfo: 'GPU 信息',
  vendor: '供应商',
  renderer: '渲染器',
  driverVersion: '驱动版本',
  apiType: 'API 类型',
  accelerationSection: '硬件加速',
  gpuAcceleration: '使用 GPU 加速',
  gpuAccelerationHint: '启用硬件加速以获得更流畅的图形和视频播放。',
  webglSection: 'WebGL',
  webglEnabled: '启用 WebGL',
  webgl1Available: 'WebGL 1.0',
  webgl2Available: 'WebGL 2.0',
  webglVersion: 'WebGL 版本',
  webgpuSection: 'WebGPU',
  webgpuEnabled: '启用 WebGPU',
  webgpuAvailable: 'WebGPU 可用',
  adapterInfo: '适配器信息',
  features: '功能',
  angleSection: 'ANGLE 后端',
  angleBackend: 'ANGLE 后端',
  angleBackendHint: '选择 WebGL 的图形后端（推荐默认）。',
  angleDefault: '默认',
  angleGl: 'OpenGL',
  angleD3d11: 'Direct3D 11',
  angleD3d9: 'Direct3D 9',
  angleMetal: 'Metal',
  angleVulkan: 'Vulkan',
  advancedSection: '高级',
  gpuRasterization: 'GPU 光栅化',
  zeroCopyVideo: '零拷贝视频',
  ignoreGpuBlocklist: '忽略 GPU 黑名单',
  ignoreGpuBlocklistWarning: '警告：忽略 GPU 黑名单可能会导致某些 GPU 出现稳定性问题。',
  performanceSection: '性能指标',
  memoryUsed: '已用内存',
  memoryTotal: '总内存',
  gpuUtilization: 'GPU 利用率',
  temperature: '温度',
  refreshMetrics: '刷新指标',
  reset: '重置为默认值',
  saved: 'GPU 设置已保存',
  saveError: '保存 GPU 设置失败',
  loading: '正在加载 GPU 信息...',
};

const JA: GpuSettingsStrings = {
  title: 'GPUとハードウェアアクセラレーション',
  hint: 'GPUアクセラレーション、WebGL、WebGPU、ハードウェアレンダリング設定を構成します。',
  gpuInfo: 'GPU情報',
  vendor: 'ベンダー',
  renderer: 'レンダラー',
  driverVersion: 'ドライバーバージョン',
  apiType: 'APIタイプ',
  accelerationSection: 'ハードウェアアクセラレーション',
  gpuAcceleration: 'GPUアクセラレーションを使用',
  gpuAccelerationHint: 'より滑らかなグラフィックとビデオ再生のためにハードウェアアクセラレーションを有効にします。',
  webglSection: 'WebGL',
  webglEnabled: 'WebGLを有効にする',
  webgl1Available: 'WebGL 1.0',
  webgl2Available: 'WebGL 2.0',
  webglVersion: 'WebGLバージョン',
  webgpuSection: 'WebGPU',
  webgpuEnabled: 'WebGPUを有効にする',
  webgpuAvailable: 'WebGPU利用可能',
  adapterInfo: 'アダプター情報',
  features: '機能',
  angleSection: 'ANGLEバックエンド',
  angleBackend: 'ANGLEバックエンド',
  angleBackendHint: 'WebGLのグラフィックバックエンドを選択します（デフォルト推奨）。',
  angleDefault: 'デフォルト',
  angleGl: 'OpenGL',
  angleD3d11: 'Direct3D 11',
  angleD3d9: 'Direct3D 9',
  angleMetal: 'Metal',
  angleVulkan: 'Vulkan',
  advancedSection: '詳細',
  gpuRasterization: 'GPUラスタライゼーション',
  zeroCopyVideo: 'ゼロコピービデオ',
  ignoreGpuBlocklist: 'GPUブロックリストを無視',
  ignoreGpuBlocklistWarning: '警告：GPUブロックリストを無視すると、一部のGPUで安定性の問題が発生する可能性があります。',
  performanceSection: 'パフォーマンス指標',
  memoryUsed: '使用メモリ',
  memoryTotal: '総メモリ',
  gpuUtilization: 'GPU使用率',
  temperature: '温度',
  refreshMetrics: '指標を更新',
  reset: 'デフォルトにリセット',
  saved: 'GPU設定が保存されました',
  saveError: 'GPU設定の保存に失敗しました',
  loading: 'GPU情報を読み込み中...',
};

const PACKS: Record<AppLocale, GpuSettingsStrings> = {
  en: EN,
  zh: ZH,
  ja: JA,
  ko: EN,
  es: EN,
  fr: EN,
  de: EN,
  pt: EN,
  ru: EN,
};

/** Localized GPU settings copy. */
export function gpuSettingsStrings(locale?: AppLocale): GpuSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
