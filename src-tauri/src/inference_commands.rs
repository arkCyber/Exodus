//! Tauri commands for LLM Inference Engine

use crate::inference_engine::{
    ChatMessage, ChatRequest, EmbeddingRequest, EmbeddingResponse, InferenceConfig,
    InferenceEngine, InferenceRequest, InferenceResponse, ModelInfo,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn inference_load_model(
    engine: State<'_, Arc<InferenceEngine>>,
    model_name: String,
) -> Result<(), String> {
    engine.load_model(model_name).await
}

#[tauri::command]
pub async fn inference_unload_model(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<(), String> {
    engine.unload_model().await
}

#[tauri::command]
pub async fn inference_generate(
    engine: State<'_, Arc<InferenceEngine>>,
    request: InferenceRequest,
) -> Result<InferenceResponse, String> {
    engine.generate(request).await
}

#[tauri::command]
pub async fn inference_chat(
    engine: State<'_, Arc<InferenceEngine>>,
    request: ChatRequest,
) -> Result<InferenceResponse, String> {
    engine.chat(request).await
}

#[tauri::command]
pub async fn inference_embed(
    engine: State<'_, Arc<InferenceEngine>>,
    request: EmbeddingRequest,
) -> Result<EmbeddingResponse, String> {
    engine.embed(request).await
}

#[tauri::command]
pub async fn inference_add_model(
    engine: State<'_, Arc<InferenceEngine>>,
    model_info: ModelInfo,
) -> Result<(), String> {
    engine.add_model(model_info).await
}

#[tauri::command]
pub async fn inference_remove_model(
    engine: State<'_, Arc<InferenceEngine>>,
    model_name: String,
) -> Result<(), String> {
    engine.remove_model(model_name).await
}

#[tauri::command]
pub async fn inference_list_models(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<Vec<ModelInfo>, String> {
    Ok(engine.list_models().await)
}

#[tauri::command]
pub async fn inference_get_loaded_model(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<Option<String>, String> {
    Ok(engine.get_loaded_model().await)
}

#[tauri::command]
pub async fn inference_get_status(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<String, String> {
    let status = engine.get_status().await;
    Ok(format!("{:?}", status))
}

#[tauri::command]
pub async fn inference_get_stats(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<serde_json::Value, String> {
    let stats = engine.get_stats().await;
    Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn inference_update_config(
    engine: State<'_, Arc<InferenceEngine>>,
    config: InferenceConfig,
) -> Result<(), String> {
    engine.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn inference_get_config(
    engine: State<'_, Arc<InferenceEngine>>,
) -> Result<InferenceConfig, String> {
    Ok(engine.get_config().await)
}
