//! Exodus Hub Server
//!
//! HTTP/WebSocket API for message storage, group management, WeChat official accounts, video conferencing, and P2P coordination.

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use futures_util::{stream::StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, debug, warn};

use crate::manager::{ImManager, ChatType, Message as DbMessage, SyncResponse};
use crate::p2p::{P2PManager, SyncConfig, NodeInfo};

/// Server state
#[derive(Clone)]
pub struct ImServerState {
    im_manager: Arc<ImManager>,
    clients: Arc<RwLock<HashMap<String, broadcast::Sender<ServerMessage>>>>,
    online_users: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    p2p: Option<Arc<P2PManager>>,
}

/// Server message sent to clients via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServerMessage {
    #[serde(rename = "new_message")]
    NewMessage {
        conversation_id: String,
        sender_id: String,
        content: String,
        timestamp: DateTime<Utc>,
        sequence: Option<u32>,
        integrity_hash: Option<String>,
    },
    #[serde(rename = "message_receipt")]
    MessageReceipt {
        message_id: String,
        receiver_id: String,
        sequence: u32,
        received_at: DateTime<Utc>,
    },
    #[serde(rename = "user_online")]
    UserOnline {
        user_id: String,
        username: String,
    },
    #[serde(rename = "user_offline")]
    UserOffline {
        user_id: String,
    },
    #[serde(rename = "typing")]
    Typing {
        conversation_id: String,
        user_id: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

/// Client message received from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "heartbeat")]
    Heartbeat,
    #[serde(rename = "typing")]
    Typing {
        conversation_id: String,
    },
    #[serde(rename = "subscribe")]
    Subscribe {
        conversation_id: String,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        conversation_id: String,
    },
    #[serde(rename = "message_receipt")]
    MessageReceipt {
        message_id: String,
        sequence: u32,
    },
}

/// HTTP API: Send message
#[derive(Debug, Deserialize)]
struct SendMessageRequest {
    conversation_id: String,
    sender_id: String,
    receiver_id: Option<String>,
    content: String,
    reply_to: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendMessageResponse {
    message_id: String,
    sequence: Option<u32>,
    timestamp: DateTime<Utc>,
}

/// HTTP API: Get messages
#[derive(Debug, Deserialize)]
struct GetMessagesQuery {
    conversation_id: String,
    limit: Option<u32>,
    after_sequence: Option<u32>,
}

/// HTTP API: Register user
#[derive(Debug, Deserialize)]
struct RegisterUserRequest {
    username: String,
    display_name: String,
}

#[derive(Debug, Serialize)]
struct RegisterUserResponse {
    user_id: String,
}

/// HTTP API: Get offline messages
#[derive(Debug, Deserialize)]
struct GetOfflineMessagesQuery {
    user_id: String,
    conversation_id: Option<String>,
    limit: Option<u32>,
}

impl ImServerState {
    /// Create a new server state
    pub fn new(im_manager: Arc<ImManager>) -> Self {
        Self {
            im_manager,
            clients: Arc::new(RwLock::new(HashMap::new())),
            online_users: Arc::new(RwLock::new(HashMap::new())),
            p2p: None,
        }
    }

    /// Create a new server state with P2P support
    pub fn with_p2p(im_manager: Arc<ImManager>, p2p: Arc<P2PManager>) -> Self {
        Self {
            im_manager,
            clients: Arc::new(RwLock::new(HashMap::new())),
            online_users: Arc::new(RwLock::new(HashMap::new())),
            p2p: Some(p2p),
        }
    }

    /// Get IM manager
    pub fn im_manager(&self) -> &Arc<ImManager> {
        &self.im_manager
    }

    /// Get P2P manager
    pub fn p2p(&self) -> Option<&Arc<P2PManager>> {
        self.p2p.as_ref()
    }

    pub async fn is_user_online(&self, user_id: &str) -> bool {
        let online_users = self.online_users.read().await;
        online_users.contains_key(user_id)
    }

    pub async fn get_online_users(&self) -> Vec<String> {
        let online_users = self.online_users.read().await;
        online_users.keys().cloned().collect()
    }

    async fn broadcast_to_conversation(
        &self,
        conversation_id: &str,
        message: ServerMessage,
    ) -> Result<()> {
        let members = self
            .im_manager
            .get_group_members(conversation_id)
            .await
            .unwrap_or_default();

        let clients = self.clients.read().await;
        for member in members {
            if let Some(sender) = clients.get(&member.user_id) {
                if let Err(e) = sender.send(message.clone()) {
                    warn!("Failed to send message to user {}: {:?}", member.user_id, e);
                }
            }
        }

        Ok(())
    }

    async fn send_to_user(&self, user_id: &str, message: ServerMessage) -> Result<()> {
        let clients = self.clients.read().await;
        if let Some(sender) = clients.get(user_id) {
            sender.send(message)?;
        }
        Ok(())
    }

    async fn mark_user_online(&self, user_id: &str) {
        let mut online_users = self.online_users.write().await;
        online_users.insert(user_id.to_string(), Utc::now());
    }

    async fn mark_user_offline(&self, user_id: &str) {
        let mut online_users = self.online_users.write().await;
        online_users.remove(user_id);
    }
}

pub fn create_im_router(state: ImServerState) -> Router {
    Router::new()
        .route("/api/im/send", post(send_message))
        .route("/api/im/messages", get(get_messages))
        .route("/api/im/register", post(register_user))
        .route("/api/im/offline", get(get_offline_messages))
        .route("/api/im/online", get(get_online_users))
        .route("/api/im/check-online", get(check_user_online))
        .route("/api/im/pending", get(get_pending_messages))
        .route("/api/im/pending/clear", post(clear_pending_messages))
        .route("/api/im/sequence/update", post(update_user_sequence))
        .route("/api/im/sequence/get", get(get_user_sequence))
        .route("/api/im/sequence/sender", get(get_sender_sequence))
        .route("/api/im/missing/detect", get(detect_missing_messages))
        .route("/api/im/missing/fetch", get(fetch_missing_messages))
        .route("/api/im/receipt", post(send_message_receipt))
        .route("/api/im/receipts/:message_id", get(get_message_receipts))
        .route("/api/im/verify", post(verify_message))
        .route("/api/im/resend/request", post(request_resend_messages))
        .route("/api/sync/messages", get(sync_messages))
        .route("/api/sync/compressed", get(sync_messages_compressed))
        // P2P API routes
        .route("/api/p2p/topic/create", post(p2p_create_topic))
        .route("/api/p2p/topic/list", get(p2p_list_topics))
        .route("/api/p2p/node/register", post(p2p_register_node))
        .route("/api/p2p/node/list", get(p2p_list_nodes))
        .route("/api/p2p/sync", get(p2p_smart_sync))
        .route("/api/ws", get(websocket_handler))
        .with_state(state)
}

async fn send_message(
    State(state): State<ImServerState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, StatusCode> {
    debug!("Received message from {} in conversation {}", req.sender_id, req.conversation_id);

    // Check if receiver is online before sending
    let receiver_online = if let Some(ref receiver_id) = req.receiver_id {
        state.is_user_online(receiver_id).await
    } else {
        false
    };

    let db_message = match state
        .im_manager
        .send_message(
            &req.conversation_id,
            &req.sender_id,
            req.receiver_id.as_deref(),
            &req.content,
            req.reply_to.as_deref(),
        )
        .await
    {
        Ok(msg) => msg,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // If receiver is offline, add to pending queue
    if let Some(ref receiver_id) = req.receiver_id {
        if !receiver_online {
            debug!("Receiver {} is offline, adding to pending queue", receiver_id);
            if state.im_manager.add_pending_message(
                &db_message.id,
                receiver_id,
                &req.conversation_id,
            ).await.is_err() {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let server_msg = ServerMessage::NewMessage {
        conversation_id: db_message.conversation_id.clone(),
        sender_id: db_message.sender_id.clone(),
        content: db_message.content.clone(),
        timestamp: db_message.timestamp,
        sequence: db_message.sequence,
        integrity_hash: db_message.integrity_hash.clone(),
    };

    let conversation = match state
        .im_manager
        .get_conversation(&req.conversation_id)
        .await
    {
        Ok(Some(conv)) => conv,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if conversation.chat_type == ChatType::Group {
        let _ = state.broadcast_to_conversation(&req.conversation_id, server_msg).await;
    } else {
        if let Some(ref receiver_id) = req.receiver_id {
            if receiver_online {
                let _ = state.send_to_user(receiver_id, server_msg).await;
            }
        }
    }

    let response = SendMessageResponse {
        message_id: db_message.id,
        sequence: db_message.sequence,
        timestamp: db_message.timestamp,
    };

    Ok(Json(response))
}

async fn get_messages(
    State(state): State<ImServerState>,
    Query(query): Query<GetMessagesQuery>,
) -> Result<Json<Vec<DbMessage>>, StatusCode> {
    let messages = if let Some(after_seq) = query.after_sequence {
        state
            .im_manager
            .get_messages_by_sequence_range("", after_seq, after_seq + query.limit.unwrap_or(100))
            .await
    } else {
        state
            .im_manager
            .get_messages(&query.conversation_id, query.limit)
            .await
    };

    match messages {
        Ok(msgs) => Ok(Json(msgs)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn register_user(
    State(state): State<ImServerState>,
    Json(req): Json<RegisterUserRequest>,
) -> Result<Json<RegisterUserResponse>, StatusCode> {
    let user = match state
        .im_manager
        .create_user(&req.username, &req.display_name)
        .await
    {
        Ok(u) => u,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let response = RegisterUserResponse { user_id: user.id };
    Ok(Json(response))
}

async fn get_offline_messages(
    State(state): State<ImServerState>,
    Query(query): Query<GetOfflineMessagesQuery>,
) -> Result<Json<Vec<DbMessage>>, StatusCode> {
    let conversations = match state
        .im_manager
        .list_user_conversations(&query.user_id)
        .await
    {
        Ok(conv) => conv,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut all_messages = Vec::new();

    for conv in conversations {
        if let Some(ref conv_id) = query.conversation_id {
            if conv.id != *conv_id {
                continue;
            }
        }

        let messages = match state
            .im_manager
            .get_messages(&conv.id, query.limit)
            .await
        {
            Ok(msgs) => msgs,
            Err(_) => continue,
        };

        all_messages.extend(messages);
    }

    all_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(Json(all_messages))
}

async fn get_online_users(
    State(state): State<ImServerState>,
) -> Json<Vec<String>> {
    let online_users: Vec<String> = state.get_online_users().await;
    Json(online_users)
}

/// HTTP API: Check if a specific user is online
#[derive(Debug, Deserialize)]
struct CheckUserOnlineQuery {
    user_id: String,
}

#[derive(Debug, Serialize)]
struct CheckUserOnlineResponse {
    online: bool,
}

async fn check_user_online(
    State(state): State<ImServerState>,
    Query(query): Query<CheckUserOnlineQuery>,
) -> Json<CheckUserOnlineResponse> {
    let online = state.is_user_online(&query.user_id).await;
    Json(CheckUserOnlineResponse { online })
}

async fn get_pending_messages(
    State(state): State<ImServerState>,
    Query(query): Query<GetPendingMessagesQuery>,
) -> Result<Json<Vec<crate::manager::PendingMessage>>, StatusCode> {
    let pending = match state
        .im_manager
        .get_pending_messages(&query.user_id)
        .await
    {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(pending))
}

async fn clear_pending_messages(
    State(state): State<ImServerState>,
    Query(query): Query<ClearPendingMessagesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cleared = match state
        .im_manager
        .clear_pending_messages(&query.user_id)
        .await
    {
        Ok(c) => c,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(serde_json::json!({ "cleared": cleared })))
}

#[derive(Debug, Deserialize)]
struct GetPendingMessagesQuery {
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct ClearPendingMessagesQuery {
    user_id: String,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ImServerState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    info!("WebSocket connection from user: {}", user_id);
    ws.on_upgrade(move |socket| handle_websocket(socket, state, user_id))
}

async fn handle_websocket(
    socket: WebSocket,
    state: ImServerState,
    user_id: String,
) {
    let (tx, _rx) = broadcast::channel(100);

    state.mark_user_online(&user_id).await;

    {
        let mut clients = state.clients.write().await;
        clients.insert(user_id.clone(), tx.clone());
    }

    let pending: Vec<crate::manager::PendingMessage> = state.im_manager.get_pending_messages(&user_id).await.unwrap_or_default();
    if !pending.is_empty() {
        info!("User {} has {} pending messages", user_id, pending.len());
        
        for pending_msg in &pending {
            if let Ok(Some(msg)) = state.im_manager.get_message_by_id(&pending_msg.message_id).await {
                let server_msg = ServerMessage::NewMessage {
                    conversation_id: msg.conversation_id.clone(),
                    sender_id: msg.sender_id.clone(),
                    content: msg.content.clone(),
                    timestamp: msg.timestamp,
                    sequence: msg.sequence,
                    integrity_hash: msg.integrity_hash.clone(),
                };
                let _ = tx.send(server_msg);
            }
        }
    }

    let (mut sender, mut receiver) = socket.split();

    let mut rx = tx.subscribe();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let state_clone = state.clone();
    let user_id_clone = user_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Heartbeat => {
                                state_clone.mark_user_online(&user_id_clone).await;
                                debug!("Heartbeat from user: {}", user_id_clone);
                            }
                            ClientMessage::Typing { conversation_id } => {
                                let typing_msg = ServerMessage::Typing {
                                    conversation_id: conversation_id.clone(),
                                    user_id: user_id_clone.clone(),
                                };
                                let _ = state_clone.broadcast_to_conversation(&conversation_id, typing_msg).await;
                            }
                            ClientMessage::Subscribe { conversation_id } => {
                                debug!("User {} subscribed to conversation {}", user_id_clone, conversation_id);
                            }
                            ClientMessage::Unsubscribe { conversation_id } => {
                                debug!("User {} unsubscribed from conversation {}", user_id_clone, conversation_id);
                            }
                            ClientMessage::MessageReceipt { message_id, sequence } => {
                                debug!("Received message receipt from {} for message {} sequence {}", user_id_clone, message_id, sequence);
                                // Create receipt in database
                                let _ = state_clone.im_manager.create_message_receipt(
                                    &message_id,
                                    &user_id_clone,
                                    sequence,
                                ).await;
                                
                                // Forward receipt to sender
                                if let Ok(Some(msg)) = state_clone.im_manager.get_message_by_id(&message_id).await {
                                    let receipt_msg = ServerMessage::MessageReceipt {
                                        message_id: message_id.clone(),
                                        receiver_id: user_id_clone.clone(),
                                        sequence,
                                        received_at: Utc::now(),
                                    };
                                    let _ = state_clone.send_to_user(&msg.sender_id, receipt_msg).await;
                                }
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed by user: {}", user_id_clone);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    state.mark_user_offline(&user_id).await;
    {
        let mut clients = state.clients.write().await;
        clients.remove(&user_id);
    }

    info!("WebSocket disconnected for user: {}", user_id);
}

/// HTTP API: Update user sequence
#[derive(Debug, Deserialize)]
struct UpdateUserSequenceRequest {
    user_id: String,
    sender_id: String,
    last_sequence: u32,
}

#[derive(Debug, Serialize)]
struct UpdateUserSequenceResponse {
    success: bool,
}

async fn update_user_sequence(
    State(state): State<ImServerState>,
    Json(req): Json<UpdateUserSequenceRequest>,
) -> Result<Json<UpdateUserSequenceResponse>, StatusCode> {
    debug!("Updating user sequence for user {} from sender {}", req.user_id, req.sender_id);

    match state
        .im_manager
        .update_user_sequence(&req.user_id, &req.sender_id, req.last_sequence)
        .await
    {
        Ok(_) => Ok(Json(UpdateUserSequenceResponse { success: true })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// HTTP API: Get user sequence
#[derive(Debug, Deserialize)]
struct GetUserSequenceQuery {
    user_id: String,
    sender_id: String,
}

async fn get_user_sequence(
    State(state): State<ImServerState>,
    Query(query): Query<GetUserSequenceQuery>,
) -> Result<Json<Option<crate::manager::UserSequence>>, StatusCode> {
    let sequence = match state
        .im_manager
        .get_user_sequence(&query.user_id, &query.sender_id)
        .await
    {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(sequence))
}

/// HTTP API: Get sender sequence
#[derive(Debug, Deserialize)]
struct GetSenderSequenceQuery {
    sender_id: String,
}

async fn get_sender_sequence(
    State(state): State<ImServerState>,
    Query(query): Query<GetSenderSequenceQuery>,
) -> Result<Json<Option<crate::manager::SenderSequence>>, StatusCode> {
    let sequence = match state
        .im_manager
        .get_sender_sequence(&query.sender_id)
        .await
    {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(sequence))
}

/// HTTP API: Detect missing messages
#[derive(Debug, Deserialize)]
struct DetectMissingMessagesQuery {
    user_id: String,
    sender_id: String,
}

#[derive(Debug, Serialize)]
struct DetectMissingMessagesResponse {
    missing: bool,
    start_sequence: Option<u32>,
    end_sequence: Option<u32>,
    missing_count: u32,
}

async fn detect_missing_messages(
    State(state): State<ImServerState>,
    Query(query): Query<DetectMissingMessagesQuery>,
) -> Result<Json<DetectMissingMessagesResponse>, StatusCode> {
    let missing_range = match state
        .im_manager
        .detect_missing_messages(&query.user_id, &query.sender_id)
        .await
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match missing_range {
        Some((start, end)) => {
            let missing_count = if end >= start {
                end - start + 1
            } else {
                0
            };
            Ok(Json(DetectMissingMessagesResponse {
                missing: true,
                start_sequence: Some(start),
                end_sequence: Some(end),
                missing_count,
            }))
        }
        None => Ok(Json(DetectMissingMessagesResponse {
            missing: false,
            start_sequence: None,
            end_sequence: None,
            missing_count: 0,
        })),
    }
}

/// HTTP API: Fetch missing messages
#[derive(Debug, Deserialize)]
struct FetchMissingMessagesQuery {
    sender_id: String,
    start_sequence: u32,
    end_sequence: u32,
}

async fn fetch_missing_messages(
    State(state): State<ImServerState>,
    Query(query): Query<FetchMissingMessagesQuery>,
) -> Result<Json<Vec<DbMessage>>, StatusCode> {
    let messages = match state
        .im_manager
        .get_messages_by_sequence_range(
            &query.sender_id,
            query.start_sequence,
            query.end_sequence,
        )
        .await
    {
        Ok(msgs) => msgs,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(messages))
}

/// HTTP API: Send message receipt
#[derive(Debug, Deserialize)]
struct SendReceiptRequest {
    message_id: String,
    receiver_id: String,
    sequence: u32,
}

#[derive(Debug, Serialize)]
struct SendReceiptResponse {
    success: bool,
    receipt_id: String,
}

async fn send_message_receipt(
    State(state): State<ImServerState>,
    Json(req): Json<SendReceiptRequest>,
) -> Result<Json<SendReceiptResponse>, StatusCode> {
    debug!("Creating receipt for message {} from receiver {}", req.message_id, req.receiver_id);

    let receipt = match state
        .im_manager
        .create_message_receipt(&req.message_id, &req.receiver_id, req.sequence)
        .await
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Forward receipt to sender via WebSocket if online
    if let Ok(Some(msg)) = state.im_manager.get_message_by_id(&req.message_id).await {
        if state.is_user_online(&msg.sender_id).await {
            let receipt_msg = ServerMessage::MessageReceipt {
                message_id: req.message_id.clone(),
                receiver_id: req.receiver_id.clone(),
                sequence: req.sequence,
                received_at: receipt.received_at,
            };
            let _ = state.send_to_user(&msg.sender_id, receipt_msg).await;
        }
    }

    Ok(Json(SendReceiptResponse {
        success: true,
        receipt_id: receipt.id,
    }))
}

/// HTTP API: Get message receipts
async fn get_message_receipts(
    State(state): State<ImServerState>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<crate::manager::MessageReceipt>>, StatusCode> {
    let receipts = match state
        .im_manager
        .get_message_receipts(&message_id)
        .await
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(receipts))
}

/// HTTP API: Verify message integrity
#[derive(Debug, Deserialize)]
struct VerifyMessageRequest {
    message_id: String,
}

#[derive(Debug, Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    integrity_hash: Option<String>,
}

async fn verify_message(
    State(state): State<ImServerState>,
    Json(req): Json<VerifyMessageRequest>,
) -> Result<Json<VerifyMessageResponse>, StatusCode> {
    let message = match state
        .im_manager
        .get_message_by_id(&req.message_id)
        .await
    {
        Ok(Some(m)) => m,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let valid = crate::manager::ImManager::verify_message_integrity(&message);

    Ok(Json(VerifyMessageResponse {
        valid,
        integrity_hash: message.integrity_hash,
    }))
}

/// HTTP API: Request resend of missing messages
#[derive(Debug, Deserialize)]
struct ResendRequest {
    sender_id: String,
    start_sequence: u32,
    end_sequence: u32,
}

#[derive(Debug, Serialize)]
struct ResendResponse {
    success: bool,
    messages: Vec<crate::manager::Message>,
}

async fn request_resend_messages(
    State(state): State<ImServerState>,
    Json(req): Json<ResendRequest>,
) -> Result<Json<ResendResponse>, StatusCode> {
    debug!("Requesting resend of messages from {} sequence {} to {}", req.sender_id, req.start_sequence, req.end_sequence);

    let messages = match state
        .im_manager
        .get_messages_by_sequence_range(&req.sender_id, req.start_sequence, req.end_sequence)
        .await
    {
        Ok(msgs) => msgs,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(ResendResponse {
        success: true,
        messages,
    }))
}

/// HTTP API: Sync messages (with pagination)
#[derive(Debug, Deserialize)]
struct SyncMessagesQuery {
    conversation_id: String,
    after_sequence: Option<u32>,
    limit: Option<usize>,
}

async fn sync_messages(
    State(state): State<ImServerState>,
    Query(query): Query<SyncMessagesQuery>,
) -> Result<Json<SyncResponse>, StatusCode> {
    debug!("Sync messages for conversation {} after sequence {:?}", query.conversation_id, query.after_sequence);

    let limit = query.limit.unwrap_or(100);
    let sync_response = match state
        .im_manager
        .get_messages_for_sync(&query.conversation_id, query.after_sequence, limit)
        .await
    {
        Ok(r) => r,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Json(sync_response))
}

// P2P API: Create topic
#[derive(Debug, Deserialize)]
struct CreateTopicRequest {
    name: String,
}

#[derive(Debug, Serialize)]
struct CreateTopicResponse {
    topic_id: String,
    name: String,
}

async fn p2p_create_topic(
    State(state): State<ImServerState>,
    Json(req): Json<CreateTopicRequest>,
) -> Result<Json<CreateTopicResponse>, StatusCode> {
    let p2p = state.p2p().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let topic_id = p2p.topic().create_topic(&req.name).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(CreateTopicResponse {
        topic_id: topic_id.to_hex(),
        name: req.name,
    }))
}

// P2P API: List topics
#[derive(Debug, Serialize)]
struct ListTopicsResponse {
    topics: Vec<TopicInfo>,
}

#[derive(Debug, Serialize)]
struct TopicInfo {
    id: String,
    name: String,
    member_count: usize,
}

async fn p2p_list_topics(
    State(state): State<ImServerState>,
) -> Result<Json<ListTopicsResponse>, StatusCode> {
    let p2p = state.p2p().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let topics = p2p.topic().list_topics().await;
    
    let topic_infos: Vec<TopicInfo> = topics.into_iter().map(|t| TopicInfo {
        id: t.id.to_hex(),
        name: t.name,
        member_count: t.member_count,
    }).collect();
    
    Ok(Json(ListTopicsResponse {
        topics: topic_infos,
    }))
}

// P2P API: Register node
#[derive(Debug, Deserialize)]
struct RegisterNodeRequest {
    node_id: String,
    address: String,
    topics: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegisterNodeResponse {
    success: bool,
}

async fn p2p_register_node(
    State(state): State<ImServerState>,
    Json(req): Json<RegisterNodeRequest>,
) -> Result<Json<RegisterNodeResponse>, StatusCode> {
    let p2p = state.p2p().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let addr: std::net::SocketAddr = req.address.parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let node_info = NodeInfo {
        id: req.node_id.clone(),
        address: addr,
        last_seen: chrono::Utc::now(),
        topics: req.topics,
    };
    
    p2p.node().upsert_node(node_info.clone()).await;
    p2p.node().set_local_node(node_info).await;
    
    Ok(Json(RegisterNodeResponse {
        success: true,
    }))
}

// P2P API: List nodes
#[derive(Debug, Serialize)]
struct ListNodesResponse {
    nodes: Vec<NodeInfo>,
}

async fn p2p_list_nodes(
    State(state): State<ImServerState>,
) -> Result<Json<ListNodesResponse>, StatusCode> {
    let p2p = state.p2p().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let nodes = p2p.node().list_nodes().await;
    
    Ok(Json(ListNodesResponse {
        nodes,
    }))
}

// P2P API: Smart sync
#[derive(Debug, Deserialize)]
struct P2PSyncQuery {
    topic_id: String,
    last_sequence: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Serialize)]
struct P2PSyncResponse {
    mode: String,
    messages: Vec<crate::manager::Message>,
}

async fn p2p_smart_sync(
    State(state): State<ImServerState>,
    Query(query): Query<P2PSyncQuery>,
) -> Result<Json<P2PSyncResponse>, StatusCode> {
    let p2p = state.p2p().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let topic_id = crate::p2p::TopicId::from_hex(&query.topic_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let config = SyncConfig::default();
    let sync_strategy = p2p.create_sync_strategy(config);
    
    let last_sequence = query.last_sequence.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);
    
    let mode = sync_strategy.determine_sync_mode(&topic_id, last_sequence).await;
    let messages = sync_strategy.sync_messages(&topic_id, last_sequence, limit).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(P2PSyncResponse {
        mode: format!("{:?}", mode),
        messages,
    }))
}

/// HTTP API: Sync messages (compressed)
#[derive(Debug, Deserialize)]
struct SyncCompressedQuery {
    conversation_id: String,
    after_sequence: Option<u32>,
    limit: Option<usize>,
}

async fn sync_messages_compressed(
    State(state): State<ImServerState>,
    Query(query): Query<SyncCompressedQuery>,
) -> Result<axum::response::Response, StatusCode> {
    debug!("Sync compressed messages for conversation {} after sequence {:?}", query.conversation_id, query.after_sequence);

    let limit = query.limit.unwrap_or(1000);
    let compressed = match state
        .im_manager
        .get_compressed_messages_for_sync(&query.conversation_id, query.after_sequence, limit)
        .await
    {
        Ok(data) => data,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/octet-stream")
        .header("content-encoding", "zstd")
        .body(axum::body::Body::from(compressed))
        .unwrap();

    Ok(response)
}
