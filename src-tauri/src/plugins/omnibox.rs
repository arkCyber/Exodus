//! Exodus Browser — chrome.omnibox API implementation
//!
//! Provides omnibox (address bar) suggestions functionality for extensions
//! with high reliability and safety guarantees following aerospace-grade standards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Omnibox suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmniboxSuggestion {
    pub content: String,
    pub description: String,
    #[serde(default)]
    pub deletable: bool,
}

/// Omnibox default suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultSuggestion {
    pub description: String,
}

/// Omnibox registry
pub struct OmniboxRegistry {
    default_suggestions: Arc<Mutex<HashMap<String, DefaultSuggestion>>>,
    suggestions: Arc<Mutex<HashMap<String, Vec<OmniboxSuggestion>>>>,
}

impl OmniboxRegistry {
    pub fn new() -> Self {
        Self {
            default_suggestions: Arc::new(Mutex::new(HashMap::new())),
            suggestions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the default suggestion for an extension
    pub fn set_default_suggestion(
        &self,
        extension_id: &str,
        suggestion: DefaultSuggestion,
    ) -> Result<(), String> {
        let mut suggestions = self
            .default_suggestions
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        suggestions.insert(extension_id.to_string(), suggestion);
        Ok(())
    }

    /// Get the default suggestion for an extension
    pub fn get_default_suggestion(&self, extension_id: &str) -> Option<DefaultSuggestion> {
        let suggestions = self.default_suggestions.lock().ok()?;
        suggestions.get(extension_id).cloned()
    }

    /// Add suggestions for an extension
    pub fn add_suggestions(
        &self,
        extension_id: &str,
        new_suggestions: Vec<OmniboxSuggestion>,
    ) -> Result<(), String> {
        let mut suggestions = self
            .suggestions
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        suggestions
            .entry(extension_id.to_string())
            .or_insert_with(Vec::new)
            .extend(new_suggestions);
        Ok(())
    }

    /// Get suggestions for an extension
    pub fn get_suggestions(&self, extension_id: &str) -> Vec<OmniboxSuggestion> {
        let suggestions = self.suggestions.lock().ok();
        match suggestions {
            Some(guard) => guard.get(extension_id).cloned().unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Clear suggestions for an extension
    pub fn clear_suggestions(&self, extension_id: &str) -> Result<(), String> {
        let mut suggestions = self
            .suggestions
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        suggestions.remove(extension_id);
        Ok(())
    }
}

impl Default for OmniboxRegistry {
    fn default() -> Self {
        Self::new()
    }
}
