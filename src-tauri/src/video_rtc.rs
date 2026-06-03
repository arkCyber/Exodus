//! Exodus Browser — in-process WebRTC signaling (WeChat-style calls + meetings).
//!
//! Persists sessions under `{app_data}/video_rtc/` and mirrors signals to the P2P gossip
//! microservice when available. Real-time media uses WebRTC in the frontend.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Gossip topic prefix for 1:1 peer channels.
pub const RTC_PEER_TOPIC_PREFIX: &str = "exodus-rtc-peer-";
/// Gossip topic prefix for meeting rooms.
pub const RTC_MEETING_TOPIC_PREFIX: &str = "exodus-rtc-meeting-";

/// WebRTC signaling message (SDP / ICE / call control).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcSignalMessage {
    pub id: String,
    pub signal_type: String,
    pub session_id: String,
    pub from_node: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<Value>,
    pub timestamp: u64,
}

/// 1:1 call session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcCallSession {
    pub session_id: String,
    pub caller_node: String,
    pub callee_node: String,
    pub caller_name: String,
    pub callee_name: Option<String>,
    pub status: String,
    pub video_enabled: bool,
    pub audio_enabled: bool,
    pub created_at: u64,
    pub connected_at: Option<u64>,
    pub ended_at: Option<u64>,
}

/// Meeting room participant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcMeetingParticipant {
    pub node_id: String,
    pub display_name: String,
    pub joined_at: u64,
    pub video_enabled: bool,
    pub audio_enabled: bool,
}

/// Multi-party meeting room.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtcMeetingRoom {
    pub meeting_id: String,
    pub title: String,
    pub host_node: String,
    pub status: String,
    pub max_participants: u32,
    pub participants: Vec<RtcMeetingParticipant>,
    pub created_at: u64,
}

/// Local node + sessions registry.
pub struct VideoRtcState {
    pub node_id: String,
    display_name: Mutex<String>,
    storage_dir: PathBuf,
    calls: Mutex<HashMap<String, RtcCallSession>>,
    meetings: Mutex<HashMap<String, RtcMeetingRoom>>,
    signals: Mutex<HashMap<String, Vec<RtcSignalMessage>>>,
    last_poll: Mutex<HashMap<String, u64>>,
}

impl VideoRtcState {
    /// Create RTC hub under app data; reuse CDN node id when provided.
    pub fn new(app_data_dir: &Path, node_id: impl Into<String>, display_name: impl Into<String>) -> Result<Arc<Self>, String> {
        let storage_dir = app_data_dir.join("video_rtc");
        fs::create_dir_all(&storage_dir).map_err(|e| e.to_string())?;
        let hub = Arc::new(Self {
            node_id: node_id.into(),
            display_name: Mutex::new(display_name.into()),
            storage_dir,
            calls: Mutex::new(HashMap::new()),
            meetings: Mutex::new(HashMap::new()),
            signals: Mutex::new(HashMap::new()),
            last_poll: Mutex::new(HashMap::new()),
        });
        hub.load_persisted()?;
        Ok(hub)
    }

    /// Local display name shown to peers.
    pub fn display_name(&self) -> String {
        self.display_name
            .lock()
            .map(|n| n.clone())
            .unwrap_or_else(|_| "Exodus User".into())
    }

    /// Update local display name.
    pub fn set_display_name(&self, name: String) {
        if let Ok(mut n) = self.display_name.lock() {
            *n = name;
        }
    }

    fn load_persisted(&self) -> Result<(), String> {
        let calls_path = self.storage_dir.join("calls.json");
        if calls_path.exists() {
            let raw = fs::read_to_string(&calls_path).map_err(|e| e.to_string())?;
            if let Ok(map) = serde_json::from_str::<HashMap<String, RtcCallSession>>(&raw) {
                *self.calls.lock().map_err(|e| e.to_string())? = map;
            }
        }
        let meetings_path = self.storage_dir.join("meetings.json");
        if meetings_path.exists() {
            let raw = fs::read_to_string(&meetings_path).map_err(|e| e.to_string())?;
            if let Ok(map) = serde_json::from_str::<HashMap<String, RtcMeetingRoom>>(&raw) {
                *self.meetings.lock().map_err(|e| e.to_string())? = map;
            }
        }
        Ok(())
    }

    fn persist_calls(&self) -> Result<(), String> {
        let map = self.calls.lock().map_err(|e| e.to_string())?;
        let path = self.storage_dir.join("calls.json");
        fs::write(path, serde_json::to_string_pretty(&*map).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }

    fn persist_meetings(&self) -> Result<(), String> {
        let map = self.meetings.lock().map_err(|e| e.to_string())?;
        let path = self.storage_dir.join("meetings.json");
        fs::write(path, serde_json::to_string_pretty(&*map).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }

    /// Deterministic gossip topic for two node ids.
    pub fn peer_topic(node_a: &str, node_b: &str) -> String {
        let mut ids = [node_a.to_string(), node_b.to_string()];
        ids.sort();
        format!("{}{}-{}", RTC_PEER_TOPIC_PREFIX, ids[0], ids[1])
    }

    /// Meeting room gossip topic.
    pub fn meeting_topic(meeting_id: &str) -> String {
        format!("{RTC_MEETING_TOPIC_PREFIX}{meeting_id}")
    }

    /// Store signal locally and return message id.
    pub fn publish_signal(&self, topic: &str, mut msg: RtcSignalMessage) -> Result<String, String> {
        if msg.id.is_empty() {
            msg.id = Uuid::new_v4().to_string();
        }
        if msg.timestamp == 0 {
            msg.timestamp = now_secs();
        }
        let id = msg.id.clone();
        let mut signals = self.signals.lock().map_err(|e| e.to_string())?;
        let queue = signals.entry(topic.to_string()).or_default();
        queue.push(msg);
        while queue.len() > 2000 {
            queue.remove(0);
        }
        Ok(id)
    }

    /// Poll signals for a topic newer than `since` (unix secs).
    pub fn poll_signals(&self, topic: &str, since: u64) -> Result<Vec<RtcSignalMessage>, String> {
        let signals = self.signals.lock().map_err(|e| e.to_string())?;
        let list = signals.get(topic).cloned().unwrap_or_default();
        let out: Vec<_> = list.into_iter().filter(|m| m.timestamp > since).collect();
        if let Ok(mut last) = self.last_poll.lock() {
            let max_ts = out.iter().map(|m| m.timestamp).max().unwrap_or(since);
            last.insert(topic.to_string(), max_ts);
        }
        Ok(out)
    }

    /// Merge gossip-fetched payloads into local store.
    pub fn ingest_gossip_messages(&self, topic: &str, payloads: &[Value]) -> Result<(), String> {
        for value in payloads {
            if let Ok(mut msg) = serde_json::from_value::<RtcSignalMessage>(value.clone()) {
                if msg.id.is_empty() {
                    msg.id = Uuid::new_v4().to_string();
                }
                let _ = self.publish_signal(topic, msg);
            } else if let Some(inner) = value.get("payload") {
                if let Ok(mut msg) = serde_json::from_value::<RtcSignalMessage>(inner.clone()) {
                    if msg.id.is_empty() {
                        msg.id = Uuid::new_v4().to_string();
                    }
                    let _ = self.publish_signal(topic, msg);
                }
            }
        }
        Ok(())
    }

    /// Start a 1:1 call session.
    pub fn start_call(
        &self,
        callee_node: String,
        callee_name: Option<String>,
        video: bool,
        audio: bool,
    ) -> Result<RtcCallSession, String> {
        let session_id = Uuid::new_v4().to_string();
        let session = RtcCallSession {
            session_id: session_id.clone(),
            caller_node: self.node_id.clone(),
            callee_node,
            caller_name: self.display_name(),
            callee_name,
            status: "ringing".into(),
            video_enabled: video,
            audio_enabled: audio,
            created_at: now_secs(),
            connected_at: None,
            ended_at: None,
        };
        self.calls
            .lock()
            .map_err(|e| e.to_string())?
            .insert(session_id, session.clone());
        self.persist_calls()?;
        Ok(session)
    }

    /// Update call status.
    pub fn update_call_status(&self, session_id: &str, status: &str) -> Result<RtcCallSession, String> {
        let mut calls = self.calls.lock().map_err(|e| e.to_string())?;
        let session = calls
            .get_mut(session_id)
            .ok_or_else(|| format!("Call not found: {session_id}"))?;
        session.status = status.to_string();
        if status == "connected" {
            session.connected_at = Some(now_secs());
        }
        if status == "ended" || status == "rejected" {
            session.ended_at = Some(now_secs());
        }
        let out = session.clone();
        drop(calls);
        self.persist_calls()?;
        Ok(out)
    }

    pub fn get_call(&self, session_id: &str) -> Option<RtcCallSession> {
        self.calls.lock().ok()?.get(session_id).cloned()
    }

    pub fn list_calls(&self) -> Vec<RtcCallSession> {
        self.calls
            .lock()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Create a meeting room (host = local node).
    pub fn create_meeting(&self, title: String, max_participants: u32) -> Result<RtcMeetingRoom, String> {
        let meeting_id = format!("mtg-{}", &Uuid::new_v4().to_string()[..8]);
        let room = RtcMeetingRoom {
            meeting_id: meeting_id.clone(),
            title,
            host_node: self.node_id.clone(),
            status: "active".into(),
            max_participants: max_participants.max(2).min(12),
            participants: vec![RtcMeetingParticipant {
                node_id: self.node_id.clone(),
                display_name: self.display_name(),
                joined_at: now_secs(),
                video_enabled: true,
                audio_enabled: true,
            }],
            created_at: now_secs(),
        };
        self.meetings
            .lock()
            .map_err(|e| e.to_string())?
            .insert(meeting_id, room.clone());
        self.persist_meetings()?;
        Ok(room)
    }

    /// Join meeting as local node.
    pub fn join_meeting(
        &self,
        meeting_id: &str,
        display_name: Option<String>,
    ) -> Result<RtcMeetingRoom, String> {
        let mut meetings = self.meetings.lock().map_err(|e| e.to_string())?;
        let room = meetings
            .get_mut(meeting_id)
            .ok_or_else(|| format!("Meeting not found: {meeting_id}"))?;
        if room.participants.len() >= room.max_participants as usize {
            return Err("Meeting is full".into());
        }
        if !room.participants.iter().any(|p| p.node_id == self.node_id) {
            room.participants.push(RtcMeetingParticipant {
                node_id: self.node_id.clone(),
                display_name: display_name.unwrap_or_else(|| self.display_name()),
                joined_at: now_secs(),
                video_enabled: true,
                audio_enabled: true,
            });
        }
        let out = room.clone();
        drop(meetings);
        self.persist_meetings()?;
        Ok(out)
    }

    pub fn leave_meeting(&self, meeting_id: &str) -> Result<RtcMeetingRoom, String> {
        let mut meetings = self.meetings.lock().map_err(|e| e.to_string())?;
        let room = meetings
            .get_mut(meeting_id)
            .ok_or_else(|| format!("Meeting not found: {meeting_id}"))?;
        room.participants.retain(|p| p.node_id != self.node_id);
        if room.participants.is_empty() {
            room.status = "ended".into();
        }
        let out = room.clone();
        drop(meetings);
        self.persist_meetings()?;
        Ok(out)
    }

    pub fn get_meeting(&self, meeting_id: &str) -> Option<RtcMeetingRoom> {
        self.meetings.lock().ok()?.get(meeting_id).cloned()
    }

    pub fn list_meetings(&self) -> Vec<RtcMeetingRoom> {
        self.meetings
            .lock()
            .map(|m| m.values().filter(|r| r.status == "active").cloned().collect())
            .unwrap_or_default()
    }
}

fn now_secs() -> u64 {
    Utc::now().timestamp() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_topic_is_symmetric() {
        let t1 = VideoRtcState::peer_topic("exodus-b", "exodus-a");
        let t2 = VideoRtcState::peer_topic("exodus-a", "exodus-b");
        assert_eq!(t1, t2);
        assert!(t1.starts_with(RTC_PEER_TOPIC_PREFIX));
    }

    #[test]
    fn publish_and_poll_signals() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-test", "Tester").expect("hub");
        let topic = "exodus-rtc-peer-a-b";
        hub.publish_signal(
            topic,
            RtcSignalMessage {
                id: String::new(),
                signal_type: "ring".into(),
                session_id: "s1".into(),
                from_node: "exodus-a".into(),
                to_node: Some("exodus-b".into()),
                display_name: None,
                sdp: None,
                candidate: None,
                timestamp: 0,
            },
        )
        .expect("pub");
        let msgs = hub.poll_signals(topic, 0).expect("poll");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].signal_type, "ring");
    }

    #[test]
    fn test_meeting_topic() {
        let topic = VideoRtcState::meeting_topic("test-meeting-123");
        assert_eq!(topic, "exodus-rtc-meeting-test-meeting-123");
    }

    #[test]
    fn test_display_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-test", "Tester").expect("hub");
        assert_eq!(hub.display_name(), "Tester");
        
        hub.set_display_name("New Name".to_string());
        assert_eq!(hub.display_name(), "New Name");
    }

    #[test]
    fn test_start_call() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-caller", "Caller").expect("hub");
        
        let session = hub.start_call("exodus-callee".to_string(), Some("Callee".to_string()), true, false).expect("start call");
        
        assert_eq!(session.caller_node, "exodus-caller");
        assert_eq!(session.callee_node, "exodus-callee");
        assert_eq!(session.status, "ringing");
        assert!(session.video_enabled);
        assert!(!session.audio_enabled);
        assert!(session.callee_name.is_some());
    }

    #[test]
    fn test_update_call_status() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-caller", "Caller").expect("hub");
        
        let session = hub.start_call("exodus-callee".to_string(), Some("Callee".to_string()), true, false).expect("start call");
        let session_id = session.session_id.clone();
        
        let updated = hub.update_call_status(&session_id, "connected").expect("update status");
        assert_eq!(updated.status, "connected");
        assert!(updated.connected_at.is_some());
    }

    #[test]
    fn test_get_call() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-caller", "Caller").expect("hub");
        
        let session = hub.start_call("exodus-callee".to_string(), Some("Callee".to_string()), true, false).expect("start call");
        let session_id = session.session_id.clone();
        
        let retrieved = hub.get_call(&session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected call to exist").session_id, session_id);
    }

    #[test]
    fn test_list_calls() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-caller", "Caller").expect("hub");
        
        hub.start_call("exodus-callee-1".to_string(), Some("Callee 1".to_string()), true, false).expect("start call");
        hub.start_call("exodus-callee-2".to_string(), Some("Callee 2".to_string()), true, true).expect("start call");
        
        let calls = hub.list_calls();
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn test_create_meeting() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-host", "Host").expect("hub");
        
        let meeting = hub.create_meeting("Test Meeting".to_string(), 5).expect("create meeting");
        
        assert_eq!(meeting.host_node, "exodus-host");
        assert_eq!(meeting.title, "Test Meeting");
        assert_eq!(meeting.status, "active");
        assert_eq!(meeting.participants.len(), 1);
        assert_eq!(meeting.participants[0].node_id, "exodus-host");
    }

    #[test]
    fn test_join_meeting() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-host", "Host").expect("hub");
        
        let meeting = hub.create_meeting("Test Meeting".to_string(), 5).expect("create meeting");
        let meeting_id = meeting.meeting_id.clone();
        
        // Create a second hub for a different user
        let hub2 = VideoRtcState::new(tmp.path(), "exodus-joiner", "Joiner").expect("hub");
        let joined = hub2.join_meeting(&meeting_id, Some("Joiner".to_string())).expect("join meeting");
        
        assert_eq!(joined.participants.len(), 2);
    }

    #[test]
    fn test_leave_meeting() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-host", "Host").expect("hub");
        
        let meeting = hub.create_meeting("Test Meeting".to_string(), 5).expect("create meeting");
        let meeting_id = meeting.meeting_id.clone();
        
        let left = hub.leave_meeting(&meeting_id).expect("leave meeting");
        assert!(left.participants.is_empty());
        assert_eq!(left.status, "ended");
    }

    #[test]
    fn test_get_meeting() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-host", "Host").expect("hub");
        
        let meeting = hub.create_meeting("Test Meeting".to_string(), 5).expect("create meeting");
        let meeting_id = meeting.meeting_id.clone();
        
        let retrieved = hub.get_meeting(&meeting_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected meeting to exist").meeting_id, meeting_id);
    }

    #[test]
    fn test_list_meetings() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-host", "Host").expect("hub");
        
        hub.create_meeting("Meeting 1".to_string(), 5).expect("create meeting");
        hub.create_meeting("Meeting 2".to_string(), 5).expect("create meeting");
        
        let meetings = hub.list_meetings();
        assert_eq!(meetings.len(), 2);
    }

    #[test]
    fn test_signal_queue_limit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-test", "Tester").expect("hub");
        let topic = "test-topic";
        
        // Publish more than 2000 signals
        for i in 0..2100 {
            hub.publish_signal(
                topic,
                RtcSignalMessage {
                    id: format!("msg-{}", i),
                    signal_type: "test".into(),
                    session_id: "s1".into(),
                    from_node: "exodus-a".into(),
                    to_node: Some("exodus-b".into()),
                    display_name: None,
                    sdp: None,
                    candidate: None,
                    timestamp: i as u64,
                },
            )
            .expect("pub");
        }
        
        let msgs = hub.poll_signals(topic, 0).expect("poll");
        // Should be limited to 2000
        assert!(msgs.len() <= 2000);
    }

    #[test]
    fn test_ingest_gossip_messages() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let hub = VideoRtcState::new(tmp.path(), "exodus-test", "Tester").expect("hub");
        let topic = "test-topic";
        
        let payload = serde_json::json!({
            "id": "test-msg",
            "signalType": "offer",
            "sessionId": "s1",
            "fromNode": "exodus-a",
            "toNode": "exodus-b",
            "timestamp": 0
        });
        
        hub.ingest_gossip_messages(topic, &[payload]).expect("ingest");
        
        let msgs = hub.poll_signals(topic, 0).expect("poll");
        assert!(msgs.len() >= 1);
        assert_eq!(msgs[0].signal_type, "offer");
    }

    #[test]
    fn test_rtc_signal_message_serialization() {
        let msg = RtcSignalMessage {
            id: "test-id".to_string(),
            signal_type: "offer".to_string(),
            session_id: "session-1".to_string(),
            from_node: "node-1".to_string(),
            to_node: Some("node-2".to_string()),
            display_name: Some("Test User".to_string()),
            sdp: None,
            candidate: None,
            timestamp: 12345,
        };
        
        let json = serde_json::to_string(&msg).expect("serialize");
        let deserialized: RtcSignalMessage = serde_json::from_str(&json).expect("deserialize");
        
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.signal_type, "offer");
        assert_eq!(deserialized.from_node, "node-1");
    }

    #[test]
    fn test_rtc_call_session_serialization() {
        let session = RtcCallSession {
            session_id: "session-1".to_string(),
            caller_node: "node-1".to_string(),
            callee_node: "node-2".to_string(),
            caller_name: "Caller".to_string(),
            callee_name: Some("Callee".to_string()),
            status: "connected".to_string(),
            video_enabled: true,
            audio_enabled: false,
            created_at: 12345,
            connected_at: Some(12346),
            ended_at: None,
        };
        
        let json = serde_json::to_string(&session).expect("serialize");
        let deserialized: RtcCallSession = serde_json::from_str(&json).expect("deserialize");
        
        assert_eq!(deserialized.session_id, "session-1");
        assert_eq!(deserialized.status, "connected");
        assert!(deserialized.video_enabled);
    }
}
