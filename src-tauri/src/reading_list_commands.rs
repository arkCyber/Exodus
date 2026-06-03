//! Exodus Browser — Reading List Tauri commands
//!
//! Provides Tauri commands for reading list functionality

use tauri::State;

use super::reading_list::ReadingListState;

/// Add item to reading list
#[tauri::command]
pub fn reading_list_add(
    url: String,
    title: String,
    excerpt: Option<String>,
    state: State<'_, ReadingListState>,
) -> Result<String, String> {
    let item = state.add_item(url, title, excerpt)?;
    Ok(item.id)
}

/// Remove item from reading list
#[tauri::command]
pub fn reading_list_remove(
    id: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.remove_item(&id)
}

/// Mark item as read
#[tauri::command]
pub fn reading_list_mark_read(
    id: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.mark_as_read(&id)
}

/// Archive item
#[tauri::command]
pub fn reading_list_archive(
    id: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.archive_item(&id)
}

/// Unarchive item
#[tauri::command]
pub fn reading_list_unarchive(
    id: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.unarchive_item(&id)
}

/// Add tag to item
#[tauri::command]
pub fn reading_list_add_tag(
    id: String,
    tag: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.add_tag(&id, tag)
}

/// Remove tag from item
#[tauri::command]
pub fn reading_list_remove_tag(
    id: String,
    tag: String,
    state: State<'_, ReadingListState>,
) -> Result<(), String> {
    state.remove_tag(&id, &tag)
}

/// Get all reading list items
#[tauri::command]
pub fn reading_list_get_all(
    state: State<'_, ReadingListState>,
) -> Result<Vec<super::reading_list::ReadingListItem>, String> {
    state.get_all()
}

/// Get unread items
#[tauri::command]
pub fn reading_list_get_unread(
    state: State<'_, ReadingListState>,
) -> Result<Vec<super::reading_list::ReadingListItem>, String> {
    state.get_unread()
}

/// Get archived items
#[tauri::command]
pub fn reading_list_get_archived(
    state: State<'_, ReadingListState>,
) -> Result<Vec<super::reading_list::ReadingListItem>, String> {
    state.get_archived()
}

/// Get items by tag
#[tauri::command]
pub fn reading_list_get_by_tag(
    tag: String,
    state: State<'_, ReadingListState>,
) -> Result<Vec<super::reading_list::ReadingListItem>, String> {
    state.get_by_tag(&tag)
}

/// Search reading list
#[tauri::command]
pub fn reading_list_search(
    query: String,
    state: State<'_, ReadingListState>,
) -> Result<Vec<super::reading_list::ReadingListItem>, String> {
    state.search(&query)
}
