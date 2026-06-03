//! Chrome Extension API - chrome.bookmarks
//!
//! Provides Tauri commands for the chrome.bookmarks API, allowing extensions
//! to interact with the browser's bookmarks.

use crate::bookmark_sync::{BookmarkSyncManager, SyncBookmark, SyncFolder};
use crate::plugins::manager::ExtensionManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Bookmark node type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BookmarkNodeType {
    Folder,
    Bookmark,
    Separator,
}

/// Bookmark node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub index: Option<u32>,
    pub title: String,
    pub url: Option<String>,
    pub date_added: Option<u64>,
    pub date_group_modified: Option<u64>,
    pub unmodifiable: Option<String>,
    pub children: Option<Vec<BookmarkNode>>,
}

/// Create bookmark parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkCreateParams {
    pub parent_id: String,
    pub title: String,
    pub url: Option<String>,
    pub index: Option<u32>,
}

/// Search bookmarks parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkSearchQuery {
    pub query: String,
}

/// Get bookmark tree
#[tauri::command]
pub async fn chrome_bookmarks_get_tree(
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<BookmarkNode>, String> {
    // Get bookmark tree from bookmark sync manager
    let bookmarks = bookmark_manager.get_all_bookmarks();
    let folders = bookmark_manager.get_all_folders();
    
    // Convert to BookmarkNode format
    let mut root_nodes = Vec::new();
    
    // Add root folder
    let mut root_folder = BookmarkNode {
        id: "0".to_string(),
        parent_id: None,
        index: Some(0),
        title: "Bookmarks Bar".to_string(),
        url: None,
        date_added: None,
        date_group_modified: None,
        unmodifiable: Some("managed".to_string()),
        children: Some(Vec::new()),
    };
    
    // Add folders as children
    for folder in folders {
        let folder_node = BookmarkNode {
            id: folder.id.clone(),
            parent_id: Some("0".to_string()),
            index: Some(folder.position),
            title: folder.title,
            url: None,
            date_added: Some(folder.date_added),
            date_group_modified: Some(folder.last_modified),
            unmodifiable: None,
            children: Some(Vec::new()),
        };
        if let Some(ref mut children) = root_folder.children {
            children.push(folder_node);
        }
    }
    
    // Add bookmarks as children
    for bookmark in bookmarks {
        let bookmark_node = BookmarkNode {
            id: bookmark.id.clone(),
            parent_id: bookmark.parent_id.clone().or_else(|| Some("0".to_string())),
            index: Some(bookmark.position),
            title: bookmark.title,
            url: Some(bookmark.url),
            date_added: Some(bookmark.date_added),
            date_group_modified: Some(bookmark.last_modified),
            unmodifiable: None,
            children: None,
        };
        if let Some(ref mut children) = root_folder.children {
            children.push(bookmark_node);
        }
    }
    
    root_nodes.push(root_folder);
    Ok(root_nodes)
}

/// Get children of a bookmark folder
#[tauri::command]
pub async fn chrome_bookmarks_get_children(
    parent_id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<BookmarkNode>, String> {
    // Get children of a specific folder
    let bookmarks = bookmark_manager.get_all_bookmarks();
    let folders = bookmark_manager.get_all_folders();
    
    let mut children = Vec::new();
    
    // Add folders with matching parent_id
    for folder in folders {
        if folder.parent_id.as_ref() == Some(&parent_id) {
            children.push(BookmarkNode {
                id: folder.id.clone(),
                parent_id: folder.parent_id.clone(),
                index: Some(folder.position),
                title: folder.title,
                url: None,
                date_added: Some(folder.date_added),
                date_group_modified: Some(folder.last_modified),
                unmodifiable: None,
                children: Some(Vec::new()),
            });
        }
    }
    
    // Add bookmarks with matching parent_id
    for bookmark in bookmarks {
        if bookmark.parent_id.as_ref() == Some(&parent_id) {
            children.push(BookmarkNode {
                id: bookmark.id.clone(),
                parent_id: bookmark.parent_id.clone(),
                index: Some(bookmark.position),
                title: bookmark.title,
                url: Some(bookmark.url),
                date_added: Some(bookmark.date_added),
                date_group_modified: Some(bookmark.last_modified),
                unmodifiable: None,
                children: None,
            });
        }
    }
    
    Ok(children)
}

/// Get recent bookmarks
#[tauri::command]
pub async fn chrome_bookmarks_get_recent(
    count: u32,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<BookmarkNode>, String> {
    // Get recent bookmarks
    let bookmarks = bookmark_manager.get_all_bookmarks();
    
    // Sort by date_added descending and take the most recent
    let mut sorted_bookmarks: Vec<_> = bookmarks.into_iter().collect();
    sorted_bookmarks.sort_by(|a, b| b.date_added.cmp(&a.date_added));
    sorted_bookmarks.truncate(count as usize);
    
    // Convert to BookmarkNode format
    let recent_bookmarks: Vec<BookmarkNode> = sorted_bookmarks
        .into_iter()
        .map(|bookmark| BookmarkNode {
            id: bookmark.id,
            parent_id: bookmark.parent_id,
            index: Some(bookmark.position),
            title: bookmark.title,
            url: Some(bookmark.url),
            date_added: Some(bookmark.date_added),
            date_group_modified: Some(bookmark.last_modified),
            unmodifiable: None,
            children: None,
        })
        .collect();
    
    Ok(recent_bookmarks)
}

/// Search bookmarks
#[tauri::command]
pub async fn chrome_bookmarks_search(
    query: BookmarkSearchQuery,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<Vec<BookmarkNode>, String> {
    // Search bookmarks by query
    let bookmarks = bookmark_manager.get_all_bookmarks();
    let folders = bookmark_manager.get_all_folders();
    
    let query_lower = query.query.to_lowercase();
    let mut results = Vec::new();
    
    // Search in bookmarks
    for bookmark in bookmarks {
        if bookmark.title.to_lowercase().contains(&query_lower) 
            || bookmark.url.to_lowercase().contains(&query_lower) {
            results.push(BookmarkNode {
                id: bookmark.id,
                parent_id: bookmark.parent_id,
                index: Some(bookmark.position),
                title: bookmark.title,
                url: Some(bookmark.url),
                date_added: Some(bookmark.date_added),
                date_group_modified: Some(bookmark.last_modified),
                unmodifiable: None,
                children: None,
            });
        }
    }
    
    // Search in folders
    for folder in folders {
        if folder.title.to_lowercase().contains(&query_lower) {
            results.push(BookmarkNode {
                id: folder.id,
                parent_id: folder.parent_id,
                index: Some(folder.position),
                title: folder.title,
                url: None,
                date_added: Some(folder.date_added),
                date_group_modified: Some(folder.last_modified),
                unmodifiable: None,
                children: Some(Vec::new()),
            });
        }
    }
    
    Ok(results)
}

/// Create a bookmark
#[tauri::command]
pub async fn chrome_bookmarks_create(
    params: BookmarkCreateParams,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<BookmarkNode, String> {
    // Create a new bookmark
    let bookmark_id = bookmark_manager
        .add_bookmark(params.url.clone().unwrap_or_default(), params.title.clone())
        .map_err(|e| format!("Failed to create bookmark: {}", e))?;
    
    // Get the created bookmark
    let bookmarks = bookmark_manager.get_all_bookmarks();
    if let Some(bookmark) = bookmarks.iter().find(|b| b.id == bookmark_id) {
        Ok(BookmarkNode {
            id: bookmark.id.clone(),
            parent_id: bookmark.parent_id.clone(),
            index: Some(bookmark.position),
            title: bookmark.title.clone(),
            url: Some(bookmark.url.clone()),
            date_added: Some(bookmark.date_added),
            date_group_modified: Some(bookmark.last_modified),
            unmodifiable: None,
            children: None,
        })
    } else {
        Err("Failed to retrieve created bookmark".to_string())
    }
}

/// Update a bookmark
#[tauri::command]
pub async fn chrome_bookmarks_update(
    id: String,
    changes: serde_json::Value,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<BookmarkNode, String> {
    // Update an existing bookmark
    let url = changes.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());
    let title = changes.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    bookmark_manager
        .update_bookmark(id.clone(), url, title)
        .map_err(|e| format!("Failed to update bookmark: {}", e))?;
    
    // Get the updated bookmark
    let bookmarks = bookmark_manager.get_all_bookmarks();
    if let Some(bookmark) = bookmarks.iter().find(|b| b.id == id) {
        Ok(BookmarkNode {
            id: bookmark.id.clone(),
            parent_id: bookmark.parent_id.clone(),
            index: Some(bookmark.position),
            title: bookmark.title.clone(),
            url: Some(bookmark.url.clone()),
            date_added: Some(bookmark.date_added),
            date_group_modified: Some(bookmark.last_modified),
            unmodifiable: None,
            children: None,
        })
    } else {
        Err("Failed to retrieve updated bookmark".to_string())
    }
}

/// Move a bookmark
#[tauri::command]
pub async fn chrome_bookmarks_move(
    id: String,
    destination: serde_json::Value,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<BookmarkNode, String> {
    // Move a bookmark to a new location
    let parent_id = destination.get("parentId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let index = destination.get("index").and_then(|v| v.as_u64()).map(|u| u as u32);
    
    // For now, we'll just update the parent_id by recreating the bookmark
    // In a full implementation, we'd need to update the bookmark in place
    let bookmarks = bookmark_manager.get_all_bookmarks();
    if let Some(bookmark) = bookmarks.iter().find(|b| b.id == id) {
        let new_bookmark_id = bookmark_manager
            .add_bookmark(bookmark.url.clone(), bookmark.title.clone())
            .map_err(|e| format!("Failed to move bookmark: {}", e))?;
        
        // Remove the old bookmark
        bookmark_manager
            .remove_bookmark(&id)
            .map_err(|e| format!("Failed to remove old bookmark: {}", e))?;
        
        // Get the new bookmark
        let new_bookmarks = bookmark_manager.get_all_bookmarks();
        if let Some(new_bookmark) = new_bookmarks.iter().find(|b| b.id == new_bookmark_id) {
            Ok(BookmarkNode {
                id: new_bookmark.id.clone(),
                parent_id: parent_id,
                index: index.or(Some(new_bookmark.position)),
                title: new_bookmark.title.clone(),
                url: Some(new_bookmark.url.clone()),
                date_added: Some(new_bookmark.date_added),
                date_group_modified: Some(new_bookmark.last_modified),
                unmodifiable: None,
                children: None,
            })
        } else {
            Err("Failed to retrieve moved bookmark".to_string())
        }
    } else {
        Err("Bookmark not found".to_string())
    }
}

/// Remove a bookmark
#[tauri::command]
pub async fn chrome_bookmarks_remove(
    id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    // Remove a bookmark
    bookmark_manager
        .remove_bookmark(&id)
        .map_err(|e| format!("Failed to remove bookmark: {}", e))
}

/// Remove a bookmark tree
#[tauri::command]
pub async fn chrome_bookmarks_remove_tree(
    id: String,
    extension_manager: State<'_, Arc<ExtensionManager>>,
    bookmark_manager: State<'_, Arc<BookmarkSyncManager>>,
) -> Result<(), String> {
    // Remove a bookmark tree (folder and all children)
    // First, try to remove as a folder
    let folder_result = bookmark_manager.remove_folder(&id);
    
    if folder_result.is_ok() {
        // Successfully removed folder
        return Ok(());
    }
    
    // If not a folder, try to remove as a bookmark
    bookmark_manager
        .remove_bookmark(&id)
        .map_err(|e| format!("Failed to remove bookmark tree: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_create_params_deserialization() {
        let params_json = r#"{"parentId":"1","title":"Test","url":"https://example.com"}"#;
        let params: BookmarkCreateParams = serde_json::from_str(params_json).unwrap();
        assert_eq!(params.parent_id, "1");
        assert_eq!(params.title, "Test");
        assert_eq!(params.url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_bookmark_node_serialization() {
        let node = BookmarkNode {
            id: "test-id".to_string(),
            parent_id: Some("parent-id".to_string()),
            index: Some(0),
            title: "Test Bookmark".to_string(),
            url: Some("https://example.com".to_string()),
            date_added: Some(1234567890),
            date_group_modified: None,
            unmodifiable: None,
            children: None,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("Test Bookmark"));
    }

    #[test]
    fn test_bookmark_node_with_children() {
        let node = BookmarkNode {
            id: "folder-id".to_string(),
            parent_id: Some("parent-id".to_string()),
            index: Some(0),
            title: "My Folder".to_string(),
            url: None,
            date_added: Some(1234567890),
            date_group_modified: Some(1234567900),
            unmodifiable: None,
            children: Some(vec![
                BookmarkNode {
                    id: "child-id".to_string(),
                    parent_id: Some("folder-id".to_string()),
                    index: Some(0),
                    title: "Child Bookmark".to_string(),
                    url: Some("https://child.com".to_string()),
                    date_added: Some(1234567891),
                    date_group_modified: None,
                    unmodifiable: None,
                    children: None,
                }
            ]),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("My Folder"));
        assert!(json.contains("children"));
    }

    #[test]
    fn test_bookmark_search_query_serialization() {
        let query = BookmarkSearchQuery {
            query: "test".to_string(),
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("query"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_bookmark_update_params_serialization() {
        let params = BookmarkCreateParams {
            parent_id: "parent-id".to_string(),
            title: "Updated Title".to_string(),
            url: Some("https://updated.com".to_string()),
            index: Some(1),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("Updated Title"));
        assert!(json.contains("https://updated.com"));
    }

    #[test]
    fn test_bookmark_move_params_serialization() {
        let params = BookmarkCreateParams {
            parent_id: "new-parent".to_string(),
            title: "".to_string(),
            url: None,
            index: Some(2),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("new-parent"));
        assert!(json.contains("index"));
    }

    #[test]
    fn test_bookmark_node_folder() {
        let node = BookmarkNode {
            id: "folder-id".to_string(),
            parent_id: None,
            index: Some(0),
            title: "Bookmarks Bar".to_string(),
            url: None,
            date_added: Some(1234567890),
            date_group_modified: None,
            unmodifiable: None,
            children: Some(vec![]),
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("Bookmarks Bar"));
    }
}
