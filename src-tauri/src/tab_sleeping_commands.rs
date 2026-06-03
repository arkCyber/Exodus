//! Tauri commands for tab sleeping management

use crate::tab_sleeping::{TabMetadata, TabSleepConfig, TabSleepManager, TabSleepStats};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn tab_sleep_register(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
    label: String,
    url: String,
    title: String,
    is_pinned: bool,
) -> Result<(), String> {
    let mut metadata = TabMetadata::new(tab_id, label, url);
    metadata.title = title;
    metadata.is_pinned = is_pinned;
    
    manager.register_tab(metadata).await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_unregister(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.unregister_tab(&tab_id).await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_mark_active(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.mark_active(&tab_id).await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_update_media(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
    is_playing_audio: bool,
    is_playing_video: bool,
) -> Result<(), String> {
    manager
        .update_tab(&tab_id, |tab| {
            tab.is_playing_audio = is_playing_audio;
            tab.is_playing_video = is_playing_video;
        })
        .await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_update_memory(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
    memory_mb: f64,
) -> Result<(), String> {
    manager
        .update_tab(&tab_id, |tab| {
            tab.memory_estimate_mb = memory_mb;
        })
        .await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_get_candidates(
    manager: State<'_, Arc<TabSleepManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_tabs_to_sleep().await)
}

#[tauri::command]
pub async fn tab_sleep_mark_sleeping(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.mark_sleeping(&tab_id).await
}

#[tauri::command]
pub async fn tab_sleep_wake(
    manager: State<'_, Arc<TabSleepManager>>,
    tab_id: String,
) -> Result<(), String> {
    manager.wake_tab(&tab_id).await
}

#[tauri::command]
pub async fn tab_sleep_get_all(
    manager: State<'_, Arc<TabSleepManager>>,
) -> Result<Vec<TabMetadata>, String> {
    Ok(manager.get_all_tabs().await)
}

#[tauri::command]
pub async fn tab_sleep_get_stats(
    manager: State<'_, Arc<TabSleepManager>>,
) -> Result<TabSleepStats, String> {
    Ok(manager.get_stats().await)
}

#[tauri::command]
pub async fn tab_sleep_update_config(
    manager: State<'_, Arc<TabSleepManager>>,
    config: TabSleepConfig,
) -> Result<(), String> {
    manager.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn tab_sleep_get_config(
    manager: State<'_, Arc<TabSleepManager>>,
) -> Result<TabSleepConfig, String> {
    Ok(manager.get_config().await)
}
