//! Exodus Browser — Chrome Omnibox API Tauri commands
//!
//! Provides Tauri commands for chrome.omnibox API

use serde::Serialize;
use tauri::{AppHandle, State};

use super::manager::ExtensionState;
use super::omnibox::{DefaultSuggestion, OmniboxRegistry, OmniboxSuggestion};
use super::runtime;

/// Set the default suggestion for an extension
#[tauri::command]
pub fn extension_omnibox_set_default_suggestion(
    extension_id: String,
    suggestion: DefaultSuggestion,
    state: State<'_, OmniboxRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.set_default_suggestion(&extension_id, suggestion)
}

/// Get the default suggestion for an extension
#[tauri::command]
pub fn extension_omnibox_get_default_suggestion(
    extension_id: String,
    state: State<'_, OmniboxRegistry>,
) -> Result<Option<DefaultSuggestion>, String> {
    let registry = state.inner();
    Ok(registry.get_default_suggestion(&extension_id))
}

/// Add suggestions for an extension
#[tauri::command]
pub fn extension_omnibox_add_suggestions(
    extension_id: String,
    suggestions: Vec<OmniboxSuggestion>,
    state: State<'_, OmniboxRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.add_suggestions(&extension_id, suggestions)
}

/// Get suggestions for an extension
#[tauri::command]
pub fn extension_omnibox_get_suggestions(
    extension_id: String,
    state: State<'_, OmniboxRegistry>,
) -> Result<Vec<OmniboxSuggestion>, String> {
    let registry = state.inner();
    Ok(registry.get_suggestions(&extension_id))
}

/// Clear suggestions for an extension
#[tauri::command]
pub fn extension_omnibox_clear_suggestions(
    extension_id: String,
    state: State<'_, OmniboxRegistry>,
) -> Result<(), String> {
    let registry = state.inner();
    registry.clear_suggestions(&extension_id)
}

/// Extension omnibox keyword for address bar routing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionOmniboxKeyword {
    pub extension_id: String,
    pub extension_name: String,
    pub keyword: String,
}

/// List enabled extensions that declare an `omnibox.keyword` in manifest.json.
#[tauri::command]
pub fn extension_omnibox_list_keywords(
    ext_state: State<'_, ExtensionState>,
) -> Result<Vec<ExtensionOmniboxKeyword>, String> {
    let mgr = ext_state
        .inner()
        .lock()
        .map_err(|e| format!("Extension state lock error: {e}"))?;
    Ok(mgr
        .list()
        .into_iter()
        .filter(|e| e.enabled)
        .filter_map(|e| {
            e.omnibox_keyword.as_ref().map(|keyword| ExtensionOmniboxKeyword {
                extension_id: e.id.clone(),
                extension_name: e.name.clone(),
                keyword: keyword.clone(),
            })
        })
        .collect())
}

/// Dispatch a chrome.omnibox event to the extension background (`onInputChanged`, `onInputEntered`, etc.).
#[tauri::command]
pub fn extension_omnibox_dispatch(
    app: AppHandle,
    extension_id: String,
    event: String,
    text: String,
) -> Result<(), String> {
    runtime::deliver_omnibox_event(&app, &extension_id, &event, &text)
}
