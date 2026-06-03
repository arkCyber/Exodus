//! In-process Contact Directory hub — avoids Unix socket RPC failures.

use crate::microservice::contact_directory_service::{
    ContactDirectoryService, ContactDirectoryServiceConfig,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Central hub wrapping one `ContactDirectoryService` instance in the app process.
pub struct ContactDirectoryHub {
    pub service: Arc<ContactDirectoryService>,
    pub storage_dir: PathBuf,
}

/// Managed Tauri state for the contact directory hub.
pub struct ContactDirectoryState {
    pub hub: Arc<ContactDirectoryHub>,
}

/// Hub metadata returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDirectoryHubInfo {
    pub storage_dir: String,
    pub node_id: String,
    pub in_process: bool,
}

impl ContactDirectoryHub {
    /// Bootstrap hub under `{app_data}/contact_directory/`.
    pub fn bootstrap(app_data_dir: &Path, local_node_id: Option<String>) -> Result<Arc<Self>, String> {
        let config =
            ContactDirectoryServiceConfig::under_app_data_with_node(app_data_dir, local_node_id);
        let storage_dir = config.storage_dir.clone();
        let service = Arc::new(
            ContactDirectoryService::new(config)
                .map_err(|e| format!("Contact directory init failed: {e}"))?,
        );
        Ok(Arc::new(Self { service, storage_dir }))
    }

    /// Summary for UI / diagnostics.
    pub fn info(&self) -> ContactDirectoryHubInfo {
        ContactDirectoryHubInfo {
            storage_dir: self.storage_dir.display().to_string(),
            node_id: self.service.node_id().to_string(),
            in_process: true,
        }
    }
}

/// Ensure the in-process hub is registered on the app handle (idempotent).
pub fn ensure_contact_directory_hub(
    app: &AppHandle,
    app_data_dir: &Path,
    local_node_id: Option<String>,
) -> Result<(), String> {
    if app.try_state::<ContactDirectoryState>().is_some() {
        return Ok(());
    }
    let hub = ContactDirectoryHub::bootstrap(app_data_dir, local_node_id)?;
    app.manage(ContactDirectoryState { hub });
    tracing::info!(
        "Contact directory hub ready (in-process) at {}",
        app_data_dir.display()
    );
    Ok(())
}

/// Resolve managed hub from app state.
pub fn hub(app: &AppHandle) -> Result<Arc<ContactDirectoryHub>, String> {
    Ok(app
        .try_state::<ContactDirectoryState>()
        .ok_or_else(|| {
            "Contact directory hub not started — open Contacts or call contact_directory_service_start"
                .to_string()
        })?
        .hub
        .clone())
}

#[cfg(all(test, feature = "im-tests"))]
mod tests {
    use super::*;
    use crate::microservice::Contact;

    #[test]
    fn hub_add_list_in_process() {
        let dir = std::env::temp_dir().join(format!("exodus_contact_hub_test_{}", uuid::Uuid::new_v4()));
        let hub = ContactDirectoryHub::bootstrap(&dir, Some("node-hub-001".into())).expect("bootstrap");
        let c = Contact {
            contact_id: "hub-c1".into(),
            name: "Hub Friend".into(),
            contact_type: "human".into(),
            agent_deployment_type: None,
            agent_ids: vec![],
            node_id: "node-hub-001".into(),
            groups: vec!["friends".into()],
            tags: vec![],
            notes: String::new(),
            is_favorite: false,
            is_blocked: false,
            created_at: 1,
            last_contacted: 0,
            contact_count: 0,
            public_account_id: None,
            iot_device_type: None,
            iot_protocol: None,
            iot_status: None,
            iot_last_seen: None,
            iot_capabilities: None,
            iot_location: None,
        };
        hub.service.add_contact(c).expect("add");
        let list = hub.service.list_contacts();
        assert!(list.iter().any(|x| x.name == "Hub Friend"));
        assert!(dir.join("directory.json").is_file());
    }

    #[test]
    fn hub_persists_across_restart() {
        let dir = std::env::temp_dir().join(format!("exodus_contact_persist_{}", uuid::Uuid::new_v4()));
        {
            let hub = ContactDirectoryHub::bootstrap(&dir, None).expect("bootstrap");
            let c = Contact {
                contact_id: "persist-1".into(),
                name: "Persisted".into(),
                contact_type: "human".into(),
                agent_deployment_type: None,
                agent_ids: vec![],
                node_id: "node-persist".into(),
                groups: vec![],
                tags: vec![],
                notes: String::new(),
                is_favorite: false,
                is_blocked: false,
                created_at: 1,
                last_contacted: 0,
                contact_count: 0,
                public_account_id: None,
                iot_device_type: None,
                iot_protocol: None,
                iot_status: None,
                iot_last_seen: None,
                iot_capabilities: None,
                iot_location: None,
            };
            hub.service.add_contact(c).expect("add");
        }
        let hub2 = ContactDirectoryHub::bootstrap(&dir, None).expect("reload");
        assert!(hub2
            .service
            .list_contacts()
            .iter()
            .any(|x| x.name == "Persisted"));
    }
}
