//! Tauri commands for WebRTC signaling — 1:1 calls and meeting rooms.

use crate::microservice::gossip_client::gossip_json_rpc;
use crate::microservice::P2pGossipConfig;
use crate::p2p_cdn::P2pCdnState;
use crate::video_rtc::{RtcCallSession, RtcMeetingRoom, RtcSignalMessage, VideoRtcState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

/// Managed RTC hub.
pub struct VideoRtcManaged {
    pub hub: Arc<VideoRtcState>,
}

fn rtc_hub(app: &AppHandle) -> Result<Arc<VideoRtcState>, String> {
    Ok(app
        .try_state::<VideoRtcManaged>()
        .ok_or_else(|| "Video RTC not started — call video_rtc_service_start".to_string())?
        .hub
        .clone())
}

/// Ensure RTC hub is initialized (idempotent).
pub fn ensure_video_rtc(app: &AppHandle, app_data_dir: &std::path::Path) -> Result<(), String> {
    if app.try_state::<VideoRtcManaged>().is_some() {
        return Ok(());
    }
    let node_id = app
        .try_state::<Arc<P2pCdnState>>()
        .map(|c| c.node_id.clone())
        .unwrap_or_else(|| format!("exodus-{}", uuid::Uuid::new_v4()));
    let hub = VideoRtcState::new(app_data_dir, node_id, "Exodus User")?;
    app.manage(VideoRtcManaged { hub });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoRtcNodeInfo {
    pub node_id: String,
    pub display_name: String,
}

/// Start video RTC stack + gossip subscription.
#[tauri::command]
pub async fn video_rtc_service_start(app: AppHandle) -> Result<VideoRtcNodeInfo, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    ensure_video_rtc(&app, &app_data)?;
    let hub = rtc_hub(&app)?;

    let _ = crate::microservice::p2p_commands::p2p_gossip_service_start(app.clone()).await;
    let topic = format!("exodus-rtc-node-{}", hub.node_id);
    let _ = crate::microservice::p2p_commands::p2p_gossip_subscribe(
        topic.clone(),
        hub.node_id.clone(),
    )
    .await;

    Ok(VideoRtcNodeInfo {
        node_id: hub.node_id.clone(),
        display_name: hub.display_name(),
    })
}

#[tauri::command]
pub async fn video_rtc_node_info(app: AppHandle) -> Result<VideoRtcNodeInfo, String> {
    let hub = rtc_hub(&app)?;
    Ok(VideoRtcNodeInfo {
        node_id: hub.node_id.clone(),
        display_name: hub.display_name(),
    })
}

#[tauri::command]
pub async fn video_rtc_set_display_name(app: AppHandle, display_name: String) -> Result<(), String> {
    let hub = rtc_hub(&app)?;
    // display_name is on Arc inner — need interior mutability or recreate; use Mutex wrapper
    // VideoRtcState has pub display_name but not mutable - add method
    hub.set_display_name(display_name);
    Ok(())
}

#[tauri::command]
pub async fn video_rtc_publish_signal(
    app: AppHandle,
    topic: String,
    signal: RtcSignalMessage,
) -> Result<String, String> {
    let hub = rtc_hub(&app)?;
    let msg_id = hub.publish_signal(&topic, signal.clone())?;

    let payload = serde_json::to_value(&signal).map_err(|e| e.to_string())?;
    let gossip_path = P2pGossipConfig::default().socket_path;
    let topic_clone = topic.clone();
    let app_emit = app.clone();
    tauri::async_runtime::spawn(async move {
        if gossip_json_rpc(
            &gossip_path,
            "publish",
            json!({ "topic": topic_clone, "payload": payload }),
        )
        .await
        .is_ok()
        {
            return;
        }
        let _ = app_emit;
    });

    if signal.signal_type == "ring" {
        let _ = app.emit(
            "exodus-rtc-incoming-call",
            json!({
                "sessionId": signal.session_id,
                "fromNode": signal.from_node,
                "displayName": signal.display_name,
                "topic": topic,
            }),
        );
    }
    let _ = app.emit("exodus-rtc-signal", &signal);
    Ok(msg_id)
}

#[tauri::command]
pub async fn video_rtc_poll_signals(
    app: AppHandle,
    topic: String,
    since: Option<u64>,
) -> Result<Vec<RtcSignalMessage>, String> {
    let hub = rtc_hub(&app)?;
    let since_ts = since.unwrap_or(0);

    let gossip_path = P2pGossipConfig::default().socket_path;
    if let Ok(result) = gossip_json_rpc(
        &gossip_path,
        "get_messages",
        json!({ "topic": topic, "limit": 100 }),
    )
    .await
    {
        if let Some(arr) = result.get("messages").and_then(|v| v.as_array()) {
            let payloads: Vec<serde_json::Value> = arr
                .iter()
                .filter_map(|m| m.get("payload").cloned().or(Some(m.clone())))
                .collect();
            let _ = hub.ingest_gossip_messages(&topic, &payloads);
        }
    }

    hub.poll_signals(&topic, since_ts)
}

#[tauri::command]
pub async fn video_rtc_call_start(
    app: AppHandle,
    callee_node: String,
    callee_name: Option<String>,
    video_enabled: bool,
    audio_enabled: bool,
) -> Result<RtcCallSession, String> {
    let hub = rtc_hub(&app)?;
    let session = hub.start_call(callee_node.clone(), callee_name, video_enabled, audio_enabled)?;
    let topic = VideoRtcState::peer_topic(&hub.node_id, &callee_node);
    let signal = RtcSignalMessage {
        id: String::new(),
        signal_type: "ring".into(),
        session_id: session.session_id.clone(),
        from_node: hub.node_id.clone(),
        to_node: Some(callee_node),
        display_name: Some(hub.display_name()),
        sdp: None,
        candidate: None,
        timestamp: 0,
    };
    let _ = video_rtc_publish_signal(app.clone(), topic, signal).await;
    Ok(session)
}

#[tauri::command]
pub async fn video_rtc_call_update(
    app: AppHandle,
    session_id: String,
    status: String,
) -> Result<RtcCallSession, String> {
    let hub = rtc_hub(&app)?;
    hub.update_call_status(&session_id, &status)
}

#[tauri::command]
pub async fn video_rtc_call_list(app: AppHandle) -> Result<Vec<RtcCallSession>, String> {
    Ok(rtc_hub(&app)?.list_calls())
}

#[tauri::command]
pub async fn video_rtc_meeting_create(
    app: AppHandle,
    title: String,
    max_participants: Option<u32>,
) -> Result<RtcMeetingRoom, String> {
    let hub = rtc_hub(&app)?;
    let room = hub.create_meeting(title, max_participants.unwrap_or(6))?;
    let topic = VideoRtcState::meeting_topic(&room.meeting_id);
    let signal = RtcSignalMessage {
        id: String::new(),
        signal_type: "join".into(),
        session_id: room.meeting_id.clone(),
        from_node: hub.node_id.clone(),
        to_node: None,
        display_name: Some(hub.display_name()),
        sdp: None,
        candidate: None,
        timestamp: 0,
    };
    let _ = video_rtc_publish_signal(app, topic, signal).await;
    Ok(room)
}

#[tauri::command]
pub async fn video_rtc_meeting_join(
    app: AppHandle,
    meeting_id: String,
    display_name: Option<String>,
) -> Result<RtcMeetingRoom, String> {
    let hub = rtc_hub(&app)?;
    let room = hub.join_meeting(&meeting_id, display_name)?;
    let topic = VideoRtcState::meeting_topic(&meeting_id);
    let signal = RtcSignalMessage {
        id: String::new(),
        signal_type: "join".into(),
        session_id: meeting_id.clone(),
        from_node: hub.node_id.clone(),
        to_node: None,
        display_name: Some(hub.display_name()),
        sdp: None,
        candidate: None,
        timestamp: 0,
    };
    let _ = video_rtc_publish_signal(app.clone(), topic, signal).await;
    let _ = app.emit("exodus-rtc-meeting-update", &room);
    Ok(room)
}

#[tauri::command]
pub async fn video_rtc_meeting_leave(app: AppHandle, meeting_id: String) -> Result<RtcMeetingRoom, String> {
    let hub = rtc_hub(&app)?;
    let room = hub.leave_meeting(&meeting_id)?;
    let topic = VideoRtcState::meeting_topic(&meeting_id);
    let signal = RtcSignalMessage {
        id: String::new(),
        signal_type: "leave".into(),
        session_id: meeting_id,
        from_node: hub.node_id.clone(),
        to_node: None,
        display_name: Some(hub.display_name()),
        sdp: None,
        candidate: None,
        timestamp: 0,
    };
    let _ = video_rtc_publish_signal(app.clone(), topic, signal).await;
    let _ = app.emit("exodus-rtc-meeting-update", &room);
    Ok(room)
}

#[tauri::command]
pub async fn video_rtc_meeting_get(app: AppHandle, meeting_id: String) -> Result<RtcMeetingRoom, String> {
    rtc_hub(&app)?
        .get_meeting(&meeting_id)
        .ok_or_else(|| format!("Meeting not found: {meeting_id}"))
}

#[tauri::command]
pub async fn video_rtc_meeting_list(app: AppHandle) -> Result<Vec<RtcMeetingRoom>, String> {
    Ok(rtc_hub(&app)?.list_meetings())
}

#[tauri::command]
pub async fn video_rtc_peer_topic(
    app: AppHandle,
    remote_node: String,
) -> Result<String, String> {
    let hub = rtc_hub(&app)?;
    Ok(VideoRtcState::peer_topic(&hub.node_id, &remote_node))
}
