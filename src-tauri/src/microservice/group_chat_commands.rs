//! Tauri commands for Group Chat Service
//! 
//! These commands allow the frontend to interact with the Group Chat Service
//! via JSON-RPC over Unix Domain Sockets. Includes both group chat and 1-to-1 direct messaging.

use crate::microservice::group_chat_client::group_chat_json_rpc;
use crate::microservice::{GroupChatService, GroupChatServiceConfig, GroupChat, GroupMessage, GroupMember, GroupInvitation, DirectChat, DirectMessage, MessageReceipt, UserSequence};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Managed Group Chat Service instance
pub struct ManagedGroupChatService {
    service: Arc<GroupChatService>,
    running: Arc<Mutex<bool>>,
}

impl ManagedGroupChatService {
    pub fn new(service: GroupChatService) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                if *running {
                    return Ok(());
                }
                *running = true;
            }
        }
        
        self.service.start().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), String> {
        {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
        }
        
        self.service.stop().await.map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.running.lock()
            .map(|running| *running)
            .unwrap_or(false)
    }
}

/// Send JSON-RPC request to Group Chat Service.
async fn send_group_chat_request(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    group_chat_json_rpc(socket_path, method, params).await
}

/// Start the Group Chat Service
#[tauri::command]
pub async fn group_chat_service_start(
    app: AppHandle,
) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let socket_path = config.socket_path.clone();
    
    let service = GroupChatService::new(config)
        .map_err(|e| format!("Failed to create Group Chat Service: {}", e))?;
    
    let managed = ManagedGroupChatService::new(service);
    managed.start().await?;
    
    let _ = app.emit("group-chat-service-started", json!({
        "socket_path": socket_path.to_string_lossy().to_string()
    }));
    
    Ok(())
}

/// Stop the Group Chat Service
#[tauri::command]
pub async fn group_chat_service_stop() -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let service = GroupChatService::new(config)
        .map_err(|e| format!("Failed to create Group Chat Service: {}", e))?;
    service.stop().await.map_err(|e| e.to_string())
}

/// Create a new group
#[tauri::command]
pub async fn group_create(group: GroupChat) -> Result<String, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!(group);
    let result = send_group_chat_request(&config.socket_path, "create_group", params).await?;
    
    result.get("group_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid group_id response".to_string())
}

/// Update group
#[tauri::command]
pub async fn group_update(group: GroupChat) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!(group);
    send_group_chat_request(&config.socket_path, "update_group", params).await?;
    Ok(())
}

/// Delete group
#[tauri::command]
pub async fn group_delete(group_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "group_id": group_id });
    send_group_chat_request(&config.socket_path, "delete_group", params).await?;
    Ok(())
}

/// Get group
#[tauri::command]
pub async fn group_get(group_id: String) -> Result<GroupChat, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "group_id": group_id });
    let result = send_group_chat_request(&config.socket_path, "get_group", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse group: {}", e))
}

/// List user's groups
#[tauri::command]
pub async fn group_list_user(user_id: String) -> Result<Vec<GroupChat>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_group_chat_request(&config.socket_path, "list_user_groups", params).await?;
    
    let groups = result.get("groups")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid groups response".to_string())?;
    
    groups.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Add member to group
#[tauri::command]
pub async fn group_add_member(group_id: String, member: GroupMember) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "member": member
    });
    send_group_chat_request(&config.socket_path, "add_member", params).await?;
    Ok(())
}

/// Remove member from group
#[tauri::command]
pub async fn group_remove_member(group_id: String, agent_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "agent_id": agent_id
    });
    send_group_chat_request(&config.socket_path, "remove_member", params).await?;
    Ok(())
}

/// Get group members
#[tauri::command]
pub async fn group_get_members(group_id: String) -> Result<Vec<GroupMember>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "group_id": group_id });
    let result = send_group_chat_request(&config.socket_path, "get_members", params).await?;
    
    let members = result.get("members")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid members response".to_string())?;
    
    members.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Send message to group
#[tauri::command]
pub async fn group_send_message(message: GroupMessage) -> Result<String, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!(message);
    let result = send_group_chat_request(&config.socket_path, "send_message", params).await?;
    
    result.get("message_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid message_id response".to_string())
}

/// Get group messages
#[tauri::command]
pub async fn group_get_messages(group_id: String, limit: Option<usize>, before: Option<u64>) -> Result<Vec<GroupMessage>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "limit": limit,
        "before": before
    });
    let result = send_group_chat_request(&config.socket_path, "get_messages", params).await?;
    
    let messages = result.get("messages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid messages response".to_string())?;
    
    messages.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Edit message
#[tauri::command]
pub async fn group_edit_message(group_id: String, message_id: String, new_content: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "message_id": message_id,
        "new_content": new_content
    });
    send_group_chat_request(&config.socket_path, "edit_message", params).await?;
    Ok(())
}

/// Delete message
#[tauri::command]
pub async fn group_delete_message(group_id: String, message_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "message_id": message_id
    });
    send_group_chat_request(&config.socket_path, "delete_message", params).await?;
    Ok(())
}

/// Create invitation
#[tauri::command]
pub async fn group_create_invitation(invitation: GroupInvitation) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!(invitation);
    send_group_chat_request(&config.socket_path, "create_invitation", params).await?;
    Ok(())
}

/// Accept invitation
#[tauri::command]
pub async fn group_accept_invitation(invitation_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "invitation_id": invitation_id });
    send_group_chat_request(&config.socket_path, "accept_invitation", params).await?;
    Ok(())
}

/// Reject invitation
#[tauri::command]
pub async fn group_reject_invitation(invitation_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "invitation_id": invitation_id });
    send_group_chat_request(&config.socket_path, "reject_invitation", params).await?;
    Ok(())
}

/// Get pending invitations
#[tauri::command]
pub async fn group_get_pending_invitations(user_id: String) -> Result<Vec<GroupInvitation>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_group_chat_request(&config.socket_path, "get_pending_invitations", params).await?;
    
    let invitations = result.get("invitations")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid invitations response".to_string())?;
    
    invitations.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update member online status
#[tauri::command]
pub async fn group_update_member_online(group_id: String, agent_id: String, is_online: bool) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "agent_id": agent_id,
        "is_online": is_online
    });
    send_group_chat_request(&config.socket_path, "update_member_online", params).await?;
    Ok(())
}

/// Search groups
#[tauri::command]
pub async fn group_search(query: String, limit: Option<usize>) -> Result<Vec<GroupChat>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "query": query,
        "limit": limit
    });
    let result = send_group_chat_request(&config.socket_path, "search_groups", params).await?;
    
    let groups = result.get("groups")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid groups response".to_string())?;
    
    groups.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Link group to public account
#[tauri::command]
pub async fn group_link_to_public_account(group_id: String, account_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "account_id": account_id
    });
    let result = send_group_chat_request(&config.socket_path, "link_to_public_account", params).await?;
    
    result.get("linked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid linked response".to_string())
}

/// Unlink group from public account
#[tauri::command]
pub async fn group_unlink_from_public_account(group_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "group_id": group_id });
    let result = send_group_chat_request(&config.socket_path, "unlink_from_public_account", params).await?;
    
    result.get("unlinked")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid unlinked response".to_string())
}

/// Get groups linked to a public account
#[tauri::command]
pub async fn group_get_by_public_account(account_id: String) -> Result<Vec<GroupChat>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "account_id": account_id });
    let result = send_group_chat_request(&config.socket_path, "get_groups_by_public_account", params).await?;
    
    let groups = result.get("groups")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid groups response".to_string())?;
    
    groups.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Add admin to group
#[tauri::command]
pub async fn group_add_admin(group_id: String, user_id: String, requester_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "user_id": user_id,
        "requester_id": requester_id
    });
    let result = send_group_chat_request(&config.socket_path, "add_admin", params).await?;
    
    result.get("added")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid added response".to_string())
}

/// Remove admin from group
#[tauri::command]
pub async fn group_remove_admin(group_id: String, user_id: String, requester_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "user_id": user_id,
        "requester_id": requester_id
    });
    let result = send_group_chat_request(&config.socket_path, "remove_admin", params).await?;
    
    result.get("removed")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid removed response".to_string())
}

/// Check if user is admin
#[tauri::command]
pub async fn group_is_admin(group_id: String, user_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "user_id": user_id
    });
    let result = send_group_chat_request(&config.socket_path, "is_admin", params).await?;
    
    result.get("is_admin")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid is_admin response".to_string())
}

/// Check if user is owner
#[tauri::command]
pub async fn group_is_owner(group_id: String, user_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "user_id": user_id
    });
    let result = send_group_chat_request(&config.socket_path, "is_owner", params).await?;
    
    result.get("is_owner")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid is_owner response".to_string())
}

/// Check if user has permission
#[tauri::command]
pub async fn group_has_permission(group_id: String, user_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "group_id": group_id,
        "user_id": user_id
    });
    let result = send_group_chat_request(&config.socket_path, "has_permission", params).await?;
    
    result.get("has_permission")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid has_permission response".to_string())
}

// ========== 1-to-1 Direct Chat Commands ==========

/// Create or get a direct chat between two users
#[tauri::command]
pub async fn direct_chat_create_or_get(user_a: String, user_b: String) -> Result<DirectChat, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "user_a": user_a,
        "user_b": user_b
    });
    let result = send_group_chat_request(&config.socket_path, "direct_chat_create_or_get", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse chat: {}", e))
}

/// Send direct message
#[tauri::command]
pub async fn direct_send_message(message: DirectMessage) -> Result<String, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!(message);
    let result = send_group_chat_request(&config.socket_path, "direct_send_message", params).await?;
    
    result.get("message_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Invalid message_id response".to_string())
}

/// Get direct messages
#[tauri::command]
pub async fn direct_get_messages(chat_id: String, limit: Option<usize>, before: Option<u64>) -> Result<Vec<DirectMessage>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "chat_id": chat_id,
        "limit": limit,
        "before": before
    });
    let result = send_group_chat_request(&config.socket_path, "direct_get_messages", params).await?;
    
    let messages = result.get("messages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid messages response".to_string())?;
    
    messages.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Edit direct message
#[tauri::command]
pub async fn direct_edit_message(chat_id: String, message_id: String, new_content: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "chat_id": chat_id,
        "message_id": message_id,
        "new_content": new_content
    });
    send_group_chat_request(&config.socket_path, "direct_edit_message", params).await?;
    Ok(())
}

/// Delete direct message
#[tauri::command]
pub async fn direct_delete_message(chat_id: String, message_id: String) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "chat_id": chat_id,
        "message_id": message_id
    });
    send_group_chat_request(&config.socket_path, "direct_delete_message", params).await?;
    Ok(())
}

/// Get direct chat
#[tauri::command]
pub async fn direct_get_chat(chat_id: String) -> Result<DirectChat, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "chat_id": chat_id });
    let result = send_group_chat_request(&config.socket_path, "direct_get_chat", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse chat: {}", e))
}

/// List direct chats for a user
#[tauri::command]
pub async fn direct_list_chats(user_id: String) -> Result<Vec<DirectChat>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "user_id": user_id });
    let result = send_group_chat_request(&config.socket_path, "direct_list_chats", params).await?;
    
    let chats = result.get("chats")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid chats response".to_string())?;
    
    chats.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Update user sequence tracking
#[tauri::command]
pub async fn direct_update_sequence(user_id: String, sender_id: String, last_sequence: u32) -> Result<(), String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "sender_id": sender_id,
        "last_sequence": last_sequence
    });
    send_group_chat_request(&config.socket_path, "direct_update_sequence", params).await?;
    Ok(())
}

/// Get user sequence tracking
#[tauri::command]
pub async fn direct_get_sequence(user_id: String, sender_id: String) -> Result<UserSequence, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "sender_id": sender_id
    });
    let result = send_group_chat_request(&config.socket_path, "direct_get_sequence", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse sequence: {}", e))
}

/// Detect missing messages
#[tauri::command]
pub async fn direct_detect_missing(user_id: String, sender_id: String) -> Result<serde_json::Value, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "user_id": user_id,
        "sender_id": sender_id
    });
    send_group_chat_request(&config.socket_path, "direct_detect_missing", params).await
}

/// Get messages by sequence range
#[tauri::command]
pub async fn direct_get_messages_by_sequence(sender_id: String, start_seq: u32, end_seq: u32) -> Result<Vec<DirectMessage>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "sender_id": sender_id,
        "start_sequence": start_seq,
        "end_sequence": end_seq
    });
    let result = send_group_chat_request(&config.socket_path, "direct_get_messages_by_sequence", params).await?;
    
    let messages = result.get("messages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid messages response".to_string())?;
    
    messages.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Create message receipt
#[tauri::command]
pub async fn direct_create_receipt(message_id: String, receiver_id: String, sequence: u32) -> Result<MessageReceipt, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "message_id": message_id,
        "receiver_id": receiver_id,
        "sequence": sequence
    });
    let result = send_group_chat_request(&config.socket_path, "direct_create_receipt", params).await?;
    
    serde_json::from_value(result).map_err(|e| format!("Failed to parse receipt: {}", e))
}

/// Get receipts for a message
#[tauri::command]
pub async fn direct_get_receipts(message_id: String) -> Result<Vec<MessageReceipt>, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ "message_id": message_id });
    let result = send_group_chat_request(&config.socket_path, "direct_get_receipts", params).await?;
    
    let receipts = result.get("receipts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Invalid receipts response".to_string())?;
    
    receipts.iter()
        .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
        .collect()
}

/// Verify message integrity
#[tauri::command]
pub async fn direct_verify_message(chat_id: String, message_id: String) -> Result<bool, String> {
    let config = GroupChatServiceConfig::default();
    let params = json!({ 
        "chat_id": chat_id,
        "message_id": message_id
    });
    let result = send_group_chat_request(&config.socket_path, "direct_verify_message", params).await?;
    
    result.get("valid")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| "Invalid valid response".to_string())
}
