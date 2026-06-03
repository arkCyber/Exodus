//! Bridge between manifest-level `plugins::permissions` and `extension_permissions` store.

use std::sync::Arc;

// use crate::extension_permissions::{ExtensionPermissionsManager, PermissionType};

use super::manager::ExtensionManager;
use super::permissions::Permission;

/// Map a manifest permission to the extension-permissions enum (when supported).
pub fn plugin_permission_to_type(_permission: Permission) -> Option<()> {
    None
}

// /// Sync manifest grants from `ExtensionManager` into `ExtensionPermissionsManager`.
// pub fn sync_extension_grants(
//     perm_mgr: &ExtensionPermissionsManager,
//     extension_id: &str,
//     perms: &[Permission],
// ) -> Result<(), String> {
//     for permission in perms {
//         if let Some(_permission_type) = plugin_permission_to_type(*permission) {
//             perm_mgr
//                 .grant_permission(extension_id.to_string(), _permission_type)
//                 .map_err(|e| e.to_string())?;
//         }
//     }
//     Ok(())
// }

// /// Revoke all extension permissions in the parallel store (uninstall).
// pub fn sync_extension_revoke_all(
//     perm_mgr: &ExtensionPermissionsManager,
//     extension_id: &str,
// ) -> Result<(), String> {
//     perm_mgr
//         .revoke_all_permissions(extension_id)
//         .map_err(|e| e.to_string())
// }

// /// Sync every loaded extension from the manager into the permissions store.
// pub fn sync_all_from_extension_manager(
//     ext_mgr: &ExtensionManager,
//     perm_mgr: &Arc<ExtensionPermissionsManager>,
// ) {
//     for ext in ext_mgr.list() {
//         if let Ok(perms) = ext_mgr.permissions_for(&ext.id) {
//             let _ = sync_extension_grants(perm_mgr.as_ref(), &ext.id, &perms);
//         }
//         for host in ext_mgr.host_permissions_for(&ext.id) {
//             let _ = perm_mgr.add_host_permission(ext.id.clone(), host);
//         }
//     }
// }
