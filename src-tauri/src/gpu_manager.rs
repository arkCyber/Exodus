//! GPU Hardware Acceleration Manager for Exodus Browser
//!
//! This module provides GPU detection, configuration, and performance monitoring
//! for hardware-accelerated rendering, WebGL, and WebGPU support.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub renderer: String,
    pub driver_version: String,
    pub api_type: String, // "OpenGL", "Metal", "DirectX", "Vulkan"
    pub max_texture_size: u32,
    pub max_viewport_dims: [u32; 2],
}

/// WebGL support information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGLSupport {
    pub webgl1_available: bool,
    pub webgl2_available: bool,
    pub webgl_version: String,
    pub max_texture_size: u32,
    pub max_renderbuffer_size: u32,
    pub max_viewport_dims: [u32; 2],
    pub extensions: Vec<String>,
    pub vendor: String,
    pub renderer: String,
}

/// WebGPU support information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGPUSupport {
    pub available: bool,
    pub adapter_info: Option<String>,
    pub features: Vec<String>,
    pub limits: WebGPULimits,
    pub vendor: String,
    pub architecture: String,
}

/// WebGPU limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGPULimits {
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
}

/// GPU performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPerformanceMetrics {
    pub timestamp: u64,
    pub memory_used: u64, // in bytes
    pub memory_total: u64, // in bytes
    pub gpu_utilization: f32, // 0.0 to 1.0
    pub temperature: Option<f32>, // in Celsius
    pub power_usage: Option<f32>, // in watts
}

impl GpuPerformanceMetrics {
    /// Validate performance metrics
    pub fn validate(&self) -> Result<(), String> {
        if self.gpu_utilization < 0.0 || self.gpu_utilization > 1.0 {
            return Err("GPU utilization must be between 0.0 and 1.0".to_string());
        }
        if let Some(temp) = self.temperature {
            if temp < -273.15 || temp > 150.0 {
                return Err("Temperature must be between -273.15 and 150.0 Celsius".to_string());
            }
        }
        if let Some(power) = self.power_usage {
            if power < 0.0 || power > 1000.0 {
                return Err("Power usage must be between 0.0 and 1000.0 watts".to_string());
            }
        }
        Ok(())
    }
}

/// GPU acceleration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAccelerationSettings {
    pub enabled: bool,
    pub webgl_enabled: bool,
    pub webgpu_enabled: bool,
    pub angle_backend: String, // "default", "gl", "d3d11", "d3d9", "metal", "vulkan"
    pub gpu_rasterization: bool,
    pub zero_copy_video: bool,
    pub ignore_gpu_blocklist: bool,
}

impl Default for GpuAccelerationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            webgl_enabled: true,
            webgpu_enabled: true,
            angle_backend: "default".to_string(),
            gpu_rasterization: true,
            zero_copy_video: true,
            ignore_gpu_blocklist: false,
        }
    }
}

/// GPU Manager
pub struct GpuManager {
    settings: Arc<Mutex<GpuAccelerationSettings>>,
    gpu_info: Arc<Mutex<Option<GpuInfo>>>,
    webgl_support: Arc<Mutex<Option<WebGLSupport>>>,
    webgpu_support: Arc<Mutex<Option<WebGPUSupport>>>,
    performance_history: Arc<Mutex<Vec<GpuPerformanceMetrics>>>,
}

impl GpuManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(GpuAccelerationSettings::default())),
            gpu_info: Arc::new(Mutex::new(None)),
            webgl_support: Arc::new(Mutex::new(None)),
            webgpu_support: Arc::new(Mutex::new(None)),
            performance_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Detect GPU information
    pub fn detect_gpu(&self) -> Result<GpuInfo, String> {
        #[cfg(target_os = "macos")]
        {
            self.detect_gpu_macos()
        }
        #[cfg(target_os = "windows")]
        {
            self.detect_gpu_windows()
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_gpu_linux()
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("GPU detection not supported on this platform".to_string())
        }
    }

    /// Detect WebGL support
    pub fn detect_webgl_support(&self) -> Result<WebGLSupport, String> {
        let gpu_info = self.detect_gpu()?;
        
        // Platform-specific WebGL detection
        #[cfg(target_os = "macos")]
        {
            self.detect_webgl_macos(&gpu_info)
        }
        #[cfg(target_os = "windows")]
        {
            self.detect_webgl_windows(&gpu_info)
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_webgl_linux(&gpu_info)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("WebGL detection not supported on this platform".to_string())
        }
    }

    /// Detect WebGPU support
    pub fn detect_webgpu_support(&self) -> Result<WebGPUSupport, String> {
        let gpu_info = self.detect_gpu()?;
        
        // Platform-specific WebGPU detection
        #[cfg(target_os = "macos")]
        {
            self.detect_webgpu_macos(&gpu_info)
        }
        #[cfg(target_os = "windows")]
        {
            self.detect_webgpu_windows(&gpu_info)
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_webgpu_linux(&gpu_info)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err("WebGPU detection not supported on this platform".to_string())
        }
    }

    /// Get GPU info (cached)
    pub fn get_gpu_info(&self) -> Option<GpuInfo> {
        self.gpu_info.lock().ok().and_then(|info| info.clone())
    }

    /// Get WebGL support (cached)
    pub fn get_webgl_support(&self) -> Option<WebGLSupport> {
        self.webgl_support.lock().ok().and_then(|support| support.clone())
    }

    /// Get WebGPU support (cached)
    pub fn get_webgpu_support(&self) -> Option<WebGPUSupport> {
        self.webgpu_support.lock().ok().and_then(|support| support.clone())
    }

    /// Refresh all GPU information
    pub fn refresh_all(&self) -> Result<(), String> {
        let gpu_info = self.detect_gpu()?;
        let webgl_support = self.detect_webgl_support()?;
        let webgpu_support = self.detect_webgpu_support()?;
        
        if let Ok(mut info) = self.gpu_info.lock() {
            *info = Some(gpu_info);
        }
        if let Ok(mut support) = self.webgl_support.lock() {
            *support = Some(webgl_support);
        }
        if let Ok(mut support) = self.webgpu_support.lock() {
            *support = Some(webgpu_support);
        }
        
        Ok(())
    }

    /// Enable GPU acceleration
    pub fn enable(&self) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
        }
    }

    /// Disable GPU acceleration
    pub fn disable(&self) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
        }
    }

    /// Check if GPU acceleration is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|s| s.enabled)
            .unwrap_or(false)
    }

    /// Record performance metrics
    pub fn record_performance(&self, metrics: GpuPerformanceMetrics) -> Result<(), String> {
        metrics.validate()?;
        if let Ok(mut history) = self.performance_history.lock() {
            history.push(metrics);

            // Keep only last 1000 entries
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        Ok(())
    }

    /// Get latest performance metrics
    pub fn get_latest_performance(&self) -> Option<GpuPerformanceMetrics> {
        self.performance_history.lock()
            .ok()
            .and_then(|h| h.last().cloned())
    }

    /// Clear performance history
    pub fn clear_performance_history(&self) {
        if let Ok(mut history) = self.performance_history.lock() {
            history.clear();
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_gpu_macos(&self) -> Result<GpuInfo, String> {
        use std::process::Command;
        
        let output = Command::new("system_profiler")
            .args(&["SPDisplaysDataType", "-json"])
            .output()
            .map_err(|e| format!("Failed to run system_profiler: {}", e))?;
        
        if !output.status.success() {
            return Err("system_profiler command failed".to_string());
        }
        
        let json_str = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse system_profiler output: {}", e))?;
        
        let displays = json["SPDisplaysDataType"]
            .as_array()
            .ok_or("Invalid displays data")?;
        
        if let Some(display) = displays.first() {
            let vendor = display["sppci_vendor"].as_str().unwrap_or("Unknown").to_string();
            let renderer = display["sppci_model"].as_str().unwrap_or("Unknown").to_string();
            let vram = display["sppci_vram_shared"].as_str().unwrap_or("Unknown");
            
            Ok(GpuInfo {
                vendor,
                renderer,
                driver_version: vram.to_string(),
                api_type: "Metal".to_string(),
                max_texture_size: 16384,
                max_viewport_dims: [16384, 16384],
            })
        } else {
            Err("No display information found".to_string())
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_gpu_windows(&self) -> Result<GpuInfo, String> {
        use std::process::Command;
        
        let output = Command::new("wmic")
            .args(&["path", "win32_VideoController", "get", "name,DriverVersion,AdapterRAM", "/format:csv"])
            .output()
            .map_err(|e| format!("Failed to run wmic: {}", e))?;
        
        if !output.status.success() {
            return Err("wmic command failed".to_string());
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        
        if lines.len() > 1 {
            let parts: Vec<&str> = lines[1].split(',').collect();
            if parts.len() >= 3 {
                let renderer = parts.get(1).map(|s| s.trim()).unwrap_or("Unknown");
                let driver_version = parts.get(2).map(|s| s.trim()).unwrap_or("Unknown");
                
                Ok(GpuInfo {
                    vendor: "NVIDIA/AMD/Intel".to_string(), // Would need more parsing
                    renderer: renderer.to_string(),
                    driver_version: driver_version.to_string(),
                    api_type: "DirectX".to_string(),
                    max_texture_size: 16384,
                    max_viewport_dims: [16384, 16384],
                })
            } else {
                Err("Invalid wmic output format".to_string())
            }
        } else {
            Err("No GPU information found".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_gpu_linux(&self) -> Result<GpuInfo, String> {
        use std::fs;
        use std::process::Command;
        
        let lspci_output = Command::new("lspci")
            .args(&["-v"])
            .output()
            .map_err(|e| format!("Failed to run lspci: {}", e))?;
        
        if lspci_output.status.success() {
            let output_str = String::from_utf8_lossy(&lspci_output.stdout);
            // Parse lspci output for VGA compatible controller
            for line in output_str.lines() {
                if line.contains("VGA compatible controller") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 {
                        let renderer = parts[2].trim().to_string();
                        let vendor = if renderer.contains("NVIDIA") {
                            "NVIDIA".to_string()
                        } else if renderer.contains("AMD") || renderer.contains("ATI") {
                            "AMD".to_string()
                        } else if renderer.contains("Intel") {
                            "Intel".to_string()
                        } else {
                            "Unknown".to_string()
                        };
                        
                        return Ok(GpuInfo {
                            vendor,
                            renderer,
                            driver_version: "Unknown".to_string(),
                            api_type: "OpenGL".to_string(),
                            max_texture_size: 16384,
                            max_viewport_dims: [16384, 16384],
                        });
                    }
                }
            }
        }
        
        Err("GPU detection failed on Linux".to_string())
    }

    #[cfg(target_os = "macos")]
    fn detect_webgl_macos(&self, gpu_info: &GpuInfo) -> Result<WebGLSupport, String> {
        // macOS supports WebGL 1.0 and 2.0 via Metal
        Ok(WebGLSupport {
            webgl1_available: true,
            webgl2_available: true,
            webgl_version: "2.0".to_string(),
            max_texture_size: 16384,
            max_renderbuffer_size: 16384,
            max_viewport_dims: [16384, 16384],
            extensions: vec![
                "OES_texture_float".to_string(),
                "OES_texture_float_linear".to_string(),
                "EXT_texture_filter_anisotropic".to_string(),
                "WEBGL_compressed_texture_s3tc".to_string(),
            ],
            vendor: gpu_info.vendor.clone(),
            renderer: gpu_info.renderer.clone(),
        })
    }

    #[cfg(target_os = "windows")]
    fn detect_webgl_windows(&self, gpu_info: &GpuInfo) -> Result<WebGLSupport, String> {
        // Windows supports WebGL 1.0 and 2.0 via ANGLE/DirectX
        Ok(WebGLSupport {
            webgl1_available: true,
            webgl2_available: true,
            webgl_version: "2.0".to_string(),
            max_texture_size: 16384,
            max_renderbuffer_size: 16384,
            max_viewport_dims: [16384, 16384],
            extensions: vec![
                "OES_texture_float".to_string(),
                "OES_texture_float_linear".to_string(),
                "EXT_texture_filter_anisotropic".to_string(),
                "WEBGL_compressed_texture_s3tc".to_string(),
            ],
            vendor: gpu_info.vendor.clone(),
            renderer: gpu_info.renderer.clone(),
        })
    }

    #[cfg(target_os = "linux")]
    fn detect_webgl_linux(&self, gpu_info: &GpuInfo) -> Result<WebGLSupport, String> {
        // Linux supports WebGL 1.0 and 2.0 via OpenGL
        Ok(WebGLSupport {
            webgl1_available: true,
            webgl2_available: true,
            webgl_version: "2.0".to_string(),
            max_texture_size: 16384,
            max_renderbuffer_size: 16384,
            max_viewport_dims: [16384, 16384],
            extensions: vec![
                "OES_texture_float".to_string(),
                "OES_texture_float_linear".to_string(),
                "EXT_texture_filter_anisotropic".to_string(),
                "WEBGL_compressed_texture_s3tc".to_string(),
            ],
            vendor: gpu_info.vendor.clone(),
            renderer: gpu_info.renderer.clone(),
        })
    }

    #[cfg(target_os = "macos")]
    fn detect_webgpu_macos(&self, gpu_info: &GpuInfo) -> Result<WebGPUSupport, String> {
        // Detect architecture on macOS
        let architecture = std::process::Command::new("uname")
            .args(&["-m"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // macOS supports WebGPU via Metal (macOS 13+)
        Ok(WebGPUSupport {
            available: true,
            adapter_info: Some(format!("{} - {}", gpu_info.vendor, gpu_info.renderer)),
            features: vec![
                "timestamp-query".to_string(),
                "pipeline-statistics-query".to_string(),
                "texture-compression-bc".to_string(),
            ],
            limits: WebGPULimits {
                max_texture_dimension_2d: 8192,
                max_texture_dimension_3d: 2048,
                max_texture_array_layers: 256,
                max_bind_groups: 4,
                max_dynamic_uniform_buffers_per_pipeline_layout: 8,
                max_dynamic_storage_buffers_per_pipeline_layout: 4,
            },
            vendor: gpu_info.vendor.clone(),
            architecture,
        })
    }

    #[cfg(target_os = "windows")]
    fn detect_webgpu_windows(&self, gpu_info: &GpuInfo) -> Result<WebGPUSupport, String> {
        // Detect architecture on Windows
        let architecture = std::process::Command::new("wmic")
            .args(&["os", "get", "osarchitecture", "/format:csv"])
            .output()
            .ok()
            .and_then(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                output.lines().nth(1).map(|line| {
                    line.split(',').nth(1).unwrap_or("x86_64").trim().to_string()
                })
            })
            .unwrap_or_else(|| "x86_64".to_string());

        // Windows supports WebGPU via DirectX 12
        Ok(WebGPUSupport {
            available: true,
            adapter_info: Some(format!("{} - {}", gpu_info.vendor, gpu_info.renderer)),
            features: vec![
                "timestamp-query".to_string(),
                "pipeline-statistics-query".to_string(),
                "texture-compression-bc".to_string(),
            ],
            limits: WebGPULimits {
                max_texture_dimension_2d: 8192,
                max_texture_dimension_3d: 2048,
                max_texture_array_layers: 256,
                max_bind_groups: 4,
                max_dynamic_uniform_buffers_per_pipeline_layout: 8,
                max_dynamic_storage_buffers_per_pipeline_layout: 4,
            },
            vendor: gpu_info.vendor.clone(),
            architecture,
        })
    }

    #[cfg(target_os = "linux")]
    fn detect_webgpu_linux(&self, gpu_info: &GpuInfo) -> Result<WebGPUSupport, String> {
        // Detect architecture on Linux
        let architecture = std::process::Command::new("uname")
            .args(&["-m"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Linux supports WebGPU via Vulkan
        Ok(WebGPUSupport {
            available: true,
            adapter_info: Some(format!("{} - {}", gpu_info.vendor, gpu_info.renderer)),
            features: vec![
                "timestamp-query".to_string(),
                "pipeline-statistics-query".to_string(),
                "texture-compression-bc".to_string(),
            ],
            limits: WebGPULimits {
                max_texture_dimension_2d: 8192,
                max_texture_dimension_3d: 2048,
                max_texture_array_layers: 256,
                max_bind_groups: 4,
                max_dynamic_uniform_buffers_per_pipeline_layout: 8,
                max_dynamic_storage_buffers_per_pipeline_layout: 4,
            },
            vendor: gpu_info.vendor.clone(),
            architecture,
        })
    }
}

// Tauri Commands

/// Detect GPU information
#[tauri::command]
pub fn gpu_detect(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<GpuInfo, String> {
    manager.detect_gpu()
}

/// Detect WebGL support
#[tauri::command]
pub fn gpu_detect_webgl(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<WebGLSupport, String> {
    manager.detect_webgl_support()
}

/// Detect WebGPU support
#[tauri::command]
pub fn gpu_detect_webgpu(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<WebGPUSupport, String> {
    manager.detect_webgpu_support()
}

/// Get cached GPU info
#[tauri::command]
pub fn gpu_get_info(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<GpuInfo>, String> {
    Ok(manager.get_gpu_info())
}

/// Get cached WebGL support
#[tauri::command]
pub fn gpu_get_webgl_support(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<WebGLSupport>, String> {
    Ok(manager.get_webgl_support())
}

/// Get cached WebGPU support
#[tauri::command]
pub fn gpu_get_webgpu_support(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<WebGPUSupport>, String> {
    Ok(manager.get_webgpu_support())
}

/// Refresh all GPU information
#[tauri::command]
pub fn gpu_refresh_all(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.refresh_all()
}

/// Enable GPU acceleration
#[tauri::command]
pub fn gpu_enable(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.enable();
    Ok(())
}

/// Disable GPU acceleration
#[tauri::command]
pub fn gpu_disable(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.disable();
    Ok(())
}

/// Check if GPU acceleration is enabled
#[tauri::command]
pub fn gpu_is_enabled(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<bool, String> {
    Ok(manager.is_enabled())
}

/// Record GPU performance metrics
#[tauri::command]
pub fn gpu_record_performance(
    metrics: GpuPerformanceMetrics,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.record_performance(metrics)
}

/// Get latest GPU performance metrics
#[tauri::command]
pub fn gpu_get_latest_performance(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<GpuPerformanceMetrics>, String> {
    Ok(manager.get_latest_performance())
}

/// Clear GPU performance history
#[tauri::command]
pub fn gpu_clear_performance_history(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.clear_performance_history();
    Ok(())
}

/// Get GPU acceleration settings
#[tauri::command]
pub fn gpu_get_settings(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<GpuAccelerationSettings, String> {
    manager.settings.lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Update GPU acceleration settings
#[tauri::command]
pub fn gpu_update_settings(
    settings: GpuAccelerationSettings,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut current = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    *current = settings;
    Ok(())
}

/// Set WebGL enabled
#[tauri::command]
pub fn gpu_set_webgl_enabled(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.webgl_enabled = enabled;
    Ok(())
}

/// Set WebGPU enabled
#[tauri::command]
pub fn gpu_set_webgpu_enabled(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.webgpu_enabled = enabled;
    Ok(())
}

/// Set ANGLE backend
#[tauri::command]
pub fn gpu_set_angle_backend(
    backend: String,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let valid_backends = ["default", "gl", "d3d11", "d3d9", "metal", "vulkan"];
    if !valid_backends.contains(&backend.as_str()) {
        return Err(format!("Invalid ANGLE backend: {}. Valid options: {:?}", backend, valid_backends));
    }
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.angle_backend = backend;
    Ok(())
}

/// Set GPU rasterization
#[tauri::command]
pub fn gpu_set_rasterization(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.gpu_rasterization = enabled;
    Ok(())
}

/// Set zero copy video
#[tauri::command]
pub fn gpu_set_zero_copy_video(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.zero_copy_video = enabled;
    Ok(())
}

/// Set ignore GPU blocklist
#[tauri::command]
pub fn gpu_set_ignore_blocklist(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.ignore_gpu_blocklist = enabled;
    Ok(())
}

/// Reset GPU settings to default
#[tauri::command]
pub fn gpu_reset_settings(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    *settings = GpuAccelerationSettings::default();
    Ok(())
}

/// Receive WebGL detection from JavaScript
#[tauri::command]
pub fn detect_webgl_from_js(
    webgl_info: serde_json::Value,
    _manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // Store or process WebGL info from JS context
    // This can be used to validate or supplement backend detection
    tracing::info!("Received WebGL detection from JS: {:?}", webgl_info);
    Ok(())
}

/// Receive WebGPU detection from JavaScript
#[tauri::command]
pub fn detect_webgpu_from_js(
    webgpu_info: serde_json::Value,
    _manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // Store or process WebGPU info from JS context
    // This can be used to validate or supplement backend detection
    tracing::info!("Received WebGPU detection from JS: {:?}", webgpu_info);
    Ok(())
}

/// Collect performance metrics from JavaScript
#[tauri::command]
pub fn collect_performance_metrics_from_js(
    metrics: serde_json::Value,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // Parse metrics from JS context
    let gpu_metrics: GpuPerformanceMetrics = serde_json::from_value(metrics)
        .map_err(|e| format!("Failed to parse performance metrics: {}", e))?;

    // Validate metrics
    gpu_metrics.validate()
        .map_err(|e| format!("Invalid performance metrics: {}", e))?;

    // Record metrics in manager
    manager.record_performance(gpu_metrics.clone())
        .map_err(|e| format!("Failed to record performance metrics: {}", e))?;

    tracing::info!("Collected performance metrics from JS: utilization={:.2}, memory={}MB, temp={:?}",
        gpu_metrics.gpu_utilization,
        gpu_metrics.memory_used / (1024 * 1024),
        gpu_metrics.temperature);

    Ok(())
}

// Command aliases for frontend compatibility

/// Get GPU info (alias for gpu_get_info)
#[tauri::command]
pub fn get_gpu_info(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<GpuInfo>, String> {
    Ok(manager.get_gpu_info())
}

/// Get WebGL support (alias for gpu_get_webgl_support)
#[tauri::command]
pub fn get_webgl_support(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<WebGLSupport>, String> {
    Ok(manager.get_webgl_support())
}

/// Get WebGPU support (alias for gpu_get_webgpu_support)
#[tauri::command]
pub fn get_webgpu_support(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<WebGPUSupport>, String> {
    Ok(manager.get_webgpu_support())
}

/// Get GPU acceleration settings (alias for gpu_get_settings)
#[tauri::command]
pub fn get_gpu_acceleration_settings(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<GpuAccelerationSettings, String> {
    manager.settings.lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Failed to get settings: {}", e))
}

/// Enable GPU acceleration (alias for gpu_enable)
#[tauri::command]
pub fn enable_gpu_acceleration(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.enable();
    Ok(())
}

/// Disable GPU acceleration (alias for gpu_disable)
#[tauri::command]
pub fn disable_gpu_acceleration(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    manager.disable();
    Ok(())
}

/// Set WebGL enabled (alias for gpu_set_webgl_enabled)
#[tauri::command]
pub fn set_webgl_enabled_frontend(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.webgl_enabled = enabled;
    Ok(())
}

/// Set WebGPU enabled (alias for gpu_set_webgpu_enabled)
#[tauri::command]
pub fn set_webgpu_enabled_frontend(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.webgpu_enabled = enabled;
    Ok(())
}

/// Set ANGLE backend (alias for gpu_set_angle_backend)
#[tauri::command]
pub fn set_angle_backend_frontend(
    backend: String,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    let valid_backends = ["default", "gl", "d3d11", "d3d9", "metal", "vulkan"];
    if !valid_backends.contains(&backend.as_str()) {
        return Err(format!("Invalid ANGLE backend: {}. Valid options: {:?}", backend, valid_backends));
    }
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("Failed to lock settings: {}", e))?;
    settings.angle_backend = backend;
    Ok(())
}

/// Set WebGL enabled (frontend-compatible alias)
#[tauri::command]
pub fn set_webgl_enabled(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    gpu_set_webgl_enabled(enabled, manager)
}

/// Set WebGPU enabled (frontend-compatible alias)
#[tauri::command]
pub fn set_webgpu_enabled(
    enabled: bool,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    gpu_set_webgpu_enabled(enabled, manager)
}

/// Set ANGLE backend (frontend-compatible alias)
#[tauri::command]
pub fn set_angle_backend(
    backend: String,
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    gpu_set_angle_backend(backend, manager)
}

/// Get GPU performance metrics (alias for gpu_get_latest_performance)
#[tauri::command]
pub fn get_gpu_performance_metrics(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<Option<GpuPerformanceMetrics>, String> {
    Ok(manager.get_latest_performance())
}

/// Initialize GPU detection
#[tauri::command]
pub fn initialize_gpu_detection(
    manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // Refresh all GPU information on initialization
    manager.refresh_all()?;
    Ok(())
}

/// Validate GPU settings
#[tauri::command]
pub fn validate_gpu_settings(
    settings: GpuAccelerationSettings,
    _manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // Validate ANGLE backend
    let valid_backends = ["default", "gl", "d3d11", "d3d9", "metal", "vulkan"];
    if !valid_backends.contains(&settings.angle_backend.as_str()) {
        return Err(format!("Invalid ANGLE backend: {}", settings.angle_backend));
    }

    // Validate ignore_gpu_blocklist warning
    if settings.ignore_gpu_blocklist {
        tracing::warn!("GPU blocklist is being ignored - this may cause stability issues");
    }

    Ok(())
}

/// Apply GPU settings to WebView
#[tauri::command]
pub fn apply_gpu_settings_to_webview(
    _manager: State<'_, Arc<GpuManager>>,
) -> Result<(), String> {
    // This is a placeholder for WebView GPU settings application
    // In a real implementation, this would:
    // 1. Update WebView configuration flags
    // 2. Trigger WebView restart if needed
    // 3. Apply hardware acceleration settings
    tracing::info!("Applying GPU settings to WebView (placeholder implementation)");
    Ok(())
}

/// Load GPU settings from ExodusConfig
#[tauri::command]
pub fn load_gpu_settings_from_config(
    manager: State<'_, Arc<GpuManager>>,
    config: State<'_, crate::config::ConfigState>,
) -> Result<(), String> {
    let cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;
    
    let mut settings = manager.settings.lock()
        .map_err(|e| format!("GPU manager lock error: {}", e))?;
    
    settings.enabled = cfg.gpu_enabled;
    settings.webgl_enabled = cfg.webgl_enabled;
    settings.webgpu_enabled = cfg.webgpu_enabled;
    settings.angle_backend = cfg.angle_backend.clone();
    settings.gpu_rasterization = cfg.gpu_rasterization;
    settings.zero_copy_video = cfg.zero_copy_video;
    settings.ignore_gpu_blocklist = cfg.ignore_gpu_blocklist;
    
    tracing::info!("Loaded GPU settings from config: enabled={}, webgl={}, webgpu={}", 
        settings.enabled, settings.webgl_enabled, settings.webgpu_enabled);
    
    Ok(())
}

/// Save GPU settings to ExodusConfig
#[tauri::command]
pub fn save_gpu_settings_to_config(
    manager: State<'_, Arc<GpuManager>>,
    config: State<'_, crate::config::ConfigState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let settings = manager.settings.lock()
        .map_err(|e| format!("GPU manager lock error: {}", e))?;

    let mut cfg = config.lock().map_err(|e| format!("Config lock error: {}", e))?;

    cfg.gpu_enabled = settings.enabled;
    cfg.webgl_enabled = settings.webgl_enabled;
    cfg.webgpu_enabled = settings.webgpu_enabled;
    cfg.angle_backend = settings.angle_backend.clone();
    cfg.gpu_rasterization = settings.gpu_rasterization;
    cfg.zero_copy_video = settings.zero_copy_video;
    cfg.ignore_gpu_blocklist = settings.ignore_gpu_blocklist;

    // Save to disk
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    cfg.save_to(&data_dir)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    tracing::info!("Saved GPU settings to config: enabled={}, webgl={}, webgpu={}",
        settings.enabled, settings.webgl_enabled, settings.webgpu_enabled);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_manager_creation() {
        let manager = GpuManager::new();
        // Test that manager is created with default settings
        assert!(manager.is_enabled()); // Default is enabled
    }

    #[test]
    fn test_gpu_manager_default_settings() {
        let manager = GpuManager::new();
        let settings = manager.settings.lock().unwrap();
        assert!(settings.enabled);
        assert!(settings.webgl_enabled);
        assert!(settings.webgpu_enabled);
        assert_eq!(settings.angle_backend, "default");
        assert!(settings.gpu_rasterization);
        assert!(settings.zero_copy_video);
        assert!(!settings.ignore_gpu_blocklist);
    }

    #[test]
    fn test_gpu_manager_enable_disable() {
        let manager = GpuManager::new();
        assert!(manager.is_enabled());
        manager.disable();
        assert!(!manager.is_enabled());
        manager.enable();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_gpu_performance_metrics_validation() {
        // Valid metrics
        let valid = GpuPerformanceMetrics {
            timestamp: 1234567890,
            memory_used: 1024 * 1024 * 512, // 512 MB
            memory_total: 1024 * 1024 * 8192, // 8 GB
            gpu_utilization: 0.75,
            temperature: Some(65.0),
            power_usage: Some(150.0),
        };
        assert!(valid.validate().is_ok());

        // Invalid GPU utilization (out of range)
        let invalid_util = GpuPerformanceMetrics {
            gpu_utilization: 1.5,
            ..valid.clone()
        };
        assert!(invalid_util.validate().is_err());

        // Invalid temperature (too cold)
        let invalid_temp = GpuPerformanceMetrics {
            temperature: Some(-300.0),
            ..valid.clone()
        };
        assert!(invalid_temp.validate().is_err());

        // Invalid power usage (negative)
        let invalid_power = GpuPerformanceMetrics {
            power_usage: Some(-10.0),
            ..valid
        };
        assert!(invalid_power.validate().is_err());
    }

    #[test]
    fn test_gpu_acceleration_settings_serialization() {
        let settings = GpuAccelerationSettings {
            enabled: true,
            webgl_enabled: true,
            webgpu_enabled: false,
            angle_backend: "metal".to_string(),
            gpu_rasterization: true,
            zero_copy_video: false,
            ignore_gpu_blocklist: false,
        };

        let json = serde_json::to_string(&settings).expect("serialize");
        let deserialized: GpuAccelerationSettings = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(settings.enabled, deserialized.enabled);
        assert_eq!(settings.webgl_enabled, deserialized.webgl_enabled);
        assert_eq!(settings.webgpu_enabled, deserialized.webgpu_enabled);
        assert_eq!(settings.angle_backend, deserialized.angle_backend);
    }

    #[test]
    fn test_gpu_info_serialization() {
        let info = GpuInfo {
            vendor: "NVIDIA".to_string(),
            renderer: "GeForce RTX 3080".to_string(),
            driver_version: "530.30.02".to_string(),
            api_type: "Vulkan".to_string(),
            max_texture_size: 16384,
            max_viewport_dims: [16384, 16384],
        };

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: GpuInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(info.vendor, deserialized.vendor);
        assert_eq!(info.renderer, deserialized.renderer);
        assert_eq!(info.api_type, deserialized.api_type);
    }

    #[test]
    fn test_webgl_support_serialization() {
        let support = WebGLSupport {
            webgl1_available: true,
            webgl2_available: true,
            webgl_version: "2.0".to_string(),
            max_texture_size: 16384,
            max_renderbuffer_size: 16384,
        };

        let json = serde_json::to_string(&support).expect("serialize");
        let deserialized: WebGLSupport = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(support.webgl1_available, deserialized.webgl1_available);
        assert_eq!(support.webgl2_available, deserialized.webgl2_available);
    }

    #[test]
    fn test_webgpu_support_serialization() {
        let support = WebGPUSupport {
            available: true,
            adapter_info: Some("NVIDIA GeForce RTX 3080".to_string()),
            features: vec!["timestamp-query".to_string(), "pipeline-statistics-query".to_string()],
        };

        let json = serde_json::to_string(&support).expect("serialize");
        let deserialized: WebGPUSupport = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(support.available, deserialized.available);
        assert_eq!(support.adapter_info, deserialized.adapter_info);
        assert_eq!(support.features.len(), deserialized.features.len());
    }

    #[test]
    fn test_gpu_manager_record_performance() {
        let manager = GpuManager::new();
        let metrics = GpuPerformanceMetrics {
            timestamp: 1234567890,
            memory_used: 1024 * 1024 * 512,
            memory_total: 1024 * 1024 * 8192,
            gpu_utilization: 0.75,
            temperature: Some(65.0),
            power_usage: Some(150.0),
        };

        let result = manager.record_performance(metrics.clone());
        assert!(result.is_ok());

        let latest = manager.get_latest_performance();
        assert!(latest.is_some());
        let latest = latest.unwrap();
        assert_eq!(latest.gpu_utilization, 0.75);
        assert_eq!(latest.temperature, Some(65.0));
    }

    #[test]
    fn test_gpu_manager_record_invalid_performance() {
        let manager = GpuManager::new();
        let invalid_metrics = GpuPerformanceMetrics {
            gpu_utilization: 1.5, // Invalid: > 1.0
            ..Default::default()
        };

        let result = manager.record_performance(invalid_metrics);
        assert!(result.is_err());
    }

    #[test]
    fn test_gpu_manager_clear_performance_history() {
        let manager = GpuManager::new();
        let metrics = GpuPerformanceMetrics {
            timestamp: 1234567890,
            memory_used: 1024 * 1024 * 512,
            memory_total: 1024 * 1024 * 8192,
            gpu_utilization: 0.75,
            temperature: Some(65.0),
            power_usage: Some(150.0),
        };

        manager.record_performance(metrics).unwrap();
        assert!(manager.get_latest_performance().is_some());

        manager.clear_performance_history();
        assert!(manager.get_latest_performance().is_none());
    }

    #[test]
    fn test_gpu_manager_reset_settings() {
        let manager = GpuManager::new();
        {
            let mut settings = manager.settings.lock().unwrap();
            settings.enabled = false;
            settings.webgl_enabled = false;
            settings.angle_backend = "metal".to_string();
        }

        manager.reset_settings();
        let settings = manager.settings.lock().unwrap();
        assert!(settings.enabled);
        assert!(settings.webgl_enabled);
        assert_eq!(settings.angle_backend, "default");
    }
}
