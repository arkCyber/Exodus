//! Tauri commands for microservice monitoring

use crate::microservice::{
    MetricsCollector, Metric, MetricsStats,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn metrics_counter(
    collector: State<'_, Arc<MetricsCollector>>,
    name: String,
    value: f64,
    labels: HashMap<String, String>,
) -> Result<(), String> {
    collector.counter(name, value, labels).await;
    Ok(())
}

#[tauri::command]
pub async fn metrics_gauge(
    collector: State<'_, Arc<MetricsCollector>>,
    name: String,
    value: f64,
    labels: HashMap<String, String>,
) -> Result<(), String> {
    collector.gauge(name, value, labels).await;
    Ok(())
}

#[tauri::command]
pub async fn metrics_histogram(
    collector: State<'_, Arc<MetricsCollector>>,
    name: String,
    value: f64,
) -> Result<(), String> {
    collector.histogram(name, value).await;
    Ok(())
}

#[tauri::command]
pub async fn metrics_get_metric(
    collector: State<'_, Arc<MetricsCollector>>,
    name: String,
) -> Result<Option<Metric>, String> {
    Ok(collector.get_metric(&name).await)
}

#[tauri::command]
pub async fn metrics_get_all(
    collector: State<'_, Arc<MetricsCollector>>,
) -> Result<Vec<Metric>, String> {
    Ok(collector.get_all_metrics().await)
}

#[tauri::command]
pub async fn metrics_get_stats(
    collector: State<'_, Arc<MetricsCollector>>,
) -> Result<MetricsStats, String> {
    Ok(collector.get_stats().await)
}

#[tauri::command]
pub async fn metrics_export_prometheus(
    collector: State<'_, Arc<MetricsCollector>>,
) -> Result<String, String> {
    Ok(collector.export_prometheus().await)
}

#[tauri::command]
pub async fn metrics_cleanup(
    collector: State<'_, Arc<MetricsCollector>>,
) -> Result<usize, String> {
    Ok(collector.cleanup_old_values().await)
}
