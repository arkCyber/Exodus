//! Print Settings for Exodus Browser
//! 
//! This module provides print configuration and management capabilities.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Print orientation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrintOrientation {
    Portrait,
    Landscape,
}

impl PrintOrientation {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "landscape" => PrintOrientation::Landscape,
            _ => PrintOrientation::Portrait,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            PrintOrientation::Portrait => "portrait",
            PrintOrientation::Landscape => "landscape",
        }
    }
}

/// Color mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ColorMode {
    Color,
    Grayscale,
    Monochrome,
}

impl ColorMode {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "grayscale" => ColorMode::Grayscale,
            "monochrome" => ColorMode::Monochrome,
            _ => ColorMode::Color,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            ColorMode::Color => "color",
            ColorMode::Grayscale => "grayscale",
            ColorMode::Monochrome => "monochrome",
        }
    }
}

/// Paper size
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaperSize {
    Letter,
    Legal,
    A4,
    A3,
    A5,
    Custom { width: u32, height: u32 },
}

impl PaperSize {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LEGAL" => PaperSize::Legal,
            "A3" => PaperSize::A3,
            "A5" => PaperSize::A5,
            "A4" => PaperSize::A4,
            _ => PaperSize::Letter,
        }
    }
    
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            PaperSize::Letter => "letter",
            PaperSize::Legal => "legal",
            PaperSize::A4 => "a4",
            PaperSize::A3 => "a3",
            PaperSize::A5 => "a5",
            PaperSize::Custom { .. } => "custom",
        }
    }
    
    #[allow(dead_code)]
    pub fn get_dimensions(&self) -> (u32, u32) {
        match self {
            PaperSize::Letter => (216, 279),
            PaperSize::Legal => (216, 356),
            PaperSize::A4 => (210, 297),
            PaperSize::A3 => (297, 420),
            PaperSize::A5 => (148, 210),
            PaperSize::Custom { width, height } => (*width, *height),
        }
    }
}

/// Margins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        }
    }
}

/// Print settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    /// Default printer
    pub default_printer: Option<String>,
    /// Orientation
    pub orientation: PrintOrientation,
    /// Paper size
    pub paper_size: PaperSize,
    /// Color mode
    pub color_mode: ColorMode,
    /// Margins (in mm)
    pub margins: Margins,
    /// Print background graphics
    pub print_background: bool,
    /// Print headers and footers
    pub print_headers_footers: bool,
    /// Print selection only
    pub print_selection_only: bool,
    /// Number of copies
    pub copies: u32,
    /// Duplex printing
    pub duplex: bool,
    /// Print quality (DPI)
    pub print_quality: u32,
    /// Scale percentage
    pub scale: u32,
    /// Page range
    pub page_range: Option<String>,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            default_printer: None,
            orientation: PrintOrientation::Portrait,
            paper_size: PaperSize::A4,
            color_mode: ColorMode::Color,
            margins: Margins::default(),
            print_background: true,
            print_headers_footers: true,
            print_selection_only: false,
            copies: 1,
            duplex: false,
            print_quality: 600,
            scale: 100,
            page_range: None,
        }
    }
}

/// Print job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    /// Job ID
    pub id: String,
    /// Job name
    pub name: String,
    /// URL being printed
    pub url: String,
    /// Print settings used
    pub settings: PrintSettings,
    /// Status
    pub status: String,
    /// Created timestamp
    pub created_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

impl PrintJob {
    #[allow(dead_code)]
    pub fn new(name: String, url: String, settings: PrintSettings) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            url,
            settings,
            status: "pending".to_string(),
            created_at: now,
            completed_at: None,
        }
    }
}

/// Print settings manager
pub struct PrintSettingsManager {
    settings: Arc<Mutex<PrintSettings>>,
    print_history: Arc<Mutex<Vec<PrintJob>>>,
    storage_path: PathBuf,
}

impl PrintSettingsManager {
    /// Create a new print settings manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(PrintSettings::default())),
            print_history: Arc::new(Mutex::new(Vec::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Get print settings
    pub fn get_settings(&self) -> PrintSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update print settings
    pub fn update_settings(&self, settings: PrintSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Reset to default settings
    pub fn reset_to_default(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.update_settings(PrintSettings::default())
    }
    
    /// Add print job to history
    #[allow(dead_code)]
    pub fn add_print_job(&self, job: PrintJob) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.print_history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.push(job);
        
        // Keep only last 100 jobs
        if history.len() > 100 {
            history.remove(0);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get print history
    pub fn get_print_history(&self) -> Vec<PrintJob> {
        let history = self.print_history.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        history.clone()
    }
    
    /// Clear print history
    pub fn clear_print_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut history = self.print_history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        history.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get available printers (placeholder)
    pub fn get_available_printers(&self) -> Vec<String> {
        // In a real implementation, this would query the system for available printers
        vec![
            "Default Printer".to_string(),
            "Save as PDF".to_string(),
        ]
    }
    
    /// Print to PDF (placeholder)
    pub fn print_to_pdf(&self, _url: String, _output_path: String) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would use the webview's print to PDF functionality
        Ok(())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("print_settings.json");
        let history_path = self.storage_path.join("print_history.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: PrintSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if history_path.exists() {
            let content = std::fs::read_to_string(&history_path)?;
            let history: Vec<PrintJob> = serde_json::from_str(&content)?;
            if let Ok(mut h) = self.print_history.lock() {
                *h = history;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("print_settings.json");
        let history_path = self.storage_path.join("print_history.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let history = self.print_history.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let history_content = serde_json::to_string_pretty(&*history)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&history_path, history_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get print settings
#[tauri::command]
pub fn get_print_settings(
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<PrintSettings, String> {
    Ok(manager.get_settings())
}

/// Update print settings
#[tauri::command]
pub fn update_print_settings(
    settings: PrintSettings,
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Reset print settings to default
#[tauri::command]
pub fn reset_print_settings(
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<(), String> {
    manager.reset_to_default()
        .map_err(|e| format!("Failed to reset settings: {}", e))
}

/// Get available printers
#[tauri::command]
pub fn get_available_printers(
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_available_printers())
}

/// Get print history
#[tauri::command]
pub fn get_print_history(
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<Vec<PrintJob>, String> {
    Ok(manager.get_print_history())
}

/// Clear print history
#[tauri::command]
pub fn clear_print_history(
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<(), String> {
    manager.clear_print_history()
        .map_err(|e| format!("Failed to clear history: {}", e))
}

/// Print to PDF
#[tauri::command]
pub fn print_to_pdf(
    url: String,
    output_path: String,
    manager: State<'_, Arc<PrintSettingsManager>>,
) -> Result<(), String> {
    manager.print_to_pdf(url, output_path)
        .map_err(|e| format!("Failed to print to PDF: {}", e))
}
