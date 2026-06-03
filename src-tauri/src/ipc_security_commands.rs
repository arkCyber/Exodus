//! Exodus Browser — IPC Security Tauri commands
//!
//! Provides Tauri commands for IPC security management

use tauri::State;

use super::ipc_security::{IpcMessage, IpcSecurityManager, SecurityPolicy, IpcSecurityStats};

/// Validate an IPC message
#[tauri::command]
pub fn ipc_validate_message(
    message: IpcMessage,
    manager: State<'_, IpcSecurityManager>,
) -> Result<(), String> {
    manager.validate_message(&message)
        .map_err(|e| e.to_string())
}

/// Process an IPC message (validate and sanitize)
#[tauri::command]
pub fn ipc_process_message(
    message: IpcMessage,
    manager: State<'_, IpcSecurityManager>,
) -> Result<IpcMessage, String> {
    manager.process_message(message)
        .map_err(|e| e.to_string())
}

/// Get message history
#[tauri::command]
pub fn ipc_get_message_history(
    limit: usize,
    manager: State<'_, IpcSecurityManager>,
) -> Vec<IpcMessage> {
    manager.get_message_history(limit)
}

/// Get security policy
#[tauri::command]
pub fn ipc_get_policy(
    manager: State<'_, IpcSecurityManager>,
) -> SecurityPolicy {
    manager.get_policy()
}

/// Set security policy
#[tauri::command]
pub fn ipc_set_policy(
    policy: SecurityPolicy,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.set_policy(policy);
}

/// Add allowed command
#[tauri::command]
pub fn ipc_add_allowed_command(
    command: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.add_allowed_command(command);
}

/// Remove allowed command
#[tauri::command]
pub fn ipc_remove_allowed_command(
    command: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.remove_allowed_command(command);
}

/// Add blocked command
#[tauri::command]
pub fn ipc_add_blocked_command(
    command: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.add_blocked_command(command);
}

/// Remove blocked command
#[tauri::command]
pub fn ipc_remove_blocked_command(
    command: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.remove_blocked_command(command);
}

/// Add allowed source
#[tauri::command]
pub fn ipc_add_allowed_source(
    source: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.add_allowed_source(source);
}

/// Remove allowed source
#[tauri::command]
pub fn ipc_remove_allowed_source(
    source: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.remove_allowed_source(source);
}

/// Add blocked source
#[tauri::command]
pub fn ipc_add_blocked_source(
    source: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.add_blocked_source(source);
}

/// Remove blocked source
#[tauri::command]
pub fn ipc_remove_blocked_source(
    source: String,
    manager: State<'_, IpcSecurityManager>,
) {
    manager.remove_blocked_source(source);
}

/// Clear message history
#[tauri::command]
pub fn ipc_clear_history(
    manager: State<'_, IpcSecurityManager>,
) {
    manager.clear_history();
}

/// Get security statistics
#[tauri::command]
pub fn ipc_get_stats(
    manager: State<'_, IpcSecurityManager>,
) -> IpcSecurityStats {
    manager.get_stats()
}
