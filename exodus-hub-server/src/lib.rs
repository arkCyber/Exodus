//! Exodus Hub Server
//!
//! A core service for managing chat, groups, WeChat official accounts, video conferencing, and P2P coordination.
//! Provides HTTP/WebSocket API for message storage and forwarding, online status tracking, and offline message retrieval.

pub mod manager;
pub mod server;
pub mod group_assistant;
pub mod wechat_official;
pub mod video_conference;
pub mod p2p;

pub use manager::{ImManager, ChatType, Message, PendingMessage, UserSequence, SenderSequence, MessageReceipt, SyncRequest, SyncResponse};
pub use server::{ImServerState, create_im_router};
pub use group_assistant::{GroupAssistantService, GroupAssistantConfig, GroupAssistant, AssistantMessage};
pub use wechat_official::{WeChatOfficialService, OfficialAccount, OfficialMessage};
pub use video_conference::{VideoConferenceService, ConferenceRoom, Participant};
pub use p2p::{P2PManager, GossipManager, GossipEvent, TopicManager, TopicId, NodeInfo, NodeManager, StorageIntegration, SyncStrategy, SyncMode, SyncConfig};

