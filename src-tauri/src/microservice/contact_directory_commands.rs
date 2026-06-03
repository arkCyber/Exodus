//! Tauri commands for Contact Directory — in-process hub (no Unix socket RPC).

pub use crate::microservice::contact_directory_hub::ensure_contact_directory_hub;
use crate::microservice::contact_directory_hub::{hub, ContactDirectoryHubInfo};
use crate::microservice::{Contact, ContactGroup};
use crate::p2p_cdn::P2pCdnState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| e.to_string())
}

fn ensure_hub(app: &AppHandle) -> Result<(), String> {
    let dir = app_data_dir(app)?;
    let local_node = app
        .try_state::<Arc<P2pCdnState>>()
        .map(|c| c.node_id.clone());
    ensure_contact_directory_hub(app, &dir, local_node)
}

/// Start / ensure the in-process contact directory hub.
#[tauri::command]
pub async fn contact_directory_service_start(app: AppHandle) -> Result<ContactDirectoryHubInfo, String> {
    ensure_hub(&app)?;
    let info = hub(&app)?.info();
    let _ = app.emit("contact-directory-service-started", json!({
        "storage_dir": info.storage_dir,
        "node_id": info.node_id,
        "in_process": true,
    }));
    Ok(info)
}

/// Stop optional UDS listener (hub data remains in memory until app exit).
#[tauri::command]
pub async fn contact_directory_service_stop(app: AppHandle) -> Result<(), String> {
    if let Ok(h) = hub(&app) {
        h.service.stop().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Hub diagnostics for the UI.
#[tauri::command]
pub async fn contact_directory_hub_info(app: AppHandle) -> Result<ContactDirectoryHubInfo, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.info())
}

/// 12-digit Exodus ID for this device (auto-registered in the directory).
#[tauri::command]
pub async fn contact_get_local_digit(app: AppHandle) -> Result<String, String> {
    ensure_hub(&app)?;
    let h = hub(&app)?;
    h.service
        .ensure_digit_for_node(&h.service.node_id().to_string())
}

/// JSON export bundle for backup / transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactExportBundle {
    pub version: u32,
    pub exported_at: u64,
    pub contacts: Vec<Contact>,
    pub groups: Vec<ContactGroup>,
}

#[tauri::command]
pub async fn contact_export_json(app: AppHandle) -> Result<String, String> {
    ensure_hub(&app)?;
    let h = hub(&app)?;
    let bundle = ContactExportBundle {
        version: 1,
        exported_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        contacts: h.service.list_contacts(),
        groups: h.service.list_groups(),
    };
    serde_json::to_string_pretty(&bundle).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn contact_import_json(
    app: AppHandle,
    json: String,
    merge: Option<bool>,
) -> Result<usize, String> {
    ensure_hub(&app)?;
    let bundle: ContactExportBundle =
        serde_json::from_str(&json).map_err(|e| format!("Invalid import JSON: {e}"))?;
    hub(&app)?.service.import_bundle(
        bundle.contacts,
        bundle.groups,
        merge.unwrap_or(true),
    )
}

#[tauri::command]
pub async fn contact_add(app: AppHandle, contact: Contact) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.add_contact(contact)
}

#[tauri::command]
pub async fn contact_remove(app: AppHandle, contact_id: String) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.remove_contact(contact_id)
}

#[tauri::command]
pub async fn contact_update(app: AppHandle, contact: Contact) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.update_contact(contact)
}

#[tauri::command]
pub async fn contact_get(app: AppHandle, contact_id: String) -> Result<Contact, String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .get_contact(contact_id)
        .ok_or_else(|| "Contact not found".to_string())
}

#[tauri::command]
pub async fn contact_list(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.list_contacts())
}

#[tauri::command]
pub async fn contact_search(app: AppHandle, query: String) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.search_contacts(query))
}

#[tauri::command]
pub async fn contact_get_by_group(app: AppHandle, group_id: String) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_contacts_by_group(group_id))
}

#[tauri::command]
pub async fn contact_get_favorites(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_favorites())
}

#[tauri::command]
pub async fn contact_get_blocked(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_blocked())
}

#[tauri::command]
pub async fn contact_add_to_group(
    app: AppHandle,
    contact_id: String,
    group_id: String,
) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.add_to_group(contact_id, group_id)
}

#[tauri::command]
pub async fn contact_remove_from_group(
    app: AppHandle,
    contact_id: String,
    group_id: String,
) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.remove_from_group(contact_id, group_id)
}

#[tauri::command]
pub async fn contact_group_create(app: AppHandle, group: ContactGroup) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.create_group(group)
}

#[tauri::command]
pub async fn contact_group_delete(app: AppHandle, group_id: String) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.delete_group(group_id)
}

#[tauri::command]
pub async fn contact_group_list(app: AppHandle) -> Result<Vec<ContactGroup>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.list_groups())
}

#[tauri::command]
pub async fn contact_toggle_favorite(app: AppHandle, contact_id: String) -> Result<bool, String> {
    ensure_hub(&app)?;
    hub(&app)?.service.toggle_favorite(contact_id)
}

#[tauri::command]
pub async fn contact_block(app: AppHandle, contact_id: String) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.block_contact(contact_id)
}

#[tauri::command]
pub async fn contact_unblock(app: AppHandle, contact_id: String) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.unblock_contact(contact_id)
}

#[tauri::command]
pub async fn contact_get_recent(
    app: AppHandle,
    limit: Option<usize>,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_recent_contacts(limit.unwrap_or(20)))
}

#[tauri::command]
pub async fn contact_get_by_node(app: AppHandle, node_id: String) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_contacts_by_node(node_id))
}

#[tauri::command]
pub async fn contact_get_by_agent(
    app: AppHandle,
    agent_id: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_contacts_by_agent(agent_id))
}

#[tauri::command]
pub async fn contact_register_digit_mapping(
    app: AppHandle,
    digit_id: String,
    node_id: String,
) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?.service.register_digit_mapping(digit_id, node_id)
}

#[tauri::command]
pub async fn contact_resolve_digit_to_node(
    app: AppHandle,
    digit_id: String,
) -> Result<String, String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .resolve_digit_to_node(digit_id)
        .ok_or_else(|| "12-digit ID not found".to_string())
}

#[tauri::command]
pub async fn contact_get_digit_for_node(app: AppHandle, node_id: String) -> Result<String, String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .get_digit_for_node(node_id)
        .ok_or_else(|| "No digit mapping for node".to_string())
}

#[tauri::command]
pub async fn contact_add_friend_by_digit(
    app: AppHandle,
    digit_id: String,
    name: String,
    user_id: String,
) -> Result<Contact, String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .add_friend_by_digit(digit_id, name, user_id)
}

#[tauri::command]
pub async fn contact_filter_by_type(
    app: AppHandle,
    contact_type: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.filter_contacts_by_type(contact_type))
}

#[tauri::command]
pub async fn contact_filter_by_deployment_type(
    app: AppHandle,
    deployment_type: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?
        .service
        .filter_contacts_by_deployment_type(deployment_type))
}

#[tauri::command]
pub async fn contact_link_to_public_account(
    app: AppHandle,
    contact_id: String,
    account_id: String,
) -> Result<bool, String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .link_to_public_account(contact_id, account_id)?;
    Ok(true)
}

#[tauri::command]
pub async fn contact_set_friend_request_mode(
    app: AppHandle,
    user_id: String,
    mode: String,
) -> Result<bool, String> {
    ensure_hub(&app)?;
    hub(&app)?.service.set_friend_request_mode(user_id, mode)?;
    Ok(true)
}

#[tauri::command]
pub async fn contact_get_friend_request_mode(
    app: AppHandle,
    user_id: String,
) -> Result<String, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_friend_request_mode(user_id))
}

#[tauri::command]
pub async fn contact_unlink_from_public_account(
    app: AppHandle,
    contact_id: String,
) -> Result<bool, String> {
    ensure_hub(&app)?;
    hub(&app)?.service.unlink_from_public_account(contact_id)?;
    Ok(true)
}

#[tauri::command]
pub async fn contact_get_contacts_by_public_account(
    app: AppHandle,
    account_id: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?
        .service
        .get_contacts_by_public_account(account_id))
}

// IoT device commands

#[tauri::command]
pub async fn contact_filter_by_iot_device_type(
    app: AppHandle,
    device_type: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.filter_contacts_by_iot_device_type(device_type))
}

#[tauri::command]
pub async fn contact_filter_by_iot_protocol(
    app: AppHandle,
    protocol: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.filter_contacts_by_iot_protocol(protocol))
}

#[tauri::command]
pub async fn contact_filter_by_iot_status(
    app: AppHandle,
    status: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.filter_contacts_by_iot_status(status))
}

#[tauri::command]
pub async fn contact_get_iot_devices_by_location(
    app: AppHandle,
    location: String,
) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_iot_devices_by_location(location))
}

#[tauri::command]
pub async fn contact_get_all_iot_devices(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_all_iot_devices())
}

#[tauri::command]
pub async fn contact_update_iot_device_status(
    app: AppHandle,
    contact_id: String,
    status: String,
) -> Result<(), String> {
    ensure_hub(&app)?;
    hub(&app)?
        .service
        .update_iot_device_status(contact_id, status)
}

#[tauri::command]
pub async fn contact_get_online_iot_devices(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_online_iot_devices())
}

#[tauri::command]
pub async fn contact_get_offline_iot_devices(app: AppHandle) -> Result<Vec<Contact>, String> {
    ensure_hub(&app)?;
    Ok(hub(&app)?.service.get_offline_iot_devices())
}
