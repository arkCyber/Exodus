//! PDF Viewer Integration for Exodus Browser
//! 
//! This module provides PDF viewing and management capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

/// PDF viewer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfViewerSettings {
    /// Default zoom level
    pub default_zoom: f64,
    /// Page layout (single, double, continuous)
    pub page_layout: String,
    /// Scroll mode (vertical, horizontal)
    pub scroll_mode: String,
    /// Enable text selection
    pub enable_text_selection: bool,
    /// Enable annotations
    pub enable_annotations: bool,
    /// Enable form filling
    pub enable_form_filling: bool,
    /// Auto-open PDFs in browser
    pub auto_open_in_browser: bool,
    /// Download PDFs instead of viewing
    pub download_instead_of_view: bool,
    /// Default page view (fit, fit-width, fit-page)
    pub default_page_view: String,
}

impl Default for PdfViewerSettings {
    fn default() -> Self {
        Self {
            default_zoom: 1.0,
            page_layout: "single".to_string(),
            scroll_mode: "vertical".to_string(),
            enable_text_selection: true,
            enable_annotations: true,
            enable_form_filling: true,
            auto_open_in_browser: true,
            download_instead_of_view: false,
            default_page_view: "fit".to_string(),
        }
    }
}

/// PDF document info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// Document ID
    pub id: String,
    /// File path or URL
    pub source: String,
    /// Document title
    pub title: Option<String>,
    /// Number of pages
    pub page_count: u32,
    /// File size in bytes
    pub file_size: u64,
    /// Last viewed timestamp
    pub last_viewed: u64,
    /// Current page
    pub current_page: u32,
    /// Zoom level
    pub zoom: f64,
    /// Bookmarks (page numbers)
    pub bookmarks: Vec<u32>,
    /// Notes
    pub notes: HashMap<u32, String>,
}

impl PdfDocument {
    #[allow(dead_code)]
    pub fn new(source: String, title: Option<String>, page_count: u32, file_size: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            title,
            page_count,
            file_size,
            last_viewed: now,
            current_page: 1,
            zoom: 1.0,
            bookmarks: vec![],
            notes: HashMap::new(),
        }
    }
    
    pub fn update_view_state(&mut self, page: u32, zoom: f64) {
        self.current_page = page;
        self.zoom = zoom;
        self.last_viewed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();
    }
    
    pub fn add_bookmark(&mut self, page: u32) {
        if !self.bookmarks.contains(&page) {
            self.bookmarks.push(page);
            self.bookmarks.sort();
        }
    }
    
    pub fn remove_bookmark(&mut self, page: u32) {
        self.bookmarks.retain(|&p| p != page);
    }
    
    pub fn add_note(&mut self, page: u32, note: String) {
        self.notes.insert(page, note);
    }
    
    pub fn remove_note(&mut self, page: u32) {
        self.notes.remove(&page);
    }
}

/// PDF viewer manager
pub struct PdfViewerManager {
    settings: Arc<Mutex<PdfViewerSettings>>,
    recent_documents: Arc<Mutex<Vec<PdfDocument>>>,
    storage_path: PathBuf,
}

impl PdfViewerManager {
    /// Create a new PDF viewer manager
    pub fn new(storage_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&storage_path)?;
        
        let manager = Self {
            settings: Arc::new(Mutex::new(PdfViewerSettings::default())),
            recent_documents: Arc::new(Mutex::new(Vec::new())),
            storage_path,
        };
        
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    /// Get PDF viewer settings
    pub fn get_settings(&self) -> PdfViewerSettings {
        let settings = self.settings.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        settings.clone()
    }
    
    /// Update PDF viewer settings
    pub fn update_settings(&self, settings: PdfViewerSettings) -> Result<(), Box<dyn std::error::Error>> {
        let mut current = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        *current = settings;
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Add document to recent
    pub fn add_recent_document(&self, document: PdfDocument) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        // Remove if already exists
        recent.retain(|d| d.source != document.source);
        
        // Add to front
        recent.insert(0, document);
        
        // Keep only last 20 documents
        if recent.len() > 20 {
            recent.truncate(20);
        }
        
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Get recent documents
    pub fn get_recent_documents(&self) -> Vec<PdfDocument> {
        let recent = self.recent_documents.lock()
            .unwrap_or_else(|_| panic!("Lock error"));
        recent.clone()
    }
    
    /// Clear recent documents
    pub fn clear_recent_documents(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        recent.clear();
        self.save_to_disk()?;
        Ok(())
    }
    
    /// Update document view state
    pub fn update_document_state(&self, source: String, page: u32, zoom: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(doc) = recent.iter_mut().find(|d| d.source == source) {
            doc.update_view_state(page, zoom);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Add bookmark to document
    pub fn add_document_bookmark(&self, source: String, page: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(doc) = recent.iter_mut().find(|d| d.source == source) {
            doc.add_bookmark(page);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Remove bookmark from document
    pub fn remove_document_bookmark(&self, source: String, page: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(doc) = recent.iter_mut().find(|d| d.source == source) {
            doc.remove_bookmark(page);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Add note to document
    pub fn add_document_note(&self, source: String, page: u32, note: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(doc) = recent.iter_mut().find(|d| d.source == source) {
            doc.add_note(page, note);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Remove note from document
    pub fn remove_document_note(&self, source: String, page: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        if let Some(doc) = recent.iter_mut().find(|d| d.source == source) {
            doc.remove_note(page);
            self.save_to_disk()?;
        }
        
        Ok(())
    }
    
    /// Get document by source
    pub fn get_document(&self, source: &str) -> Option<PdfDocument> {
        let recent = self.recent_documents.lock().ok()?;
        recent.iter().find(|d| d.source == source).cloned()
    }
    
    /// Reset to default settings
    pub fn reset_to_default(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.update_settings(PdfViewerSettings::default())
    }
    
    /// Load from disk
    fn load_from_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("pdf_viewer_settings.json");
        let recent_path = self.storage_path.join("recent_documents.json");
        
        if settings_path.exists() {
            let content = std::fs::read_to_string(&settings_path)?;
            let settings: PdfViewerSettings = serde_json::from_str(&content)?;
            if let Ok(mut s) = self.settings.lock() {
                *s = settings;
            }
        }
        
        if recent_path.exists() {
            let content = std::fs::read_to_string(&recent_path)?;
            let recent: Vec<PdfDocument> = serde_json::from_str(&content)?;
            if let Ok(mut r) = self.recent_documents.lock() {
                *r = recent;
            }
        }
        
        Ok(())
    }
    
    /// Save to disk
    fn save_to_disk(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = self.storage_path.join("pdf_viewer_settings.json");
        let recent_path = self.storage_path.join("recent_documents.json");
        
        let settings = self.settings.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        let recent = self.recent_documents.lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        
        let settings_content = serde_json::to_string_pretty(&*settings)?;
        let recent_content = serde_json::to_string_pretty(&*recent)?;
        
        std::fs::write(&settings_path, settings_content)?;
        std::fs::write(&recent_path, recent_content)?;
        
        Ok(())
    }
}

// Tauri Commands

/// Get PDF viewer settings
#[tauri::command]
pub fn get_pdf_viewer_settings(
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<PdfViewerSettings, String> {
    Ok(manager.get_settings())
}

/// Update PDF viewer settings
#[tauri::command]
pub fn update_pdf_viewer_settings(
    settings: PdfViewerSettings,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))
}

/// Reset PDF viewer settings to default
#[tauri::command]
pub fn reset_pdf_viewer_settings(
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.reset_to_default()
        .map_err(|e| format!("Failed to reset settings: {}", e))
}

/// Add recent PDF document
#[tauri::command]
pub fn add_recent_pdf_document(
    document: PdfDocument,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.add_recent_document(document)
        .map_err(|e| format!("Failed to add document: {}", e))
}

/// Get recent PDF documents
#[tauri::command]
pub fn get_recent_pdf_documents(
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<Vec<PdfDocument>, String> {
    Ok(manager.get_recent_documents())
}

/// Clear recent PDF documents
#[tauri::command]
pub fn clear_recent_pdf_documents(
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.clear_recent_documents()
        .map_err(|e| format!("Failed to clear documents: {}", e))
}

/// Update PDF document state
#[tauri::command]
pub fn update_pdf_document_state(
    source: String,
    page: u32,
    zoom: f64,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.update_document_state(source, page, zoom)
        .map_err(|e| format!("Failed to update state: {}", e))
}

/// Add PDF document bookmark
#[tauri::command]
pub fn add_pdf_document_bookmark(
    source: String,
    page: u32,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.add_document_bookmark(source, page)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

/// Remove PDF document bookmark
#[tauri::command]
pub fn remove_pdf_document_bookmark(
    source: String,
    page: u32,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.remove_document_bookmark(source, page)
        .map_err(|e| format!("Failed to remove bookmark: {}", e))
}

/// Add PDF document note
#[tauri::command]
pub fn add_pdf_document_note(
    source: String,
    page: u32,
    note: String,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.add_document_note(source, page, note)
        .map_err(|e| format!("Failed to add note: {}", e))
}

/// Remove PDF document note
#[tauri::command]
pub fn remove_pdf_document_note(
    source: String,
    page: u32,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<(), String> {
    manager.remove_document_note(source, page)
        .map_err(|e| format!("Failed to remove note: {}", e))
}

/// Get PDF document
#[tauri::command]
pub fn get_pdf_document(
    source: String,
    manager: State<'_, Arc<PdfViewerManager>>,
) -> Result<Option<PdfDocument>, String> {
    Ok(manager.get_document(&source))
}
