//! Exodus Browser — Tauri commands for the Allama microservice (port 11435).

use crate::allama_manager::{AllamaManager, AllamaStatusDto};
use crate::microservice::allama_service::{send_allama_rpc, AllamaServiceConfig};
use crate::microservice::{ServiceInfo, ServiceRegistry};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Start Allama HTTP (native or embedded) and the UDS control service.
#[tauri::command]
pub async fn allama_service_start(
    app: AppHandle,
    allama: State<'_, Arc<AllamaManager>>,
) -> Result<AllamaStatusDto, String> {
    allama.start().await?;
    let status = allama.status_dto().await;

    let socket_path = AllamaServiceConfig::default().socket_path;
    let service_info = ServiceInfo::new(
        "allama-service",
        socket_path.to_string_lossy().to_string(),
        std::process::id(),
    );
    let _ = app.emit("allama-service-started", service_info);

    Ok(status)
}

/// Stop Allama.
#[tauri::command]
pub async fn allama_service_stop(allama: State<'_, Arc<AllamaManager>>) -> Result<(), String> {
    allama.stop().await
}

/// Restart Allama.
#[tauri::command]
pub async fn allama_service_restart(
    allama: State<'_, Arc<AllamaManager>>,
) -> Result<AllamaStatusDto, String> {
    allama.restart().await?;
    Ok(allama.status_dto().await)
}

/// Allama stack status for settings UI.
#[tauri::command]
pub async fn allama_service_status(
    allama: State<'_, Arc<AllamaManager>>,
) -> Result<AllamaStatusDto, String> {
    Ok(allama.status_dto().await)
}

/// Probe Allama HTTP API (`/api/tags`).
#[tauri::command]
pub async fn allama_http_health(allama: State<'_, Arc<AllamaManager>>) -> Result<bool, String> {
    Ok(allama.http_online().await)
}

/// Query Allama control plane via Unix socket JSON-RPC.
#[tauri::command]
pub async fn allama_control_rpc(method: String) -> Result<serde_json::Value, String> {
    let socket = AllamaServiceConfig::default().socket_path;
    send_allama_rpc(&socket, &method).await
}

/// Register Allama in the microservice registry (metadata only).
#[tauri::command]
pub async fn allama_register_microservice(
    registry: State<'_, Arc<ServiceRegistry>>,
) -> Result<(), String> {
    let socket = AllamaServiceConfig::default().socket_path;
    let info = ServiceInfo::new(
        "allama-service",
        socket.to_string_lossy().to_string(),
        std::process::id(),
    );
    registry.register(info).map_err(|e| e.to_string())
}

/// List models exposed by Allama (`/api/tags` proxy via inference registry).
#[tauri::command]
pub async fn allama_list_models(
    engine: State<'_, Arc<crate::inference_engine::InferenceEngine>>,
) -> Result<serde_json::Value, String> {
    let models = engine.list_models().await;
    Ok(json!({ "models": models }))
}
