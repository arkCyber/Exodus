//! Tauri commands for Python microservice

use crate::python_microservice::{
    PythonExecuteRequest, PythonExecuteResponse, PythonMicroservice, PythonMicroserviceConfig,
    PythonServiceInfo, PythonServiceStatus,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn python_microservice_start(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<u32, String> {
    service.start().await
}

#[tauri::command]
pub async fn python_microservice_stop(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<(), String> {
    service.stop().await
}

#[tauri::command]
pub async fn python_microservice_restart(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<u32, String> {
    service.restart().await
}

#[tauri::command]
pub async fn python_microservice_execute(
    service: State<'_, Arc<PythonMicroservice>>,
    request: PythonExecuteRequest,
) -> Result<PythonExecuteResponse, String> {
    service.execute(request).await
}

#[tauri::command]
pub async fn python_microservice_get_info(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<PythonServiceInfo, String> {
    Ok(service.get_info().await)
}

#[tauri::command]
pub async fn python_microservice_get_status(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<PythonServiceStatus, String> {
    Ok(service.get_status().await)
}

#[tauri::command]
pub async fn python_microservice_update_config(
    service: State<'_, Arc<PythonMicroservice>>,
    config: PythonMicroserviceConfig,
) -> Result<(), String> {
    service.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn python_microservice_get_config(
    service: State<'_, Arc<PythonMicroservice>>,
) -> Result<PythonMicroserviceConfig, String> {
    Ok(service.get_config().await)
}
