//! Tauri commands for Hermes Agent

use crate::hermes_agent::{
    AgentContext, AgentStrategy, AgentTask, HermesAgent, HermesConfig, HermesStats,
    StrategyStep, TaskStatus, TaskType,
};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn hermes_create_task(
    agent: State<'_, Arc<HermesAgent>>,
    task_type: String,
    description: String,
    priority: u32,
    metadata: HashMap<String, String>,
) -> Result<String, String> {
    let task_type_enum = match task_type.as_str() {
        "Navigation" => TaskType::Navigation,
        "FormFill" => TaskType::FormFill,
        "DataExtraction" => TaskType::DataExtraction,
        "Analysis" => TaskType::Analysis,
        "Automation" => TaskType::Automation,
        custom => TaskType::Custom(custom.to_string()),
    };

    agent.create_task(task_type_enum, description, priority, metadata).await
}

#[tauri::command]
pub async fn hermes_execute_task(
    agent: State<'_, Arc<HermesAgent>>,
    task_id: String,
) -> Result<serde_json::Value, String> {
    agent.execute_task(&task_id).await
}

#[tauri::command]
pub async fn hermes_get_task(
    agent: State<'_, Arc<HermesAgent>>,
    task_id: String,
) -> Result<Option<AgentTask>, String> {
    Ok(agent.get_task(&task_id).await)
}

#[tauri::command]
pub async fn hermes_get_all_tasks(
    agent: State<'_, Arc<HermesAgent>>,
) -> Result<Vec<AgentTask>, String> {
    Ok(agent.get_all_tasks().await)
}

#[tauri::command]
pub async fn hermes_cancel_task(
    agent: State<'_, Arc<HermesAgent>>,
    task_id: String,
) -> Result<(), String> {
    agent.cancel_task(&task_id).await
}

#[tauri::command]
pub async fn hermes_create_strategy(
    agent: State<'_, Arc<HermesAgent>>,
    name: String,
    description: String,
    steps: Vec<StrategyStep>,
) -> Result<String, String> {
    agent.create_strategy(name, description, steps).await
}

#[tauri::command]
pub async fn hermes_execute_strategy(
    agent: State<'_, Arc<HermesAgent>>,
    strategy_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    agent.execute_strategy(&strategy_id).await
}

#[tauri::command]
pub async fn hermes_update_context(
    agent: State<'_, Arc<HermesAgent>>,
    context: AgentContext,
) -> Result<(), String> {
    agent.update_context(context).await;
    Ok(())
}

#[tauri::command]
pub async fn hermes_get_context(
    agent: State<'_, Arc<HermesAgent>>,
) -> Result<AgentContext, String> {
    Ok(agent.get_context().await)
}

#[tauri::command]
pub async fn hermes_get_stats(
    agent: State<'_, Arc<HermesAgent>>,
) -> Result<HermesStats, String> {
    Ok(agent.get_stats().await)
}

#[tauri::command]
pub async fn hermes_update_config(
    agent: State<'_, Arc<HermesAgent>>,
    config: HermesConfig,
) -> Result<(), String> {
    agent.update_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn hermes_get_config(
    agent: State<'_, Arc<HermesAgent>>,
) -> Result<HermesConfig, String> {
    Ok(agent.get_config().await)
}
