//! Tauri commands for resource preloader

use crate::resource_preloader::{
    PreloadHint, PreloaderConfig, PreloaderStats, ResourcePreloader,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn preloader_add_hint(
    preloader: State<'_, Arc<ResourcePreloader>>,
    hint: PreloadHint,
) -> Result<(), String> {
    preloader.add_hint(hint).await;
    Ok(())
}

#[tauri::command]
pub async fn preloader_process_queue(
    preloader: State<'_, Arc<ResourcePreloader>>,
) -> Result<usize, String> {
    preloader.process_queue().await
}

#[tauri::command]
pub async fn preloader_get_cached(
    preloader: State<'_, Arc<ResourcePreloader>>,
    url: String,
) -> Result<Option<Vec<u8>>, String> {
    Ok(preloader.get_cached(&url).await)
}

#[tauri::command]
pub async fn preloader_learn_pattern(
    preloader: State<'_, Arc<ResourcePreloader>>,
    source_url: String,
    resource_url: String,
) -> Result<(), String> {
    preloader.learn_pattern(&source_url, &resource_url).await;
    Ok(())
}

#[tauri::command]
pub async fn preloader_get_predictive_hints(
    preloader: State<'_, Arc<ResourcePreloader>>,
    url: String,
) -> Result<Vec<String>, String> {
    Ok(preloader.get_predictive_hints(&url).await)
}

#[tauri::command]
pub async fn preloader_get_stats(
    preloader: State<'_, Arc<ResourcePreloader>>,
) -> Result<PreloaderStats, String> {
    Ok(preloader.get_stats().await)
}

#[tauri::command]
pub async fn preloader_update_config(
    preloader: State<'_, Arc<ResourcePreloader>>,
    config: PreloaderConfig,
) -> Result<(), String> {
    preloader.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn preloader_get_config(
    preloader: State<'_, Arc<ResourcePreloader>>,
) -> Result<PreloaderConfig, String> {
    Ok(preloader.get_config().await)
}

#[tauri::command]
pub async fn preloader_clear_cache(
    preloader: State<'_, Arc<ResourcePreloader>>,
) -> Result<(), String> {
    preloader.clear_cache().await;
    Ok(())
}
