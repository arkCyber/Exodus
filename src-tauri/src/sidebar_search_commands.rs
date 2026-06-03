//! Exodus Browser — Sidebar Search Tauri commands
//!
//! Provides Tauri commands for sidebar search functionality

use tauri::State;

use super::sidebar_search::{SearchEngine, SearchHistoryEntry, SidebarSearchState};

/// Get all sidebar search engines
#[tauri::command]
pub fn sidebar_search_get_engines(
    state: State<'_, SidebarSearchState>,
) -> Result<Vec<SearchEngine>, String> {
    state.get_engines()
}

/// Add custom sidebar search engine
#[tauri::command]
pub fn sidebar_search_add_engine(
    engine: SearchEngine,
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.add_engine(engine)
}

/// Remove sidebar search engine
#[tauri::command]
pub fn sidebar_search_remove_engine(
    engine_id: String,
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.remove_engine(&engine_id)
}

/// Get default sidebar search engine
#[tauri::command]
pub fn sidebar_search_get_default_engine(
    state: State<'_, SidebarSearchState>,
) -> Result<String, String> {
    state.get_default_engine()
}

/// Set default sidebar search engine
#[tauri::command]
pub fn sidebar_search_set_default_engine(
    engine_id: String,
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.set_default_engine(engine_id)
}

/// Add to sidebar search history
#[tauri::command]
pub fn sidebar_search_add_to_history(
    query: String,
    engine: String,
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.add_to_history(query, engine)
}

/// Get sidebar search history
#[tauri::command]
pub fn sidebar_search_get_history(
    limit: usize,
    state: State<'_, SidebarSearchState>,
) -> Result<Vec<SearchHistoryEntry>, String> {
    state.get_history(limit)
}

/// Clear sidebar search history
#[tauri::command]
pub fn sidebar_search_clear_history(
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.clear_history()
}

/// Remove from sidebar search history
#[tauri::command]
pub fn sidebar_search_remove_from_history(
    query: String,
    state: State<'_, SidebarSearchState>,
) -> Result<(), String> {
    state.remove_from_history(&query)
}

/// Get sidebar search suggestions
#[tauri::command]
pub fn sidebar_search_get_suggestions(
    query: String,
    limit: usize,
    state: State<'_, SidebarSearchState>,
) -> Result<Vec<String>, String> {
    state.get_suggestions(&query, limit)
}
