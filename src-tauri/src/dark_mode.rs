//! Dark Mode Support for Exodus Browser
//! 
//! This module provides system-level dark mode detection and theme management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// Theme mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

/// Custom theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    /// Theme ID
    pub id: String,
    /// Theme name
    pub name: String,
    /// Theme description
    pub description: Option<String>,
    /// Is dark theme
    pub is_dark: bool,
    /// CSS variables
    pub css_variables: HashMap<String, String>,
    /// Custom CSS
    pub custom_css: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Created timestamp
    pub created_at: u64,
}

impl CustomTheme {
    pub fn new(name: String, is_dark: bool) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            is_dark,
            css_variables: HashMap::new(),
            custom_css: None,
            author: None,
            created_at: now,
        }
    }
    
    /// Get default light theme
    pub fn default_light() -> Self {
        let mut theme = Self::new("Default Light".to_string(), false);
        theme.css_variables.insert("bg-primary".to_string(), "#ffffff".to_string());
        theme.css_variables.insert("bg-secondary".to_string(), "#f5f5f5".to_string());
        theme.css_variables.insert("text-primary".to_string(), "#000000".to_string());
        theme.css_variables.insert("text-secondary".to_string(), "#666666".to_string());
        theme.css_variables.insert("border".to_string(), "#e0e0e0".to_string());
        theme.css_variables.insert("accent".to_string(), "#007bff".to_string());
        theme
    }
    
    /// Get default dark theme
    pub fn default_dark() -> Self {
        let mut theme = Self::new("Default Dark".to_string(), true);
        theme.css_variables.insert("bg-primary".to_string(), "#1e1e1e".to_string());
        theme.css_variables.insert("bg-secondary".to_string(), "#2d2d2d".to_string());
        theme.css_variables.insert("text-primary".to_string(), "#e0e0e0".to_string());
        theme.css_variables.insert("text-secondary".to_string(), "#a0a0a0".to_string());
        theme.css_variables.insert("border".to_string(), "#404040".to_string());
        theme.css_variables.insert("accent".to_string(), "#4a9eff".to_string());
        theme
    }
}

/// Theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    /// Current theme mode
    pub mode: ThemeMode,
    /// Custom theme ID (if using custom theme)
    pub custom_theme_id: Option<String>,
    /// Follow system theme
    pub follow_system: bool,
    /// Transition duration in ms
    pub transition_duration: u32,
    /// Enable theme animations
    pub enable_animations: bool,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Auto,
            custom_theme_id: None,
            follow_system: true,
            transition_duration: 300,
            enable_animations: true,
        }
    }
}

impl ThemeMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            "auto" => ThemeMode::Auto,
            _ => ThemeMode::Light,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::Auto => "auto",
        }
    }
}

/// Theme manager
pub struct ThemeManager {
    current_mode: Arc<Mutex<ThemeMode>>,
    system_dark: Arc<Mutex<bool>>,
    settings: Arc<Mutex<ThemeSettings>>,
    custom_themes: Arc<Mutex<HashMap<String, CustomTheme>>>,
    storage_path: PathBuf,
}

impl ThemeManager {
    /// Create a new theme manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let system_dark = Self::detect_system_dark_mode();
        
        let manager = Self {
            current_mode: Arc::new(Mutex::new(ThemeMode::Auto)),
            system_dark: Arc::new(Mutex::new(system_dark)),
            settings: Arc::new(Mutex::new(ThemeSettings::default())),
            custom_themes: Arc::new(Mutex::new(HashMap::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Detect system dark mode preference
    fn detect_system_dark_mode() -> bool {
        #[cfg(target_os = "macos")]
        {
            // On macOS, check system appearance
            if let Ok(output) = std::process::Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                let style = String::from_utf8_lossy(&output.stdout);
                return style.trim() == "Dark";
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, check registry
            if let Ok(output) = std::process::Command::new("reg")
                .args(&["query", "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize", "/v", "AppsUseLightTheme"])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                if output.contains("0x0") {
                    return true;
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // On Linux, check common environment variables
            if std::env::var("GTK_THEME").unwrap_or_default().contains("dark") {
                return true;
            }
            if std::env::var("QT_QPA_PLATFORMTHEME").unwrap_or_default().contains("dark") {
                return true;
            }
        }
        
        false
    }
    
    /// Get current theme mode
    pub fn get_mode(&self) -> ThemeMode {
        self.current_mode.lock()
            .map(|mode| *mode)
            .unwrap_or(ThemeMode::Auto)
    }
    
    /// Set theme mode
    pub fn set_mode(&self, mode: ThemeMode) {
        if let Ok(mut current) = self.current_mode.lock() {
            *current = mode;
        }
    }
    
    /// Get effective theme (considering auto mode)
    pub fn get_effective_theme(&self) -> ThemeMode {
        let mode = self.get_mode();
        match mode {
            ThemeMode::Auto => {
                if self.system_dark.lock()
                    .map(|dark| *dark)
                    .unwrap_or(false) {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                }
            }
            _ => mode,
        }
    }
    
    /// Check if dark mode is active
    pub fn is_dark_mode(&self) -> bool {
        self.get_effective_theme() == ThemeMode::Dark
    }
    
    /// Update system dark mode detection
    pub fn update_system_dark_mode(&self) {
        let system_dark = Self::detect_system_dark_mode();
        if let Ok(mut sd) = self.system_dark.lock() {
            *sd = system_dark;
        }
    }
    
    /// Toggle between light and dark mode
    pub fn toggle_theme(&self) {
        let current = self.get_mode();
        let new_mode = match current {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Auto => {
                if self.is_dark_mode() {
                    ThemeMode::Light
                } else {
                    ThemeMode::Dark
                }
            }
        };
        self.set_mode(new_mode);
    }
    
    /// Add a custom theme
    pub fn add_custom_theme(&self, theme: CustomTheme) -> Result<(), Box<dyn std::error::Error>> {
        let mut themes = self.custom_themes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        themes.insert(theme.id.clone(), theme);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Remove a custom theme
    pub fn remove_custom_theme(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut themes = self.custom_themes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        themes.remove(id);
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get a custom theme
    pub fn get_custom_theme(&self, id: &str) -> Option<CustomTheme> {
        let themes = self.custom_themes.lock().ok()?;
        themes.get(id).cloned()
    }
    
    /// Get all custom themes
    pub fn get_all_custom_themes(&self) -> Vec<CustomTheme> {
        let themes = self.custom_themes.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        themes.values().cloned().collect()
    }
    
    /// Update a custom theme
    pub fn update_custom_theme(&self, id: String, theme: CustomTheme) -> Result<(), Box<dyn std::error::Error>> {
        let mut themes = self.custom_themes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if themes.contains_key(&id) {
            themes.insert(id, theme);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Get theme settings
    pub fn get_settings(&self) -> ThemeSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update theme settings
    pub fn update_settings(&self, settings: ThemeSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get active theme (considering custom themes)
    pub fn get_active_theme(&self) -> CustomTheme {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        
        if let Some(custom_id) = &settings.custom_theme_id {
            if let Some(theme) = self.get_custom_theme(custom_id) {
                return theme;
            }
        }
        
        // Return default theme based on mode
        let effective = self.get_effective_theme();
        if effective == ThemeMode::Dark {
            CustomTheme::default_dark()
        } else {
            CustomTheme::default_light()
        }
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("theme_settings.json");
        let themes_path = self.storage_path.join("custom_themes.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: ThemeSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if themes_path.exists() {
            let content = std::fs::read_to_string(&themes_path)?;
            let themes: HashMap<String, CustomTheme> = serde_json::from_str(&content)?;
            if let Ok(mut t) = self.custom_themes.lock() {
                *t = themes;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("theme_settings.json");
        let themes_path = self.storage_path.join("custom_themes.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let themes = self.custom_themes.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let themes_content = serde_json::to_string_pretty(&*themes)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&themes_path, themes_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get current theme mode
#[tauri::command]
pub fn get_theme_mode(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<String, String> {
    Ok(manager.get_mode().as_str().to_string())
}

/// Set theme mode
#[tauri::command]
pub fn set_theme_mode(
    mode: String,
    manager: State<'_, Arc<ThemeManager>>,
    app: AppHandle,
) -> Result<(), String> {
    let theme_mode = ThemeMode::from_str(&mode);
    manager.set_mode(theme_mode);
    
    // Emit event to notify frontend
    let _ = app.emit("theme-changed", theme_mode.as_str());
    
    Ok(())
}

/// Get effective theme
#[tauri::command]
pub fn get_effective_theme(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<String, String> {
    Ok(manager.get_effective_theme().as_str().to_string())
}

/// Check if dark mode is active
#[tauri::command]
pub fn is_dark_mode(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<bool, String> {
    Ok(manager.is_dark_mode())
}

/// Toggle theme
#[tauri::command]
pub fn toggle_theme(
    manager: State<'_, Arc<ThemeManager>>,
    app: AppHandle,
) -> Result<String, String> {
    manager.toggle_theme();
    let new_mode = manager.get_mode();
    
    // Emit event to notify frontend
    let _ = app.emit("theme-changed", new_mode.as_str());
    
    Ok(new_mode.as_str().to_string())
}

/// Update system dark mode detection
#[tauri::command]
pub fn update_system_dark_mode(
    manager: State<'_, Arc<ThemeManager>>,
    app: AppHandle,
) -> Result<(), String> {
    manager.update_system_dark_mode();
    
    // Emit event if theme changed
    let _ = app.emit("theme-changed", manager.get_effective_theme().as_str());
    
    Ok(())
}

/// Inject dark mode CSS into webview
#[tauri::command]
pub fn inject_dark_mode_css(
    app: AppHandle,
    label: String,
    enable: bool,
) -> Result<(), String> {
    let webview = app.get_webview(&label)
        .ok_or_else(|| format!("Webview not found: {}", label))?;
    
    if enable {
        let css = r#"
            (function() {
                let style = document.createElement('style');
                style.id = 'exodus-dark-mode';
                style.textContent = `
                    :root {
                        --exodus-bg-primary: #1e1e1e;
                        --exodus-bg-secondary: #2d2d2d;
                        --exodus-text-primary: #e0e0e0;
                        --exodus-text-secondary: #a0a0a0;
                        --exodus-border: #404040;
                    }
                    
                    body {
                        background-color: var(--exodus-bg-primary) !important;
                        color: var(--exodus-text-primary) !important;
                    }
                    
                    a {
                        color: #4a9eff !important;
                    }
                    
                    input, textarea, select {
                        background-color: var(--exodus-bg-secondary) !important;
                        color: var(--exodus-text-primary) !important;
                        border-color: var(--exodus-border) !important;
                    }
                    
                    button {
                        background-color: var(--exodus-bg-secondary) !important;
                        color: var(--exodus-text-primary) !important;
                    }
                `;
                document.head.appendChild(style);
            })();
        "#;
        webview.eval(css).map_err(|e| format!("Failed to inject dark mode CSS: {}", e))?;
    } else {
        let css = r#"
            (function() {
                let style = document.getElementById('exodus-dark-mode');
                if (style) {
                    style.remove();
                }
            })();
        "#;
        webview.eval(css).map_err(|e| format!("Failed to remove dark mode CSS: {}", e))?;
    }
    
    Ok(())
}

/// Add custom theme
#[tauri::command]
pub fn add_custom_theme(
    theme: CustomTheme,
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<(), String> {
    manager.add_custom_theme(theme)
        .map_err(|e| format!("Failed to add theme: {}", e))
}

/// Remove custom theme
#[tauri::command]
pub fn remove_custom_theme(
    id: String,
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<(), String> {
    manager.remove_custom_theme(&id)
        .map_err(|e| format!("Failed to remove theme: {}", e))
}

/// Get custom theme
#[tauri::command]
pub fn get_custom_theme(
    id: String,
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<Option<CustomTheme>, String> {
    Ok(manager.get_custom_theme(&id))
}

/// Get all custom themes
#[tauri::command]
pub fn get_all_custom_themes(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<Vec<CustomTheme>, String> {
    Ok(manager.get_all_custom_themes())
}

/// Update custom theme
#[tauri::command]
pub fn update_custom_theme(
    id: String,
    theme: CustomTheme,
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<(), String> {
    manager.update_custom_theme(id, theme)
        .map_err(|e| format!("Failed to update theme: {}", e))
}

/// Get theme settings
#[tauri::command]
pub fn get_theme_settings(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<ThemeSettings, String> {
    Ok(manager.get_settings())
}

/// Update theme settings
#[tauri::command]
pub fn update_theme_settings(
    settings: ThemeSettings,
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Get active theme
#[tauri::command]
pub fn get_active_theme(
    manager: State<'_, Arc<ThemeManager>>,
) -> Result<CustomTheme, String> {
    Ok(manager.get_active_theme())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_mode_from_str() {
        assert_eq!(ThemeMode::from_str("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("light"), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("auto"), ThemeMode::Auto);
        assert_eq!(ThemeMode::from_str("invalid"), ThemeMode::Light);
    }
    
    #[test]
    fn test_theme_mode_as_str() {
        assert_eq!(ThemeMode::Light.as_str(), "light");
        assert_eq!(ThemeMode::Dark.as_str(), "dark");
        assert_eq!(ThemeMode::Auto.as_str(), "auto");
    }
    
    #[test]
    fn test_theme_manager() {
        let temp = std::env::temp_dir().join(format!("exodus_theme_{}", uuid::Uuid::new_v4()));
        let manager = ThemeManager::new(temp).expect("Failed to create ThemeManager");
        
        assert_eq!(manager.get_mode(), ThemeMode::Auto);
        
        manager.set_mode(ThemeMode::Dark);
        assert_eq!(manager.get_mode(), ThemeMode::Dark);
        
        manager.toggle_theme();
        assert_eq!(manager.get_mode(), ThemeMode::Light);
    }
    
    #[test]
    fn test_effective_theme() {
        let temp = std::env::temp_dir().join(format!("exodus_theme_{}", uuid::Uuid::new_v4()));
        let manager = ThemeManager::new(temp).expect("Failed to create ThemeManager");
        
        manager.set_mode(ThemeMode::Dark);
        assert_eq!(manager.get_effective_theme(), ThemeMode::Dark);
        
        manager.set_mode(ThemeMode::Light);
        assert_eq!(manager.get_effective_theme(), ThemeMode::Light);
    }
}
