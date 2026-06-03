//! Annotation & Highlighting for Exodus Browser
//! Provides text highlighting and annotation features

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

/// Annotation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

/// Annotation color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationColor {
    Yellow,
    Green,
    Blue,
    Pink,
    Purple,
    Orange,
}

/// Annotation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub url: String,
    pub annotation_type: AnnotationType,
    pub color: AnnotationColor,
    pub text: String,
    pub note: String,
    pub position: i32, // character position in text
    pub length: i32, // length of highlighted text
    pub created_at: i64,
    pub updated_at: i64,
}

/// Annotation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationSettings {
    pub enabled: bool,
    pub sync_annotations: bool,
    pub show_annotation_indicator: bool,
    pub default_color: AnnotationColor,
}

impl Default for AnnotationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_annotations: false,
            show_annotation_indicator: true,
            default_color: AnnotationColor::Yellow,
        }
    }
}

/// Annotation Manager
pub struct AnnotationManager {
    annotations: Arc<Mutex<HashMap<String, Annotation>>>,
    url_annotations: Arc<Mutex<HashMap<String, Vec<String>>>>, // url -> annotation IDs
    settings: Arc<Mutex<AnnotationSettings>>,
}

impl AnnotationManager {
    pub fn new() -> Self {
        Self {
            annotations: Arc::new(Mutex::new(HashMap::new())),
            url_annotations: Arc::new(Mutex::new(HashMap::new())),
            settings: Arc::new(Mutex::new(AnnotationSettings::default())),
        }
    }

    /// Create a new annotation
    pub fn create_annotation(&self, url: String, annotation_type: AnnotationType, color: AnnotationColor, text: String, note: String, position: i32, length: i32, app: AppHandle) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        
        let annotation = Annotation {
            id: id.clone(),
            url: url.clone(),
            annotation_type,
            color,
            text,
            note,
            position,
            length,
            created_at: now,
            updated_at: now,
        };
        
        if let (Ok(mut annotations), Ok(mut url_annotations)) = (self.annotations.lock(), self.url_annotations.lock()) {
            annotations.insert(id.clone(), annotation.clone());
            url_annotations.entry(url.clone()).or_insert_with(Vec::new).push(id.clone());
            
            let _ = app.emit("exodus-annotation-created", annotation);
            id
        } else {
            id
        }
    }

    /// Update an annotation
    pub fn update_annotation(&self, id: String, note: String, color: AnnotationColor, app: AppHandle) {
        if let Ok(mut annotations) = self.annotations.lock() {
            if let Some(annotation) = annotations.get_mut(&id) {
                annotation.note = note;
                annotation.color = color;
                annotation.updated_at = chrono::Utc::now().timestamp();
                let _ = app.emit("exodus-annotation-updated", annotation.clone());
            }
        }
    }

    /// Delete an annotation
    pub fn delete_annotation(&self, id: String, app: AppHandle) {
        if let (Ok(mut annotations), Ok(mut url_annotations)) = (self.annotations.lock(), self.url_annotations.lock()) {
            if let Some(annotation) = annotations.remove(&id) {
                if let Some(ids) = url_annotations.get_mut(&annotation.url) {
                    ids.retain(|x| x != &id);
                }
                let _ = app.emit("exodus-annotation-deleted", id);
            }
        }
    }

    /// Get annotation by ID
    pub fn get_annotation(&self, id: &str) -> Option<Annotation> {
        self.annotations.lock()
            .ok()
            .and_then(|annotations| annotations.get(id).cloned())
    }

    /// Get all annotations for a URL
    pub fn get_url_annotations(&self, url: &str) -> Vec<Annotation> {
        if let (Ok(annotations), Ok(url_annotations)) = (self.annotations.lock(), self.url_annotations.lock()) {
            if let Some(ids) = url_annotations.get(url) {
                ids.iter()
                    .filter_map(|id| annotations.get(id))
                    .cloned()
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Get all annotations
    pub fn get_all_annotations(&self) -> Vec<Annotation> {
        self.annotations.lock()
            .ok()
            .map(|annotations| annotations.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Search annotations by text
    pub fn search_annotations(&self, query: &str) -> Vec<Annotation> {
        self.annotations.lock()
            .ok()
            .map(|annotations| {
                annotations.values()
                    .filter(|a| a.text.to_lowercase().contains(&query.to_lowercase()) || 
                                a.note.to_lowercase().contains(&query.to_lowercase()))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Enable annotations
    pub fn enable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = true;
            let _ = app.emit("exodus-annotations-enabled", true);
        }
    }

    /// Disable annotations
    pub fn disable(&self, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.enabled = false;
            let _ = app.emit("exodus-annotations-enabled", false);
        }
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.lock()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    /// Set default color
    pub fn set_default_color(&self, color: AnnotationColor, app: AppHandle) {
        if let Ok(mut settings) = self.settings.lock() {
            settings.default_color = color.clone();
            let _ = app.emit("exodus-annotation-default-color-changed", color);
        }
    }

    /// Get settings
    pub fn get_settings(&self) -> AnnotationSettings {
        self.settings.lock()
            .map(|settings| AnnotationSettings {
                enabled: settings.enabled,
                sync_annotations: settings.sync_annotations,
                show_annotation_indicator: settings.show_annotation_indicator,
                default_color: settings.default_color.clone(),
            })
            .unwrap_or_default()
    }
}

impl Default for AnnotationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tauri command to create annotation
#[tauri::command]
pub fn create_annotation(
    url: String,
    annotation_type: String,
    color: String,
    text: String,
    note: String,
    position: i32,
    length: i32,
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) -> String {
    let ann_type = match annotation_type.as_str() {
        "highlight" => AnnotationType::Highlight,
        "note" => AnnotationType::Note,
        "bookmark" => AnnotationType::Bookmark,
        _ => AnnotationType::Highlight,
    };
    
    let ann_color = match color.as_str() {
        "yellow" => AnnotationColor::Yellow,
        "green" => AnnotationColor::Green,
        "blue" => AnnotationColor::Blue,
        "pink" => AnnotationColor::Pink,
        "purple" => AnnotationColor::Purple,
        "orange" => AnnotationColor::Orange,
        _ => AnnotationColor::Yellow,
    };
    
    manager.create_annotation(url, ann_type, ann_color, text, note, position, length, app)
}

/// Tauri command to update annotation
#[tauri::command]
pub fn update_annotation(
    id: String,
    note: String,
    color: String,
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) {
    let ann_color = match color.as_str() {
        "yellow" => AnnotationColor::Yellow,
        "green" => AnnotationColor::Green,
        "blue" => AnnotationColor::Blue,
        "pink" => AnnotationColor::Pink,
        "purple" => AnnotationColor::Purple,
        "orange" => AnnotationColor::Orange,
        _ => AnnotationColor::Yellow,
    };
    
    manager.update_annotation(id, note, ann_color, app);
}

/// Tauri command to delete annotation
#[tauri::command]
pub fn delete_annotation(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) {
    manager.delete_annotation(id, app);
}

/// Tauri command to get annotation
#[tauri::command]
pub fn get_annotation(
    id: String,
    manager: State<'_, Arc<AnnotationManager>>,
) -> Option<Annotation> {
    manager.get_annotation(&id)
}

/// Tauri command to get URL annotations
#[tauri::command]
pub fn get_url_annotations(
    url: String,
    manager: State<'_, Arc<AnnotationManager>>,
) -> Vec<Annotation> {
    manager.get_url_annotations(&url)
}

/// Tauri command to get all annotations
#[tauri::command]
pub fn get_all_annotations(
    manager: State<'_, Arc<AnnotationManager>>,
) -> Vec<Annotation> {
    manager.get_all_annotations()
}

/// Tauri command to search annotations
#[tauri::command]
pub fn search_annotations(
    query: String,
    manager: State<'_, Arc<AnnotationManager>>,
) -> Vec<Annotation> {
    manager.search_annotations(&query)
}

/// Tauri command to enable annotations
#[tauri::command]
pub fn enable_annotations(
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) {
    manager.enable(app);
}

/// Tauri command to disable annotations
#[tauri::command]
pub fn disable_annotations(
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) {
    manager.disable(app);
}

/// Tauri command to check if enabled
#[tauri::command]
pub fn is_annotations_enabled(
    manager: State<'_, Arc<AnnotationManager>>,
) -> bool {
    manager.is_enabled()
}

/// Tauri command to set default color
#[tauri::command]
pub fn set_annotation_default_color(
    color: String,
    app: AppHandle,
    manager: State<'_, Arc<AnnotationManager>>,
) {
    let ann_color = match color.as_str() {
        "yellow" => AnnotationColor::Yellow,
        "green" => AnnotationColor::Green,
        "blue" => AnnotationColor::Blue,
        "pink" => AnnotationColor::Pink,
        "purple" => AnnotationColor::Purple,
        "orange" => AnnotationColor::Orange,
        _ => AnnotationColor::Yellow,
    };
    
    manager.set_default_color(ann_color, app);
}

/// Tauri command to get annotation settings
#[tauri::command]
pub fn get_annotation_settings(
    manager: State<'_, Arc<AnnotationManager>>,
) -> AnnotationSettings {
    manager.get_settings()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_manager_creation() {
        let manager = AnnotationManager::new();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_create_get_annotation() {
        let manager = AnnotationManager::new();
        
        // Mock AppHandle - in real tests you'd use tauri::test::mock_context
        // For now, we just test the state without events
        assert!(manager.get_all_annotations().is_empty());
    }

    #[test]
    fn test_settings() {
        let manager = AnnotationManager::new();
        
        let settings = manager.get_settings();
        assert!(settings.enabled);
    }
}
